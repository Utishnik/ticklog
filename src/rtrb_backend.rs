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
//! Unlike `backend-ringbuffer`'s single shared lock (FIXO), rtrb splits the
//! ring so the producer and the drain each lock only their **own** half
//! (`Producer` vs `Consumer`), matching the ownership split of
//! `backend-ringbuf`. The chunk API is the distinguishing feature: records are
//! batched into ring chunks, and both the producer and the drain move whole
//! chunks, never single bytes.
//!
//! # Capacity
//!
//! rtrb's ring is sized in **slots** (one `u8` per slot). Like `backend-ringbuf`
//! the declared byte capacity is measured in slots, so `with_capacity` receives
//! a byte capacity and rtrb keeps one empty slot (its internal convention) so
//! the producer never observes the exact full bound.
//!
//! # Chunk accounting
//!
//! `chunk_size` (set per ring, default 1) is the number of **records** staged
//! by the producer before it flushes one rtrb chunk. `reserve` reserves staging
//! room for `total_size` bytes; `commit` appends to the staged Vec and, when
//! the staged record count reaches `chunk_size`, flushes the staged bytes as a
//! single rtrb chunk.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

use rtrb::Consumer as RtrbCons;
use rtrb::Producer as RtrbProdCr;
use rtrb::RingBuffer as RtrbRing;

/// Default number of staged records flushed per rtrb chunk when the harness
/// does not set one explicitly. The harness writes it through
/// [`crate::__private::__set_default_chunk_size`] before the runtime is built.
pub static DEFAULT_CHUNK_SIZE: AtomicUsize = AtomicUsize::new(1);

use crate::builder::Backpressure;
use crate::ring::Reservation;

/// A lock-free SPSC u8 ring backed by the `rtrb` crate's chunk API.
pub(crate) struct RingBuffer {
    /// Producer half. Locked only by the producer's `reserve`/`commit`.
    prod: Mutex<RtrbProdCr<u8>>,
    /// Consumer half. Locked only by the drain.
    cons: Mutex<RtrbCons<u8>>,
    /// Set to `false` by the producer on thread exit; the drain reads with
    /// Acquire to detect dead rings whose remaining bytes have been consumed.
    pub(crate) live: AtomicBool,
    /// Declared capacity in bytes (u8 slots).
    capacity: usize,
    /// Stable thread id of this ring's producer, set at registration.
    thread_id: AtomicU64,
    /// Producer thread name, set at registration.
    thread_name: Mutex<String>,
    /// Bytes staged by the producer since the last chunk flush.
    staged: Mutex<Vec<u8>>,
    /// Records currently staged in `staged` (elements of the same length
    /// contract as `impl Custom`/`ringbuf_backend`).
    staged_records: AtomicUsize,
    /// Records batched into one rtrb chunk before a flush.
    chunk_size: AtomicUsize,
}

// SAFETY: the split protocol hands the producer and the drain disjoint halves
// of the same SPSC channel; each half is confined to its own mutex and the
// AtomicBool live flag follows the same acquire/release contract as the
// `backend-ringbuf` backend.
unsafe impl Sync for RingBuffer {}

impl RingBuffer {
    /// Creates a new ring buffer of the default
    /// [`DEFAULT_RING_SIZE`](crate::ring::DEFAULT_RING_SIZE).
    #[allow(dead_code)] // used only by tests; `with_capacity` is the hot path
    pub(crate) fn new() -> Self {
        Self::with_capacity(crate::ring::DEFAULT_RING_SIZE)
    }

    /// Creates a new u8 ring of `capacity` bytes (slots).
    pub(crate) fn with_capacity(capacity: usize) -> Self {
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
        let (prod, cons) = RtrbRing::new(effective.saturating_sub(1));
        Self {
            prod: Mutex::new(prod),
            cons: Mutex::new(cons),
            live: AtomicBool::new(true),
            capacity: effective.saturating_sub(1),
            thread_id: AtomicU64::new(0),
            thread_name: Mutex::new(String::new()),
            staged: Mutex::new(Vec::new()),
            staged_records: AtomicUsize::new(0),
            chunk_size: AtomicUsize::new(DEFAULT_CHUNK_SIZE.load(Ordering::Relaxed).max(1)),
        }
    }

    /// Sets the number of records flushed per rtrb chunk for this ring. A
    /// non-zero `n` takes effect from the next [`commit`](Self::commit).
    pub(crate) fn set_chunk_size(&self, n: usize) {
        if n > 0 {
            self.chunk_size.store(n, Ordering::Relaxed);
        }
    }

    /// Records this ring's producer identity.
    pub(crate) fn set_thread_info(&self, thread_id: u64, thread_name: &str) {
        self.thread_id.store(thread_id, Ordering::Relaxed);
        let mut guard = self.thread_name.lock().unwrap_or_else(|e| e.into_inner());
        *guard = thread_name.to_string();
    }

    /// Identity used by the drain when formatting this ring's records.
    pub(crate) fn thread_id(&self) -> u64 {
        self.thread_id.load(Ordering::Relaxed)
    }

    /// Identity used by the drain when formatting this ring's records.
    pub(crate) fn thread_name(&self) -> String {
        self.thread_name
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Asserts there is at least `total_size` bytes of room in the ring plus
    /// currently staged bytes. Returns a dummy [`Reservation`] when there is.
    ///
    /// Mirrors [`backend-ringbuf`](crate::ringbuf_backend) exactly: staged
    /// bytes count against capacity because they are flushed to the ring at
    /// commit time. Under [`Backpressure::Drop`] returns `None` when the
    /// producer cannot flush its staged chunk + the new record; under
    /// [`Backpressure::Block`] (and the segmented policies, degraded) spins.
    pub(crate) fn reserve(&self, total_size: usize, policy: Backpressure) -> Option<Reservation> {
        let mut backoff = crate::backoff::Backoff::new();
        loop {
            let capacity = self.capacity;
            let staged_len = self.staged.lock().unwrap_or_else(|e| e.into_inner()).len();
            // Room needed to flush the staged chunk and place the new record.
            let needed = staged_len.saturating_add(total_size);
            let room = capacity;
            let prod = self.prod.lock().unwrap_or_else(|e| e.into_inner());
            // rtrb::Producer::slots() reports the *free* (writable) slots
            // (`capacity - distance`), so the occupied bytes are the declared
            // room minus the free slots. rtrb keeps its own internal slack so
            // the consumer can distinguish empty from full, which means the
            // drained tail only needs to have freed `needed` bytes.
            let free = prod.slots();
            let occupied = room.saturating_sub(free);
            let enough = occupied.saturating_add(needed) < room;
            drop(prod);
            if enough {
                return Some(Reservation {
                    ptr: std::ptr::null_mut(),
                    head: 0,
                });
            }
            match policy {
                Backpressure::Drop => return None,
                Backpressure::NanoLog | Backpressure::Quill | Backpressure::Block => {
                    if !self.live.load(Ordering::Relaxed) {
                        return None;
                    }
                    backoff.wait();
                }
            }
        }
    }

    /// Publish is a no-op for the rtrb backend: the real write happens inside
    /// [`commit`](Self::commit).
    #[allow(dead_code)] // part of the shared RingBuffer interface; custom-only
    #[inline(always)]
    pub(crate) fn publish(&self, _r: Reservation) {}

    /// Stages the record bytes and flushes a chunk to the rtrb ring when the
    /// staged record count reaches `chunk_size`.
    ///
    /// Must be called after [`reserve`](Self::reserve) succeeds. `reserve`
    /// guaranteed the staged Bytes + the new record together fit the ring, so
    /// the chunk flush never fails.
    pub(crate) fn commit(&self, bytes: &[u8]) {
        let chunk_size = self.chunk_size.load(Ordering::Relaxed).max(1);
        let mut staged = self.staged.lock().unwrap_or_else(|e| e.into_inner());
        staged.extend_from_slice(bytes);
        let records = self.staged_records.fetch_add(1, Ordering::Relaxed) + 1;
        if records >= chunk_size {
            let mut prod = self.prod.lock().unwrap_or_else(|e| e.into_inner());
            // SAFETY: `reserve` verified `staged.len()` bytes of room and rtrb
            // keeps one slot of slack, so `write_chunk` with the whole staged
            // slice can never fail.
            let to_flush = staged.len();
            if to_flush > 0 {
                let mut wchunk = prod
                    .write_chunk(to_flush)
                    .expect("ticklog staged flush: reserve promised room");
                {
                    let (a, b) = wchunk.as_mut_slices();
                    a.copy_from_slice(&staged[..a.len()]);
                    if !b.is_empty() {
                        b.copy_from_slice(&staged[a.len()..]);
                    }
                }
                wchunk.commit_all();
                staged.clear();
                self.staged_records.store(0, Ordering::Relaxed);
            }
        }
    }

    /// Moves every available ring byte into `out`, returning how many were
    /// moved. Called by the drain; never invoked by the producer.
    pub(crate) fn pop_available(&self, out: &mut Vec<u8>) -> usize {
        let mut cons = self.cons.lock().unwrap_or_else(|e| e.into_inner());
        if cons.is_empty() {
            return 0;
        }
        let before = out.len();
        let available = cons.slots();
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
            match cons.read_chunk(remaining) {
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

    /// Whether the FIFO currently holds no bytes.
    #[allow(dead_code)] // used only by tests
    pub(crate) fn is_empty(&self) -> bool {
        self.cons
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_empty()
    }

    /// Declared capacity in bytes.
    #[allow(dead_code)] // used by tests to validate capacity clamping
    pub(crate) fn capacity(&self) -> usize {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAP: usize = crate::ring::SLOT_SIZE * 256; // large enough for a few records
    const ROOM: usize = crate::ring::SLOT_SIZE; // a tiny power-of-two ring

    #[test]
    fn new_default_capacity() {
        let rb = RingBuffer::new();
        // `with_capacity` reports `effective - 1` (rtrb keeps one slot of
        // slack), so the default ring's declared capacity is default - 1.
        assert_eq!(rb.capacity(), crate::ring::DEFAULT_RING_SIZE - 1);
    }

    #[test]
    fn reserve_and_commit_fills_fifo() {
        let rb = RingBuffer::with_capacity(CAP);
        rb.set_chunk_size(1); // flush every record as its own chunk
        let payload = vec![0xABu8; 100];
        let slot = rb.reserve(payload.len(), Backpressure::Drop).unwrap();
        assert!(slot.ptr.is_null()); // dummy for rtrb; bytes move in commit()
        rb.commit(&payload);
        assert!(rb.live.load(Ordering::Relaxed));
        assert!(!rb.is_empty());
        let mut out = Vec::new();
        assert_eq!(rb.pop_available(&mut out), payload.len());
        assert_eq!(out, payload);
        assert!(rb.is_empty());
    }

    #[test]
    fn reserve_drops_when_full() {
        // Smallest valid ring; a record of `ROOM - 1` (which is `capacity - 1`
        // = `room`) cannot reserve because reserve requires
        // `occupied + needed < room` (rtrb keeps one slot of slack), so we fill
        // with `ROOM - 2` and then the next reserve must fail under Drop.
        let rb = RingBuffer::with_capacity(ROOM);
        let payload = vec![0u8; ROOM - 2];
        let _slot = rb.reserve(payload.len(), Backpressure::Drop).unwrap();
        rb.commit(&payload);
        assert!(rb.reserve(1, Backpressure::Drop).is_none());
    }

    #[test]
    fn publish_is_noop() {
        let rb = RingBuffer::with_capacity(CAP);
        let slot = rb.reserve(1, Backpressure::Drop).unwrap();
        rb.publish(slot); // must not panic or corrupt the caches
        assert!(rb.is_empty());
    }

    #[test]
    fn pop_available_moves_all_bytes_in_order() {
        let rb = RingBuffer::with_capacity(CAP);
        rb.set_chunk_size(1); // flush one record per chunk so order is exact
        rb.commit(b"abc");
        rb.commit(b"def");
        let mut out = Vec::new();
        assert_eq!(rb.pop_available(&mut out), 6);
        assert_eq!(out, b"abcdef");
        assert!(rb.is_empty());
    }

    #[test]
    fn pop_available_handles_wrapped_chunk() {
        // Regression: read_chunk may return a wrap (as_slices -> a|b with both
        // halves non-empty). The old code copy_from_slice'd `a` into a window
        // sized a.len()+b.len() and panicked.
        let rb = RingBuffer::with_capacity(ROOM);
        rb.set_chunk_size(1);
        let first = vec![0xAAu8; 40];
        rb.commit(&first);
        let mut out = Vec::new();
        assert_eq!(rb.pop_available(&mut out), 40);
        assert_eq!(out, first);
        // Second write starts near the physical end and wraps to the start.
        let second = vec![0xBBu8; 40];
        rb.reserve(second.len(), Backpressure::Drop).unwrap();
        rb.commit(&second);
        let mut wrapped = Vec::new();
        assert_eq!(rb.pop_available(&mut wrapped), 40);
        assert_eq!(wrapped, second);
        assert!(rb.is_empty());
    }

    #[test]
    fn staged_records_flush_only_when_chunk_size_reached() {
        let rb = RingBuffer::with_capacity(CAP);
        rb.set_chunk_size(2);
        // Two small records, then two >chunk_size; the first flush happens only
        // once the staged record count reaches 2, and the ring only becomes
        // non-empty after that flush.
        rb.commit(b"11");
        // staged: 1 record; chunk_size 2 -> not yet flushed -> ring stays empty
        assert!(rb.is_empty());
        rb.commit(b"22");
        assert!(!rb.is_empty());
        let mut out = Vec::new();
        assert_eq!(rb.pop_available(&mut out), 4);
        assert_eq!(out, b"1122");
        assert!(rb.is_empty());
    }
}
