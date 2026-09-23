//! Experimental SPSC ring backed by the `rtrb` crate, selected with the
//! `backend-rtrb` feature.
//!
//! `rtrb` is a lock-free real-time SPSC ring for `T` items (here `u8`) exposed
//! as a split [`Producer`]/[`Consumer`] pair through its **chunk API**: the
//! producer stages records and flushes them through rtrb's chunk interface
//! (`Producer::write_chunk`), so the drain consumes contiguous whole-chunk
//! byte slices instead of one record at a time. This amortises the producer →
//! drain publish over N staged records and lets the harness control the chunk
//! size at runtime via `--chunk-size` (records per ring chunk).
//!
//! # Ownership split (no hot-path locks)
//!
//! The ring is physically split into two owned halves:
//!
//! - [`RingProducer`] — the producer half, owned outright by this thread's
//!   [`ThreadBuf`](crate::thread_buf::ThreadBuf). Its staging buffer and
//!   chunk counter are plain fields: the hot path (`reserve`/`commit`) takes
//!   `&mut self` and touches **zero mutexes and zero atomics** for bookkeeping.
//! - [`Registration`] — the drain half (consumer + shared cold state),
//!   registered as an `Arc` in the global registry. The consumer sits behind
//!   a mutex that only the drain ever locks; shared identity (`live`,
//!   thread info, capacity) lives in an [`RingShared`].
//!
//! The two halves meet only through rtrb's own lock-free ring and the
//! `Arc`-shared cold state — mirroring the ownership split of
//! `backend-ringbuf`, without that backend's per-half `Mutex` wrappers.
//! The chunk API is the distinguishing feature: records are batched into ring
//! chunks, and both the producer and the drain move whole chunks, never
//! single bytes.
//!
//! # Capacity
//!
//! rtrb's ring is sized in **slots** (one `u8` per slot). Like `backend-ringbuf`
//! the declared byte capacity is measured in slots, so the split receives a
//! byte capacity and rtrb keeps one empty slot (its internal convention) so
//! the producer never observes the exact full bound.
//!
//! # Chunk accounting
//!
//! `chunk_size` (set per ring, default [`DEFAULT_CHUNK_SIZE`]) is the number
//! of **records** staged by the producer before it flushes one rtrb chunk.
//! `reserve` reserves staging room for `total_size` bytes; `commit` appends to
//! the staged Vec and, when the staged record count reaches `chunk_size`,
//! flushes the staged bytes as a single rtrb chunk.

use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

use rtrb::Consumer as RtrbCons;
use rtrb::Producer as RtrbProdCr;
use rtrb::RingBuffer as RtrbRing;

/// Default number of staged records flushed per rtrb chunk when the harness
/// does not set one explicitly. The harness writes it through
/// [`crate::__private::DEFAULT_CHUNK_SIZE`] before the runtime is built.
pub static DEFAULT_CHUNK_SIZE: AtomicUsize = AtomicUsize::new(1);

use crate::builder::Backpressure;
use crate::ring::Reservation;
use crate::sync::{AtomicBool, AtomicU64, Ordering};

/// Cold state shared between the producer half ([`RingProducer`]) and the
/// drain-side [`Registration`]. Never touched by the producer's hot path
/// except the `live` check inside a full-ring spin.
pub(crate) struct RingShared {
    /// Set to `false` by the producer on thread exit; the drain reads with
    /// Acquire to detect dead rings whose remaining bytes have been consumed.
    live: AtomicBool,
    /// Declared capacity in bytes (u8 slots), `effective - 1` (rtrb keeps one
    /// internal slot of slack).
    capacity: usize,
    /// Stable thread id of this ring's producer, set at registration.
    thread_id: AtomicU64,
    /// Producer thread name, set at registration. Drain-side only.
    thread_name: Mutex<String>,
}

impl RingShared {
    fn new(capacity: usize) -> Self {
        Self {
            live: AtomicBool::new(true),
            capacity,
            thread_id: AtomicU64::new(0),
            thread_name: Mutex::new(String::new()),
        }
    }

    /// Identity used by the drain when formatting this ring's records.
    fn thread_id(&self) -> u64 {
        self.thread_id.load(Ordering::Relaxed)
    }

    /// Identity used by the drain when formatting this ring's records.
    fn thread_name(&self) -> String {
        self.thread_name
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Records this ring's producer identity.
    fn set_thread_info(&self, thread_id: u64, thread_name: &str) {
        self.thread_id.store(thread_id, Ordering::Relaxed);
        let mut guard = self.thread_name.lock().unwrap_or_else(|e| e.into_inner());
        *guard = thread_name.to_string();
    }
}

/// Producer half of the split ring. Owned by the thread's
/// [`ThreadBuf`](crate::thread_buf::ThreadBuf); never shared, so staging is a
/// plain `Vec` and the hot path runs without any lock or atomic bookkeeping.
pub(crate) struct RingProducer {
    /// rtrb producer half; only this thread ever touches it.
    prod: RtrbProdCr<u8>,
    /// Bytes staged by the producer since the last chunk flush. Seeded to
    /// [`MAX_RECORD_SIZE`](crate::record::MAX_RECORD_SIZE) in [`split`] so the
    /// first post-`warm_up` `commit` never reallocates on the caller.
    staged: Vec<u8>,
    /// Records currently staged in `staged`.
    staged_records: usize,
    /// Records batched into one rtrb chunk before a flush.
    chunk_size: usize,
    /// Cold state shared with the drain-side [`Registration`].
    shared: Arc<RingShared>,
}

/// Consumer half of the split ring, owned by the drain-side [`Registration`]
/// behind its drain-only mutex.
pub(crate) struct RingConsumer {
    /// rtrb consumer half; only the drain (under the registration mutex)
    /// ever touches it.
    cons: RtrbCons<u8>,
}

impl RingConsumer {
    /// Moves every available ring byte into `out`, returning how many were
    /// moved. Called by the drain; never invoked by the producer.
    fn pop_available(&mut self, out: &mut Vec<u8>) -> usize {
        if self.cons.is_empty() {
            return 0;
        }
        let before = out.len();
        let available = self.cons.slots();
        out.resize(before + available, 0);
        let mut written = 0;
        // rtrb: read chunk by chunk so whole ring chunks are moved together.
        // A chunk may wrap the ring (as_slices -> a|b); copy each half into
        // its own exact-length window. Request only the remaining room so a
        // second iteration cannot write past `out`'s resize.
        loop {
            let remaining = available - written;
            if remaining == 0 {
                break;
            }
            match self.cons.read_chunk(remaining) {
                Err(_) => break,
                Ok(chunk) => {
                    let (a, b) = chunk.as_slices();
                    let n = a.len() + b.len();
                    out[before + written..before + written + a.len()].copy_from_slice(a);
                    if !b.is_empty() {
                        out[before + written + a.len()..before + written + n].copy_from_slice(b);
                    }
                    chunk.commit_all();
                    written += n;
                    if written >= available {
                        break;
                    }
                }
            }
        }
        out.truncate(before + written);
        written
    }
}

/// Drain-side registration of a split ring: the consumer half plus the shared
/// cold state. Exported from [`crate::ring`] as `RingBuffer` under the split
/// layout, so the drain, the registry, and the guard see the same type name
/// as every other backend.
pub(crate) struct Registration {
    /// Consumer half. Locked only by the drain (a drain-only critical
    /// section; the producer never contends for it).
    consumer: Mutex<RingConsumer>,
    /// Cold state shared with the producer half.
    shared: Arc<RingShared>,
}

/// Splits a new u8 ring of `capacity` bytes (slots) into its producer and
/// drain-side halves.
///
/// # Panics
///
/// Panics if `capacity` is not a power of two or is smaller than
/// [`SLOT_SIZE`](crate::ring::SLOT_SIZE).
pub(crate) fn split(capacity: usize) -> (RingProducer, Registration) {
    let effective = if capacity < crate::ring::SLOT_SIZE {
        crate::ring::SLOT_SIZE
    } else {
        capacity
    };
    assert!(
        effective.is_power_of_two(),
        "invariant: rtrb ring capacity must be a power of two, got {capacity}"
    );
    // rtrb keeps one empty slot so the consumer can distinguish empty from
    // full; ticklog's byte accounting therefore reports the capacity the
    // producer is allowed to fill as `capacity - 1`, matching every other
    // backend.
    let usable = effective.saturating_sub(1);
    let (prod, cons) = RtrbRing::new(usable);
    let shared = Arc::new(RingShared::new(usable));
    let producer = RingProducer {
        prod,
        // Seed to MAX_RECORD_SIZE so the first post-`warm_up` `commit` never
        // reallocates on the caller: with the default `chunk_size = 1` one
        // record is staged at a time and any record that passes the size gate
        // fits. Larger chunk batches may amortize a growth beyond this bound.
        staged: Vec::with_capacity(crate::record::MAX_RECORD_SIZE),
        staged_records: 0,
        chunk_size: DEFAULT_CHUNK_SIZE.load(Ordering::Relaxed).max(1),
        shared: Arc::clone(&shared),
    };
    let registration = Registration {
        consumer: Mutex::new(RingConsumer { cons }),
        shared,
    };
    (producer, registration)
}

impl RingProducer {
    /// Records this ring's producer identity in the shared cold state.
    pub(crate) fn set_thread_info(&self, thread_id: u64, thread_name: &str) {
        self.shared.set_thread_info(thread_id, thread_name);
    }

    /// Marks the producer side dead: no more records will be written. Release
    /// pairs with the drain's Acquire load of [`Registration::is_live`],
    /// guaranteeing all prior staging flushes are visible.
    pub(crate) fn set_dead(&self) {
        self.shared.live.store(false, Ordering::Release);
    }

    /// Whether the producer is still alive (Acquire). Tests only; the
    /// producer's own spin path reads `shared.live` with Relaxed directly.
    #[allow(dead_code)] // used only by tests
    pub(crate) fn is_live(&self) -> bool {
        self.shared.live.load(Ordering::Acquire)
    }

    /// Sets the number of records flushed per rtrb chunk for this ring. A
    /// non-zero `n` takes effect from the next [`commit`](Self::commit).
    #[allow(dead_code)] // per-ring override; the harness writes DEFAULT_CHUNK_SIZE
    pub(crate) fn set_chunk_size(&mut self, n: usize) {
        if n > 0 {
            self.chunk_size = n;
        }
    }

    /// Asserts there is at least `total_size` bytes of room in the ring plus
    /// currently staged bytes. Returns a dummy [`Reservation`] when there is.
    ///
    /// Mirrors [`backend-ringbuf`](crate::ringbuf_backend) exactly: staged
    /// bytes count against capacity because they are flushed to the ring at
    /// commit time. Under [`Backpressure::Drop`] returns `None` when the
    /// producer cannot flush its staged chunk + the new record; under
    /// [`Backpressure::Block`] (and the segmented policies, degraded) spins.
    pub(crate) fn reserve(
        &mut self,
        total_size: usize,
        policy: Backpressure,
    ) -> Option<Reservation> {
        let mut backoff = crate::backoff::Backoff::new();
        loop {
            let capacity = self.shared.capacity;
            // Room needed to flush the staged chunk and place the new record.
            let needed = self.staged.len().saturating_add(total_size);
            let room = capacity;
            // rtrb::Producer::slots() reports the *free* (writable) slots
            // (`capacity - distance`), so the occupied bytes are the declared
            // room minus the free slots. rtrb keeps its own internal slack so
            // the consumer can distinguish empty from full, which means the
            // drained tail only needs to have freed `needed` bytes.
            let free = self.prod.slots();
            let occupied = room.saturating_sub(free);
            let enough = occupied.saturating_add(needed) < room;
            if enough {
                return Some(Reservation {
                    ptr: std::ptr::null_mut(),
                    head: 0,
                });
            }
            match policy {
                Backpressure::Drop => return None,
                Backpressure::NanoLog | Backpressure::Quill | Backpressure::Block => {
                    if !self.shared.live.load(Ordering::Relaxed) {
                        return None;
                    }
                    backoff.wait();
                }
            }
        }
    }

    /// Stages the record bytes and flushes a chunk to the rtrb ring when the
    /// staged record count reaches `chunk_size`.
    ///
    /// Must be called after [`reserve`](Self::reserve) succeeds. `reserve`
    /// guaranteed the staged bytes + the new record together fit the ring, so
    /// the chunk flush never fails.
    ///
    /// With the default `chunk_size = 1` (and any batch where this record
    /// alone completes a chunk with nothing yet staged) the record is written
    /// straight into the ring chunk and the staging `Vec` is never touched,
    /// sparing one buffer memcpy per record.
    pub(crate) fn commit(&mut self, bytes: &[u8]) {
        let chunk_size = self.chunk_size.max(1);
        // Direct path: nothing staged, and this record completes a chunk.
        // `staged` empty implies `staged_records == 0`, so this is exactly
        // `chunk_size == 1` — the common configuration.
        if self.staged.is_empty() && chunk_size == 1 {
            if !bytes.is_empty() {
                // SAFETY: `reserve` verified `bytes.len()` bytes of room (the
                // staged byte count is zero, so `needed == total_size`) and
                // rtrb keeps one slot of slack, so `write_chunk` can never
                // fail.
                let mut wchunk = self
                    .prod
                    .write_chunk(bytes.len())
                    .expect("ticklog direct flush: reserve promised room");
                {
                    let (a, b) = wchunk.as_mut_slices();
                    a.copy_from_slice(&bytes[..a.len()]);
                    if !b.is_empty() {
                        b.copy_from_slice(&bytes[a.len()..]);
                    }
                }
                wchunk.commit_all();
            }
            return;
        }
        self.staged.extend_from_slice(bytes);
        self.staged_records += 1;
        if self.staged_records >= chunk_size {
            let to_flush = self.staged.len();
            if to_flush > 0 {
                // SAFETY: `reserve` verified `staged.len()` bytes of room and
                // rtrb keeps one slot of slack, so `write_chunk` with the
                // whole staged slice can never fail.
                let mut wchunk = self
                    .prod
                    .write_chunk(to_flush)
                    .expect("ticklog staged flush: reserve promised room");
                {
                    let (a, b) = wchunk.as_mut_slices();
                    a.copy_from_slice(&self.staged[..a.len()]);
                    if !b.is_empty() {
                        b.copy_from_slice(&self.staged[a.len()..]);
                    }
                }
                wchunk.commit_all();
                self.staged.clear();
                self.staged_records = 0;
            }
        }
    }
}

impl Registration {
    /// Moves every available ring byte into `out`, returning how many were
    /// moved. Called by the drain; never invoked by the producer.
    pub(crate) fn pop_available(&self, out: &mut Vec<u8>) -> usize {
        let mut consumer = self.consumer.lock().unwrap_or_else(|e| e.into_inner());
        consumer.pop_available(out)
    }

    /// Whether the FIFO currently holds no bytes.
    #[allow(dead_code)] // used only by tests
    pub(crate) fn is_empty(&self) -> bool {
        self.consumer
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .cons
            .is_empty()
    }

    /// Declared capacity in bytes.
    #[allow(dead_code)] // used by tests to validate capacity clamping
    pub(crate) fn capacity(&self) -> usize {
        self.shared.capacity
    }

    /// Records this ring's producer identity in the shared cold state.
    #[allow(dead_code)] // production sets it on the RingProducer half
    pub(crate) fn set_thread_info(&self, thread_id: u64, thread_name: &str) {
        self.shared.set_thread_info(thread_id, thread_name);
    }

    /// Identity used by the drain when formatting this ring's records.
    pub(crate) fn thread_id(&self) -> u64 {
        self.shared.thread_id()
    }

    /// Identity used by the drain when formatting this ring's records.
    pub(crate) fn thread_name(&self) -> String {
        self.shared.thread_name()
    }

    /// Whether the producer is still alive. Acquire pairs with the
    /// producer's [`RingProducer::set_dead`] Release store.
    pub(crate) fn is_live(&self) -> bool {
        self.shared.live.load(Ordering::Acquire)
    }

    /// Marks the ring dead (guard shutdown / tests). Release pairs with the
    /// drain's Acquire load in `is_live`.
    pub(crate) fn set_dead(&self) {
        self.shared.live.store(false, Ordering::Release);
    }

    /// A live registration whose producer half was dropped without marking
    /// the ring dead (`RingProducer` has no Drop impl), for tests that only
    /// need the drain-side handle.
    #[cfg(test)]
    pub(crate) fn new() -> Self {
        let (_producer, registration) = split(crate::ring::DEFAULT_RING_SIZE);
        registration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAP: usize = crate::ring::SLOT_SIZE * 256; // large enough for a few records
    const ROOM: usize = crate::ring::SLOT_SIZE; // a tiny power-of-two ring

    #[test]
    fn split_default_capacity() {
        let (_producer, registration) = split(crate::ring::DEFAULT_RING_SIZE);
        // `split` reports `effective - 1` (rtrb keeps one slot of slack), so
        // the default ring's declared capacity is default - 1.
        assert_eq!(registration.capacity(), crate::ring::DEFAULT_RING_SIZE - 1);
    }

    #[test]
    fn reserve_and_commit_fills_fifo() {
        let (mut producer, registration) = split(CAP);
        producer.set_chunk_size(1); // flush every record as its own chunk
        let payload = vec![0xABu8; 100];
        let slot = producer.reserve(payload.len(), Backpressure::Drop).unwrap();
        assert!(slot.ptr.is_null()); // dummy for rtrb; bytes move in commit()
        producer.commit(&payload);
        assert!(registration.is_live());
        assert!(!registration.is_empty());
        let mut out = Vec::new();
        assert_eq!(registration.pop_available(&mut out), payload.len());
        assert_eq!(out, payload);
        assert!(registration.is_empty());
    }

    #[test]
    fn reserve_drops_when_full() {
        // Smallest valid ring; a record of `ROOM - 1` (which is `capacity - 1`
        // = `room`) cannot reserve because reserve requires
        // `occupied + needed < room` (rtrb keeps one slot of slack), so we fill
        // with `ROOM - 2` and then the next reserve must fail under Drop.
        let (mut producer, _registration) = split(ROOM);
        let payload = vec![0u8; ROOM - 2];
        let _slot = producer.reserve(payload.len(), Backpressure::Drop).unwrap();
        producer.commit(&payload);
        assert!(producer.reserve(1, Backpressure::Drop).is_none());
    }

    #[test]
    fn pop_available_moves_all_bytes_in_order() {
        let (mut producer, registration) = split(CAP);
        producer.set_chunk_size(1); // flush one record per chunk so order is exact
        producer.commit(b"abc");
        producer.commit(b"def");
        let mut out = Vec::new();
        assert_eq!(registration.pop_available(&mut out), 6);
        assert_eq!(out, b"abcdef");
        assert!(registration.is_empty());
    }

    #[test]
    fn chunk_size_one_commits_directly_without_staging() {
        // Default chunk_size = 1 means every commit takes the direct path: the
        // record goes straight into an rtrb chunk and the staging Vec is never
        // touched. Staged state must stay empty across direct commits.
        let (mut producer, registration) = split(CAP);
        producer.set_chunk_size(1);
        producer.commit(b"abc");
        assert!(producer.staged.is_empty());
        assert_eq!(producer.staged_records, 0);
        producer.commit(b"def");
        assert!(producer.staged.is_empty());
        assert_eq!(producer.staged_records, 0);
        let mut out = Vec::new();
        assert_eq!(registration.pop_available(&mut out), 6);
        assert_eq!(out, b"abcdef");
        assert!(registration.is_empty());
    }

    #[test]
    fn direct_commit_respects_reserve_capacity() {
        // The direct path must honor reserve's accounting exactly like the
        // staged path: two payloads of ROOM-2 cannot both fit a ROOM ring, so
        // the second reserve refuses under Drop.
        let (mut producer, registration) = split(ROOM);
        producer.set_chunk_size(1);
        let payload = vec![0u8; ROOM - 2];
        let _slot = producer.reserve(payload.len(), Backpressure::Drop).unwrap();
        producer.commit(&payload);
        assert!(producer.reserve(1, Backpressure::Drop).is_none());
        let mut out = Vec::new();
        assert_eq!(registration.pop_available(&mut out), ROOM - 2);
        assert!(registration.is_empty());
    }

    #[test]
    fn direct_and_staged_commits_interleave() {
        // Changing chunk_size mid-stream mixes direct (chunk_size 1) and
        // staged (chunk_size 2) commits; bytes must arrive in commit order.
        let (mut producer, registration) = split(CAP);
        producer.set_chunk_size(1);
        producer.commit(b"aa"); // direct path
        producer.set_chunk_size(2);
        producer.commit(b"bb"); // staged, 1 of 2
        assert!(!producer.staged.is_empty());
        producer.commit(b"cc"); // staged flush, 2 of 2
        assert!(producer.staged.is_empty());
        let mut out = Vec::new();
        assert_eq!(registration.pop_available(&mut out), 6);
        assert_eq!(out, b"aabbcc");
        assert!(registration.is_empty());
    }

    #[test]
    fn pop_available_handles_wrapped_chunk() {
        // Regression: read_chunk may return a wrap (as_slices -> a|b with both
        // halves non-empty). The old code copy_from_slice'd `a` into a window
        // sized a.len()+b.len() and panicked.
        let (mut producer, registration) = split(ROOM);
        producer.set_chunk_size(1);
        let first = vec![0xAAu8; 40];
        producer.commit(&first);
        let mut out = Vec::new();
        assert_eq!(registration.pop_available(&mut out), 40);
        assert_eq!(out, first);
        // Second write starts near the physical end and wraps to the start.
        let second = vec![0xBBu8; 40];
        producer.reserve(second.len(), Backpressure::Drop).unwrap();
        producer.commit(&second);
        let mut wrapped = Vec::new();
        assert_eq!(registration.pop_available(&mut wrapped), 40);
        assert_eq!(wrapped, second);
        assert!(registration.is_empty());
    }

    #[test]
    fn staged_records_flush_only_when_chunk_size_reached() {
        let (mut producer, registration) = split(CAP);
        producer.set_chunk_size(2);
        // Two small records, then two >chunk_size; the first flush happens only
        // once the staged record count reaches 2, and the ring only becomes
        // non-empty after that flush.
        producer.commit(b"11");
        // staged: 1 record; chunk_size 2 -> not yet flushed -> ring stays empty
        assert!(registration.is_empty());
        producer.commit(b"22");
        assert!(!registration.is_empty());
        let mut out = Vec::new();
        assert_eq!(registration.pop_available(&mut out), 4);
        assert_eq!(out, b"1122");
        assert!(registration.is_empty());
    }

    #[test]
    fn set_dead_marks_registration_dead() {
        let (producer, registration) = split(CAP);
        assert!(registration.is_live());
        assert!(producer.is_live());
        producer.set_dead();
        assert!(!registration.is_live());
        assert!(!producer.is_live());
    }

    #[test]
    fn thread_info_roundtrip_through_shared() {
        let (producer, registration) = split(CAP);
        producer.set_thread_info(7, "worker");
        assert_eq!(registration.thread_id(), 7);
        assert_eq!(registration.thread_name(), "worker");
        // Re-registration (handoff) overwrites the previous identity.
        producer.set_thread_info(8, "worker-2");
        assert_eq!(registration.thread_id(), 8);
        assert_eq!(registration.thread_name(), "worker-2");
    }
}
