//! A single-producer, single-consumer ring buffer for lock-free record passing.
//!
//! Two backends implement the same [`RingBuffer`] type:
//!
//! - **Default** (`custom` submodule): a crate-local, zero-copy SPSC ring whose
//!   control fields are padded to separate cache lines with
//!   [`crossbeam_utils::CachePadded`]. The producer writes records straight
//!   into raw ring memory.
//! - **Experimental** byte-FIFO backends, enabled by the mutually exclusive
//!   features `backend-ringbuffer` (the `ringbuffer` crate), `backend-ringbuf`
//!   (the `ringbuf` crate) and `backend-triple-buffer` (the `triple_buffer`
//!   crate — a lossy latest-value exchange, benchmark-only).
//!
//! The default backend and the three experimental ones share the same
//! per-ring capacity, configured through the `ring_capacity` key of
//! [`crate::configure!`].

/// Default ring capacity in bytes. Power of two so producer and drain can use
/// a bitmask for index math. Used when [`crate::configure!`] does not specify
/// `ring_capacity`. Public so the `configure!` macro can reference it through
/// `$crate::__private` from external crates.
pub const DEFAULT_RING_SIZE: usize = 1_048_576; // 1 MiB

/// Cache-line size in bytes.
///
/// Apple Silicon (M1/M2/M3/M4) uses 128-byte cache lines on P-cores.
/// Intel, AMD, and standard ARM64 use 64-byte cache lines.
pub(crate) const CACHE_LINE_SIZE: usize = {
    #[cfg(all(target_arch = "aarch64", target_vendor = "apple"))]
    {
        128
    }
    #[cfg(not(all(target_arch = "aarch64", target_vendor = "apple")))]
    {
        64
    }
};

/// Slot size in bytes. Records are placed at slot-aligned positions so
/// adjacent records never share a data cache line.
pub(crate) const SLOT_SIZE: usize = CACHE_LINE_SIZE;

/// A reserved write slot in the ring. `ptr` points to a contiguous region of
/// at least the requested size; `head` is the value to store (with Release
/// ordering) after the caller has written the record bytes.
///
/// The experimental FIFO backends construct a dummy reservation: they have no
/// raw memory to expose, so the producer never dereferences `ptr` under those
/// features (the write path stages the record and pushes it through the
/// backend's own commit step instead).
#[allow(dead_code)] // fields are read only by the custom backend / macros
pub(crate) struct Reservation {
    pub(crate) ptr: *mut u8,
    pub(crate) head: u64,
}

/// Round `n` up to the nearest multiple of `align`.
/// `align` must be a power of 2.
#[allow(dead_code)] // used only by the custom backend (and its drain/tests)
#[inline(always)]
pub(crate) const fn align_up(n: u64, align: u64) -> u64 {
    n.wrapping_add(align - 1) & !(align - 1)
}

#[cfg(not(feature = "fifo-backend"))]
mod custom {
    //! The default zero-copy SPSC ring.
    //!
    //! The producer writes records and advances `head` (Release); the drain
    //! reads `head` (Acquire), processes records, and advances `tail`
    //! (Release). Control fields are split across two cache lines — one for
    //! each side — so the producer and drain never contend for the same line
    //! on the hot path.
    //!
    //! Under the segmented policies ([`Backpressure::NanoLog`] and
    //! [`Backpressure::Quill`]) the ring storage is carved from the `r3` arena
    //! allocator and a full ring is *handed off* to the drain instead of grown
    //! in place; see [`crate::segments`].

    use std::cell::UnsafeCell;
    use std::sync::Arc;
    use std::sync::Mutex;

    use super::{Reservation, SLOT_SIZE, align_up};
    use crate::builder::Backpressure;
    use crate::record::{END_OF_BUFFER, MAX_RECORD_SIZE, VERSION};
    use crate::sync::{AtomicBool, AtomicU64, Ordering};

    /// How many `reserve`+`publish` calls elapse before the producer stores a
    /// new `head` watermark under the `watermark-head` feature. 1000 matches
    /// the cross-language harness's batch size, so the drain lags by at most
    /// one measurement batch.
    #[cfg(feature = "watermark-head")]
    pub(crate) const WATERMARK_HEAD_RECORDS: u64 = 1000;

    /// Where the ring's bytes live. The classic path uses a crate-allocated
    /// boxed slice; the segmented paths allocate from the shared `r3` arena so
    /// per-thread buffers never touch the global allocator on the hot path.
    // SAFETY: both variants own their memory for as long as the `RingBuffer`
    // (either the `Box` owns it, or the `Arc<r3::Arena>` keeps the region
    // alive). The raw arena pointer derives from an allocation that is never
    // reclaimed before the arena itself drops, and the arena is kept alive by
    // every ring that references it, so no ring can outlive its memory.
    pub(crate) enum DataStorage {
        /// Default storage: `UnsafeCell<u8>` interior mutability, same layout
        /// guarantees as before (never a whole-slice borrow racing a writer).
        Box(Box<[UnsafeCell<u8>]>),
        /// Arena-backed storage for the segmented policies. `_keep` pins the
        /// arena so the region outlives this ring.
        Arena {
            ptr: *mut u8,
            _keep: Arc<r3::Arena<u8>>,
        },
    }

    /// Producer-cache-line half of the control state. Only the producer
    /// writes `head`; only the producer touches `tail_cache`.
    #[repr(C)]
    struct ProducerLine {
        /// Monotonic write position. Producer stores with Release; drain loads
        /// with Acquire.
        head: AtomicU64,
        /// Producer's local copy of the drain's tail. Cached to avoid reading
        /// the drain's cache line on every record.
        tail_cache: UnsafeCell<u64>,
        /// Producer-private true write position. Only touched by the producing
        /// thread, like `tail_cache`. Under the `watermark-head` feature the
        /// shared `head` above is a lagging watermark and `reserve` bookkeeping
        /// reads this slot instead.
        #[cfg(feature = "watermark-head")]
        local_head: UnsafeCell<u64>,
        /// Records published since the last `head` watermark.
        #[cfg(feature = "watermark-head")]
        since_watermark: UnsafeCell<u64>,
    }

    /// Drain-cache-line half of the control state. Only the drain writes
    /// `tail`; only the drain touches `head_cache`.
    #[repr(C)]
    struct DrainLine {
        /// Monotonic read position. Drain stores with Release; producer loads
        /// with Acquire during capacity checks. The Release/Acquire pair keeps
        /// the producer from overwriting a wrapped slot the drain is still
        /// reading.
        tail: AtomicU64,
        /// Drain's local copy of the producer's head. Cached to detect new
        /// records without a redundant atomic load.
        head_cache: UnsafeCell<u64>,
    }

    /// A single-producer, single-consumer ring buffer with
    /// [`CachePadded`](crossbeam_utils::CachePadded) control fields.
    #[repr(C)]
    pub(crate) struct RingBuffer {
        /// Producer cache line: offsets 0..CACHE_LINE_SIZE.
        producer: crossbeam_utils::CachePadded<ProducerLine>,
        /// Drain cache line, one cache line past the producer.
        drain: crossbeam_utils::CachePadded<DrainLine>,

        /// Set to `false` by the producer on thread exit. The drain reads with
        /// Acquire to detect dead rings whose remaining records have been
        /// consumed.
        pub(crate) live: AtomicBool,

        // ===== Thread identity (registration metadata) =====
        /// Stable thread id of this ring's producer, set at registration (and
        /// each handoff, for recycled segments). Written by the producer,
        /// read by the drain under the registry lock / after the producer
        /// exits. Records themselves carry no thread bytes: the ring's
        /// identity is the identity of every record it holds.
        thread_id: AtomicU64,
        /// Producer thread name, set alongside [`thread_id`](Self::thread_id).
        /// The drain locks briefly once per pass (never per record) to
        /// snapshot the name it formats with.
        thread_name: Mutex<String>,

        /// Ring storage capacity in bytes. Power of two for bitmask indexing.
        capacity: usize,
        /// `capacity - 1`, the index mask.
        mask: u64,

        /// Backing storage (boxed by default, arena-backed for segmented
        /// policies).
        data: DataStorage,

        // ===== Segmented-policy fields =====
        /// Monotonic registration serial, assigned by
        /// [`crate::thread_buf::register_ring`]. Lets the drain tell a reused
        /// (recycled) ring object apart from its previous incarnation, so it
        /// never drains a re-incarnated ring through a stale list entry.
        serial: AtomicU64,
        /// Set while a blocked producer owns this ring for cooperative
        /// formatting (NanoLog pool exhaustion). The drain skips a reserved
        /// ring until the reservation clears, guaranteeing exactly one worker
        /// formats each handed-off segment.
        helper_reserved: AtomicBool,
        /// Set by the drain for the duration of a drain pass over this ring,
        /// so a blocked producer about to cooperatively format it waits for
        /// the drain to finish reading before taking over (see
        /// [`crate::segments`]). Paired Acquire/Release with
        /// [`reserve_for_helper`](Self::reserve_for_helper).
        draining: AtomicBool,
        /// True when this ring must be returned to the `r3`-arena pool after
        /// it is fully drained (NanoLog), instead of being dropped (Quill).
        recyclable: bool,
        /// True when the drain fully drained this ring and moved it back to
        /// the pool; prevents double-recycling.
        recycled: AtomicBool,
        /// Set by the producer the moment it switches to a fresh segment
        /// (handoff). Only after this flag is set may the drain recycle the
        /// (fully drained) ring back to the pool: before that the ring is
        /// still an active producer's current buffer and must stay put.
        handed_off: AtomicBool,
    }

    // SAFETY: RingBuffer is safe to share between threads because the SPSC
    // protocol guarantees the producer and drain never touch the same state
    // concurrently:
    //
    // - `producer.tail_cache` is written and read only by the producer.
    // - `drain.head_cache` is written and read only by the drain.
    // - `head` (producer-write, drain-read) and `tail` (drain-write,
    //   producer-read) are AtomicU64 with paired Acquire/Release ordering.
    // - The data region is partitioned: the producer writes to offsets
    //   [head..head+record_size], the drain reads from [tail..head]. The
    //   invariant tail <= head ensures these ranges never overlap. Each byte
    //   is accessed by exactly one thread at any time.
    // - `r3::Arena` is `Send + Sync` for `Send` element types (`u8` here);
    //   ring storage either owns its `Box` or keeps its arena alive via `Arc`.
    unsafe impl Send for RingBuffer {}

    // SAFETY: RingBuffer is safe to share between threads because the SPSC
    // protocol guarantees the producer and drain never touch the same state
    // concurrently (see the `Send` impl above; the raw storage pointer in the
    // arena-backed variant is only ever dereferenced under that protocol).
    unsafe impl Sync for RingBuffer {}

    impl RingBuffer {
        /// Creates a new ring buffer of the default
        /// [`DEFAULT_RING_SIZE`](super::DEFAULT_RING_SIZE).
        #[allow(dead_code)] // used only by tests; `with_capacity` is the hot path
        pub(crate) fn new() -> Self {
            Self::with_capacity(super::DEFAULT_RING_SIZE)
        }

        /// Creates a new ring buffer with `capacity` bytes of zero-initialized
        /// storage.
        ///
        /// # Panics
        ///
        /// Panics if `capacity` is not a power of two or is smaller than
        /// [`SLOT_SIZE`].
        pub(crate) fn with_capacity(capacity: usize) -> Self {
            assert!(
                capacity.is_power_of_two() && capacity >= SLOT_SIZE,
                "invariant: ring capacity must be a power of two >= SLOT_SIZE, got {capacity}"
            );
            // SAFETY: `UnsafeCell<u8>` is `repr(transparent)` over `u8`, so a
            // zero-filled `Box<[u8]>` and a `Box<[UnsafeCell<u8>]>` of the same
            // length share layout. The cast reinterprets the one heap buffer and
            // preserves the slice length metadata.
            let bytes = vec![0u8; capacity].into_boxed_slice();
            let data: Box<[UnsafeCell<u8>]> =
                unsafe { Box::from_raw(Box::into_raw(bytes) as *mut [UnsafeCell<u8>]) };
            Self::from_storage(DataStorage::Box(data), capacity, false)
        }

        /// Creates a zero-initialized ring backed by the shared `r3` arena.
        /// Arena regions are per-thread, so carving this ring never takes a
        /// global allocation lock and never touches the process allocator.
        pub(crate) fn with_capacity_arena(
            capacity: usize,
            arena: Arc<r3::Arena<u8>>,
            recyclable: bool,
        ) -> Self {
            assert!(
                capacity.is_power_of_two() && capacity >= SLOT_SIZE,
                "invariant: ring capacity must be a power of two >= SLOT_SIZE, got {capacity}"
            );
            // SAFETY: `alloc_uninitialized` gives `capacity` readable/writable
            // bytes inside a region the arena keeps alive; the `Arc` in the
            // storage keeps the arena (and thus the bytes) valid for the whole
            // lifetime of this ring. Zero-fill matches the `Box` constructor,
            // which the drain relies on to treat zeroed slots as empty.
            let slice = unsafe { arena.alloc_uninitialized(capacity) };
            let ptr = slice.as_mut_ptr() as *mut u8;
            // SAFETY: the whole `capacity`-byte region is initialized above.
            unsafe {
                std::ptr::write_bytes(ptr, 0, capacity);
            }
            Self::from_storage(
                DataStorage::Arena { ptr, _keep: arena },
                capacity,
                recyclable,
            )
        }

        /// Shared construction tail for both storage paths.
        fn from_storage(data: DataStorage, capacity: usize, recyclable: bool) -> Self {
            Self {
                producer: crossbeam_utils::CachePadded::new(ProducerLine {
                    head: AtomicU64::new(0),
                    tail_cache: UnsafeCell::new(0),
                    #[cfg(feature = "watermark-head")]
                    local_head: UnsafeCell::new(0),
                    #[cfg(feature = "watermark-head")]
                    since_watermark: UnsafeCell::new(0),
                }),
                drain: crossbeam_utils::CachePadded::new(DrainLine {
                    tail: AtomicU64::new(0),
                    head_cache: UnsafeCell::new(0),
                }),
                live: AtomicBool::new(true),
                thread_id: AtomicU64::new(0),
                thread_name: Mutex::new(String::new()),
                capacity,
                mask: (capacity - 1) as u64,
                data,
                serial: AtomicU64::new(0),
                helper_reserved: AtomicBool::new(false),
                draining: AtomicBool::new(false),
                recyclable,
                recycled: AtomicBool::new(false),
                handed_off: AtomicBool::new(false),
            }
        }

        /// Registration serial (see the field docs). Assigned by
        /// [`crate::thread_buf::register_ring`].
        #[inline(always)]
        pub(crate) fn serial(&self) -> u64 {
            self.serial.load(Ordering::Relaxed)
        }

        /// Assigns the next registration serial. Called by
        /// [`crate::thread_buf::register_ring`].
        pub(crate) fn set_serial(&self, serial: u64) {
            self.serial.store(serial, Ordering::Relaxed);
        }

        /// Records this ring's producer identity (stable thread id + name).
        /// The record layout stores no per-record thread bytes: the ring's
        /// identity is the identity of every record it holds. Called by
        /// [`crate::thread_buf`] at registration and at every handoff (so a
        /// recycled segment carries its new producer's identity).
        ///
        /// The caller must set the info *before* the ring becomes visible
        /// through the registry (registration), and the drain reads the info
        /// while holding the registry lock, so the values are stable for any
        /// sync that can observe the ring.
        pub(crate) fn set_thread_info(&self, thread_id: u64, thread_name: &str) {
            self.thread_id.store(thread_id, Ordering::Relaxed);
            let mut guard = self.thread_name.lock().expect("thread_name mutex poisoned");
            *guard = thread_name.to_string();
        }

        /// Identity used by the drain when formatting this ring's records.
        #[inline(always)]
        pub(crate) fn thread_id(&self) -> u64 {
            self.thread_id.load(Ordering::Relaxed)
        }

        /// Identity used by the drain when formatting this ring's records.
        pub(crate) fn thread_name(&self) -> String {
            let guard = self.thread_name.lock().expect("thread_name mutex poisoned");
            guard.clone()
        }

        /// Whether a blocked producer has reserved this segment for
        /// cooperative formatting (NanoLog pool exhaustion).
        #[inline(always)]
        pub(crate) fn helper_reserved(&self) -> bool {
            self.helper_reserved.load(Ordering::Acquire)
        }

        /// Marks the segment as reserved by its producer for cooperative
        /// formatting. The drain skips reserved segments until they clear.
        #[inline(always)]
        pub(crate) fn reserve_for_helper(&self) {
            self.helper_reserved.store(true, Ordering::Release);
        }

        /// Clears the helper reservation after the segment was formatted.
        #[inline(always)]
        pub(crate) fn clear_helper_reservation(&self) {
            self.helper_reserved.store(false, Ordering::Release);
        }

        /// Marks the ring as being read by the drain for one drain pass.
        /// A blocked producer waiting to cooperatively format this segment
        /// spins on [`draining`](Self::draining) before touching it.
        #[inline(always)]
        pub(crate) fn mark_draining(&self) {
            self.draining.store(true, Ordering::Release);
        }

        /// Clears the drain-read marker after a drain pass over the ring.
        #[inline(always)]
        pub(crate) fn clear_draining(&self) {
            self.draining.store(false, Ordering::Release);
        }

        /// Whether the drain is currently reading this ring.
        #[inline(always)]
        pub(crate) fn draining(&self) -> bool {
            self.draining.load(Ordering::Acquire)
        }

        /// Whether this ring's storage came from the bounded NanoLog pool and
        /// must be returned there after it is drained.
        #[inline(always)]
        pub(crate) fn recyclable(&self) -> bool {
            self.recyclable
        }

        /// Marks a fully drained pool ring as recycled so neither the drain
        /// nor a producer recycles it twice.
        pub(crate) fn mark_recycled(&self) -> bool {
            !self.recycled.swap(true, Ordering::AcqRel)
        }

        /// Resets all counters for reuse from the pool. The ring's storage is
        /// untouched (it still belongs to the arena), only the control state is
        /// rewound so the next producer starts from a clean, empty ring.
        ///
        /// Must only be called when no thread is reading or writing the ring.
        pub(crate) fn reset_for_reuse(&self) {
            self.producer.head.store(0, Ordering::Relaxed);
            self.drain.tail.store(0, Ordering::Relaxed);
            // SAFETY: the caller guarantees exclusive access (the ring is
            // parked in the pool, not referenced by producer or drain).
            unsafe {
                *self.producer.tail_cache.get() = 0;
                *self.drain.head_cache.get() = 0;
                #[cfg(feature = "watermark-head")]
                {
                    *self.producer.local_head.get() = 0;
                    *self.producer.since_watermark.get() = 0;
                }
            }
            self.helper_reserved.store(false, Ordering::Release);
            self.draining.store(false, Ordering::Release);
            self.recycled.store(false, Ordering::Release);
            self.handed_off.store(false, Ordering::Release);
            self.live.store(true, Ordering::Release);
        }

        /// Marks this ring as handed off: its producer switched to a fresh
        /// segment and will never write here again. Frees the ring for
        /// recycling once the drain has fully drained it.
        #[inline(always)]
        pub(crate) fn mark_handed_off(&self) {
            self.handed_off.store(true, Ordering::Release);
        }

        /// Whether the producer has moved past this ring.
        #[inline(always)]
        pub(crate) fn handed_off(&self) -> bool {
            self.handed_off.load(Ordering::Acquire)
        }

        /// Producer's `head` atomic: stored with Release on publish, loaded
        /// with Acquire by the drain.
        #[inline(always)]
        pub(crate) fn head(&self) -> &AtomicU64 {
            &self.producer.head
        }

        /// Drain's `tail` atomic: stored with Release by the drain, loaded
        /// with Acquire by the producer during capacity checks.
        #[inline(always)]
        pub(crate) fn tail(&self) -> &AtomicU64 {
            &self.drain.tail
        }

        /// Raw pointer to the producer's private `tail_cache` slot.
        #[inline(always)]
        pub(crate) fn tail_cache(&self) -> *mut u64 {
            self.producer.tail_cache.get()
        }

        /// The producer's current write position.
        ///
        /// Under `watermark-head` this is the producer-private `local_head`;
        /// otherwise `head` is written on every publish and doubles as the
        /// position (Relaxed load of the producer's own store is sufficient).
        #[inline(always)]
        fn producer_head(&self) -> u64 {
            #[cfg(feature = "watermark-head")]
            {
                // SAFETY: local_head is producer-private; only the producing
                // thread touches it, same ownership as `tail_cache`.
                unsafe { *self.producer.local_head.get() }
            }
            #[cfg(not(feature = "watermark-head"))]
            {
                self.head().load(Ordering::Relaxed)
            }
        }

        /// Publishes the producer's current write position to the drain as a
        /// watermark. Release pairs with the drain's Acquire load of `head`,
        /// so every byte written at or before the watermark is visible before
        /// the drain reads the region.
        #[cfg(feature = "watermark-head")]
        #[inline(always)]
        pub(crate) fn flush_watermark(&self) {
            // SAFETY: both slots are producer-private.
            unsafe {
                *self.producer.since_watermark.get() = 0;
                let h = *self.producer.local_head.get();
                self.head().store(h, Ordering::Release);
            }
        }

        /// Raw pointer to the drain's private `head_cache` slot.
        ///todo only test use
        #[inline(always)]
        pub(crate) fn head_cache(&self) -> *mut u64 {
            self.drain.head_cache.get()
        }

        /// Ring storage capacity in bytes.
        #[inline(always)]
        pub(crate) fn capacity(&self) -> usize {
            self.capacity
        }

        /// Index mask (`capacity - 1`) for bitmask indexing.
        #[inline(always)]
        pub(crate) fn mask(&self) -> u64 {
            self.mask
        }

        /// Base pointer of the ring storage, usable by either side for the
        /// byte ranges it owns.
        #[inline(always)]
        pub(crate) fn data_ptr(&self) -> *mut u8 {
            match &self.data {
                DataStorage::Box(data) => data.as_ptr() as *mut u8,
                DataStorage::Arena { ptr, .. } => *ptr,
            }
        }

        /// Reserves a slot in the ring for a record of `total_size` bytes.
        ///
        /// Returns a [`Reservation`] with a write pointer and the future `head`
        /// value, or `None` if the ring is full under [`Backpressure::Drop`].
        /// Under [`Backpressure::Block`] this spins until space frees, so it
        /// always returns `Some`.
        ///
        /// The caller writes exactly `total_size` bytes to `Reservation::ptr`,
        /// then calls [`publish`](Self::publish). The slot is slot-aligned so
        /// adjacent records never share a cache line.
        ///
        /// Single-producer: the calling thread is the sole writer of this
        /// ring's `head` and `tail_cache`.
        #[inline]
        pub(crate) fn reserve(
            &self,
            total_size: usize,
            policy: Backpressure,
        ) -> Option<Reservation> {
            debug_assert!(
                total_size <= MAX_RECORD_SIZE,
                "invariant: total_size must fit the u16 total_size field"
            );
            let total = total_size as u64;
            let aligned = align_up(total, SLOT_SIZE as u64);

            // The producer is the sole writer of its position. Under the
            // `watermark-head` feature the shared `head` atomic is a lagging
            // watermark, so bookkeeping reads the producer-private copy;
            // otherwise `head` doubles as the position (Relaxed load of the
            // producer's own store is sufficient).
            let head = self.producer_head();
            let offset = (head & self.mask()) as usize;
            let remaining_phys = (self.capacity() - offset) as u64;

            // If the record would straddle the physical end of the buffer, an
            // EndOfBuffer record first fills the tail and the real record wraps
            // to offset 0. Both the filler and the record must fit, so both
            // count toward the space this write needs.
            let wrap = aligned > remaining_phys;
            let needed = if wrap {
                core::hint::cold_path();
                aligned + remaining_phys
            } else {
                aligned
            };

            if !self.try_ensure_capacity(head, needed, policy) {
                return None;
            }

            // The producer reaches the buffer through a base pointer taken from
            // the shared `data` cells, never a slice reference that would race
            // the drain. The capacity check guarantees the drain's tail will not
            // enter `[head, head + needed)` while this write proceeds, so these
            // bytes are the producer's alone. The allocation outlives the call
            // via the shared `Arc<RingBuffer>`.
            let base = self.data_ptr();

            let mut next = head;
            if wrap {
                // `remaining_phys < aligned <= 65536` here (see the top of this
                // fn), so the u16 cast is exact.
                // SAFETY: `offset` starts a slot-aligned region of
                // `remaining_phys` bytes inside the ring; the EOB header
                // occupies its first 4 bytes.
                unsafe { write_eob(base.add(offset), remaining_phys as u16) };
                next = next.wrapping_add(remaining_phys);
            }

            // The record lands at the next slot boundary: offset 0 after a wrap.
            let dst = (next & self.mask()) as usize;
            let new_head = next.wrapping_add(aligned);

            Some(Reservation {
                // SAFETY: `dst` starts the `aligned`-byte region reserved above;
                // the caller writes `total_size` bytes, which is <= aligned, so
                // the write stays within the ring.
                ptr: unsafe { base.add(dst) },
                head: new_head,
            })
        }

        /// Publishes a reserved slot to the drain. Release pairs with the
        /// drain's Acquire load of `head`, so every byte written to `r.ptr` is
        /// visible before the drain sees the new head and reads the region.
        ///
        /// Under the `watermark-head` feature the drain-facing store happens
        /// once per [`WATERMARK_HEAD_RECORDS`] publishes; every call still
        /// records the true position producer-privately.
        #[inline(always)]
        pub(crate) fn publish(&self, r: Reservation) {
            #[cfg(feature = "watermark-head")]
            {
                // SAFETY: both slots are producer-private.
                unsafe {
                    *self.producer.local_head.get() = r.head;
                    *self.producer.since_watermark.get() += 1;
                }
                if unsafe { *self.producer.since_watermark.get() } >= WATERMARK_HEAD_RECORDS {
                    self.flush_watermark();
                }
            }
            #[cfg(not(feature = "watermark-head"))]
            {
                self.head().store(r.head, Ordering::Release);
            }
        }

        /// Ensures `[head, head + needed)` is free for the producer to write,
        /// applying `policy` when the ring is full.
        ///
        /// Returns `true` once the space is available, or `false` if the
        /// record must be dropped under [`Backpressure::Drop`].
        fn try_ensure_capacity(&self, head: u64, needed: u64, policy: Backpressure) -> bool {
            // Fast path: trust the cached tail. Occupancy after the write is
            // `(head - tail) + needed`; it fits when that does not exceed the
            // capacity.
            // SAFETY: `tail_cache` is producer-private; only this thread touches
            // it.
            let cached = unsafe { *self.tail_cache() };
            if head.wrapping_add(needed).wrapping_sub(cached) <= self.mask() {
                #[cfg(feature = "fast-tail-cache")]
                {
                    // Quill-style fast path: the cached tail may be stale (the
                    // drain keeps consuming), so skip the cross-core load of the
                    // real tail entirely. Occupancy computed from a stale tail
                    // only ever over-estimates real occupancy (real tail >=
                    // cached), so a false "fits" here never overwrites a slot
                    // the drain has not consumed. The load below only runs on
                    // the cold confirm path.
                    return true;
                }
                #[cfg(not(feature = "fast-tail-cache"))]
                {
                    let tail = self.tail().load(Ordering::Acquire);
                    unsafe { *self.tail_cache() = tail };
                    return true;
                }
            }

            let mut backoff = crate::backoff::Backoff::new();
            loop {
                core::hint::cold_path();
                // Refresh from the drain. Acquire pairs with the drain's Release
                // store of `tail`, so a freed slot's reads complete before the
                // producer reuses it.
                let tail = self.tail().load(Ordering::Acquire);
                // SAFETY: producer-private, as above.
                unsafe { *self.tail_cache() = tail };
                if head.wrapping_add(needed).wrapping_sub(tail) <= self.mask() {
                    return true;
                }
                match policy {
                    Backpressure::Drop => return false,
                    // Segmented policies never spin in place: a full segment
                    // is handed off and the producer switches to a fresh one.
                    // The "blocked while the pool is exhausted" behavior lives
                    // in the producer's handoff path (`crate::segments`).
                    Backpressure::NanoLog | Backpressure::Quill => return false,
                    Backpressure::Block => {
                        if !self.live.load(Ordering::Relaxed) {
                            return false;
                        }
                        // Adaptive backoff: pause hints first, then yield to the
                        // scheduler so the drain thread makes progress.
                        backoff.wait();
                        // Under model checking, bound the otherwise-infinite
                        // spin: a path where the consumer never runs would
                        // otherwise blow loom's branch budget. Real builds
                        // still spin forever (see `backoff::Backoff`).
                        #[cfg(ticklog_loom)]
                        if backoff.loom_spins() >= 8 {
                            return false;
                        }
                    }
                }
            }
        }
    }

    /// Writes a header-only `END_OF_BUFFER` record of `span` bytes at `ptr`.
    ///
    /// The drain reads only the 4-byte prefix (version, type, total_size), then
    /// skips the whole span and wraps to the start of the ring.
    ///
    /// # Safety
    ///
    /// `ptr` must start a writable region of at least 4 bytes within the ring's
    /// data allocation.
    unsafe fn write_eob(ptr: *mut u8, span: u16) {
        let size = span.to_le_bytes();
        // SAFETY: the caller guarantees `ptr..ptr+4` is writable and in-bounds.
        unsafe {
            ptr.write(VERSION);
            ptr.add(1).write(END_OF_BUFFER);
            ptr.add(2).write(size[0]);
            ptr.add(3).write(size[1]);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::mem;

        const CAP: usize = super::super::DEFAULT_RING_SIZE;

        fn data(rb: &RingBuffer) -> &[u8] {
            // SAFETY: single-threaded test; no concurrent writer aliases the
            // ring's data.
            unsafe { std::slice::from_raw_parts(rb.data_ptr() as *const u8, rb.capacity()) }
        }

        #[test]
        fn constants_are_power_of_two() {
            assert!(CAP.is_power_of_two());
            assert!(SLOT_SIZE.is_power_of_two());
        }

        #[test]
        fn default_capacity_is_1mb() {
            assert_eq!(CAP, 1024 * 1024);
        }

        #[test]
        fn ring_alignment_at_least_one_cache_line() {
            assert!(mem::align_of::<RingBuffer>() >= 64);
        }

        #[test]
        fn head_at_offset_zero() {
            let rb = RingBuffer::new();
            let base = &rb as *const RingBuffer as usize;
            let head_addr = rb.head() as *const AtomicU64 as usize;
            assert_eq!(head_addr - base, 0);
        }

        #[test]
        fn tail_starts_one_padded_line_after_head() {
            let rb = RingBuffer::new();
            let base = &rb as *const RingBuffer as usize;
            let head_addr = rb.head() as *const AtomicU64 as usize;
            let tail_addr = rb.tail() as *const AtomicU64 as usize;
            let stride = mem::size_of::<crossbeam_utils::CachePadded<ProducerLine>>();
            assert_eq!(tail_addr - base, stride);
            assert_eq!(head_addr - base, 0);
            assert!(tail_addr - head_addr >= 64);
        }

        #[test]
        fn head_and_tail_on_different_cache_lines() {
            let rb = RingBuffer::new();
            let head_addr = rb.head() as *const AtomicU64 as usize;
            let tail_addr = rb.tail() as *const AtomicU64 as usize;
            let diff = head_addr.abs_diff(tail_addr);
            assert!(diff >= 64);
        }

        #[test]
        fn with_capacity_accepts_power_of_two() {
            let rb = RingBuffer::with_capacity(64);
            assert_eq!(rb.capacity(), 64);
            assert_eq!(rb.mask(), 63);
        }

        #[test]
        fn new_initializes_head_to_zero() {
            let rb = RingBuffer::new();
            assert_eq!(rb.head().load(Ordering::Relaxed), 0);
        }

        #[test]
        fn new_initializes_tail_to_zero() {
            let rb = RingBuffer::new();
            assert_eq!(rb.tail().load(Ordering::Relaxed), 0);
        }

        #[test]
        fn new_initializes_tail_cache_to_zero() {
            let rb = RingBuffer::new();
            // SAFETY: this test is single-threaded; no other reference aliases
            // the UnsafeCell.
            let val = unsafe { *rb.tail_cache() };
            assert_eq!(val, 0);
        }

        #[test]
        fn new_initializes_head_cache_to_zero() {
            let rb = RingBuffer::new();
            // SAFETY: this test is single-threaded; no other reference aliases
            // the UnsafeCell.
            let val = unsafe { *rb.head_cache() };
            assert_eq!(val, 0);
        }

        #[test]
        fn new_sets_live_to_true() {
            let rb = RingBuffer::new();
            assert!(rb.live.load(Ordering::Relaxed));
        }

        #[test]
        fn new_zero_initializes_data_region() {
            let rb = RingBuffer::new();
            let cap = rb.capacity();
            assert_eq!(cap, CAP);
            let data = data(&rb);
            assert_eq!(data.len(), CAP);
            assert!(data.iter().all(|&b| b == 0));
        }

        #[test]
        fn tail_cache_and_head_cache_are_zeroed() {
            let rb = RingBuffer::new();
            // SAFETY: single-threaded test; no concurrent access.
            assert_eq!(unsafe { *rb.tail_cache() }, 0);
            assert_eq!(unsafe { *rb.head_cache() }, 0);
        }

        #[test]
        fn align_up_already_aligned() {
            assert_eq!(align_up(64, 64), 64);
            assert_eq!(align_up(128, 64), 128);
            assert_eq!(align_up(0, 64), 0);
        }

        #[test]
        fn align_up_not_aligned() {
            assert_eq!(align_up(1, 64), 64);
            assert_eq!(align_up(63, 64), 64);
            assert_eq!(align_up(65, 64), 128);
        }

        #[test]
        fn align_up_align_1_is_identity() {
            assert_eq!(align_up(0, 1), 0);
            assert_eq!(align_up(1, 1), 1);
            assert_eq!(align_up(42, 1), 42);
            assert_eq!(align_up(u64::MAX, 1), u64::MAX);
        }

        #[test]
        fn align_up_power_of_two_aligns() {
            assert_eq!(align_up(0, 2), 0);
            assert_eq!(align_up(1, 2), 2);
            assert_eq!(align_up(2, 2), 2);
            assert_eq!(align_up(3, 2), 4);

            assert_eq!(align_up(0, 8), 0);
            assert_eq!(align_up(7, 8), 8);
            assert_eq!(align_up(8, 8), 8);
            assert_eq!(align_up(9, 8), 16);
        }

        #[test]
        fn align_up_large_values() {
            // Near u64::MAX, ensure no overflow.
            let n = u64::MAX - 100;
            let aligned = align_up(n, 64);
            // Should round up to the next multiple of 64.
            assert_eq!(aligned % 64, 0);
            assert!(aligned >= n);
        }

        #[test]
        fn align_up_slot_size_boundary() {
            // Record sizes near common boundaries.
            let slot = SLOT_SIZE as u64;
            assert_eq!(align_up(0, slot), 0);
            assert_eq!(align_up(1, slot), slot);
            assert_eq!(align_up(slot - 1, slot), slot);
            assert_eq!(align_up(slot, slot), slot);
            assert_eq!(align_up(slot + 1, slot), 2 * slot);
        }

        #[test]
        fn bitmask_is_capacity_minus_one() {
            let rb = RingBuffer::with_capacity(1024);
            assert_eq!(rb.mask(), 1023);
        }

        #[test]
        fn head_wrap_with_bitmask() {
            // Free-running u64 head wraps naturally via bitmask.
            let rb = RingBuffer::with_capacity(1024);
            let head: u64 = 1024 + 42;
            let offset = head & rb.mask();
            assert_eq!(offset, 42);
        }

        #[test]
        fn head_at_exact_capacity_wraps_to_zero() {
            let rb = RingBuffer::with_capacity(1024);
            let head: u64 = 1024;
            let offset = head & rb.mask();
            assert_eq!(offset, 0);
        }

        #[test]
        fn ring_buffer_is_send() {
            fn assert_send<T: Send>() {}
            assert_send::<RingBuffer>();
        }

        #[test]
        fn ring_buffer_is_sync() {
            fn assert_sync<T: Sync>() {}
            assert_sync::<RingBuffer>();
        }

        #[test]
        fn write_record_places_record_and_advances_head() {
            let rb = RingBuffer::new();
            let record = [0xABu8; 40];
            let len = record.len();
            let slot = rb.reserve(len, Backpressure::Drop).unwrap();
            unsafe {
                std::ptr::copy_nonoverlapping(record.as_ptr(), slot.ptr, len);
            }
            rb.publish(slot);

            // Head advances by the slot-aligned record size.
            let aligned = align_up(40, SLOT_SIZE as u64);
            assert_eq!(rb.head().load(Ordering::Relaxed), aligned);

            // The bytes landed at offset 0.
            let d = data(&rb);
            assert_eq!(&d[..40], &record[..]);
        }

        #[test]
        fn write_record_wraps_with_eob_at_ring_end() {
            let rb = RingBuffer::new();
            let slot = SLOT_SIZE as u64;
            // Position the producer one slot from the physical end, ring empty.
            let start = CAP as u64 - slot;
            rb.head().store(start, Ordering::Relaxed);
            rb.tail().store(start, Ordering::Relaxed);
            unsafe { *rb.tail_cache() = start };

            // A record longer than one slot cannot fit the final slot, forcing a
            // wrap: an EOB fills the last slot and the record lands at offset 0.
            let record = vec![0xCDu8; slot as usize + 1];
            let aligned = align_up(record.len() as u64, slot); // == 2 * slot
            let len = record.len();
            let res = rb.reserve(len, Backpressure::Drop).unwrap();
            unsafe {
                std::ptr::copy_nonoverlapping(record.as_ptr(), res.ptr, len);
            }
            rb.publish(res);

            // Head advanced past the EOB filler (one slot) and the record.
            assert_eq!(rb.head().load(Ordering::Relaxed), start + slot + aligned);

            let d = data(&rb);
            // EOB header sits at the old offset and spans exactly one slot.
            let eob = (start & (CAP as u64 - 1)) as usize;
            assert_eq!(d[eob], VERSION);
            assert_eq!(d[eob + 1], END_OF_BUFFER);
            assert_eq!(u16::from_le_bytes([d[eob + 2], d[eob + 3]]) as u64, slot);
            // The record wrapped to offset 0.
            assert_eq!(&d[..record.len()], &record[..]);
        }

        #[test]
        fn write_record_drops_when_full_under_drop_policy() {
            let rb = RingBuffer::new();
            // Ring completely full: head is CAP ahead of tail.
            rb.head().store(CAP as u64, Ordering::Relaxed);
            rb.tail().store(0, Ordering::Relaxed);
            unsafe { *rb.tail_cache() = 0 };

            let record = [0u8; 40];
            assert!(rb.reserve(record.len(), Backpressure::Drop).is_none());
            // Head is unchanged: nothing was written.
            assert_eq!(rb.head().load(Ordering::Relaxed), CAP as u64);
        }

        #[test]
        fn write_record_block_unblocks_when_drain_frees_space() {
            let rb = std::sync::Arc::new(RingBuffer::new());
            // Start completely full.
            rb.head().store(CAP as u64, Ordering::Relaxed);
            rb.tail().store(0, Ordering::Relaxed);
            unsafe { *rb.tail_cache() = 0 };

            // A drain-role thread frees the whole ring shortly.
            let drain = std::sync::Arc::clone(&rb);
            let handle = std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(20));
                drain.tail().store(CAP as u64, Ordering::Release);
            });

            let record = [0x5Au8; 40];
            // Blocks until the drain thread frees space, then writes.
            let len = record.len();
            let slot = rb.reserve(len, Backpressure::Block).unwrap();
            unsafe {
                std::ptr::copy_nonoverlapping(record.as_ptr(), slot.ptr, len);
            }
            rb.publish(slot);
            handle.join().unwrap();

            let aligned = align_up(40, SLOT_SIZE as u64);
            assert_eq!(rb.head().load(Ordering::Relaxed), CAP as u64 + aligned);
            let d = data(&rb);
            assert_eq!(&d[..40], &record[..]);
        }
    }
}

// The backends are mutually exclusive features; if more than one is enabled
// the first in this order wins, so the re-export never collides.
#[cfg(all(feature = "backend-ringbuf", not(feature = "backend-ringbuffer")))]
pub(crate) use crate::ringbuf_backend::RingBuffer;
#[cfg(feature = "backend-ringbuffer")]
pub(crate) use crate::ringbuffer_backend::RingBuffer;
#[cfg(all(
    feature = "backend-rtrb",
    not(feature = "backend-ringbuffer"),
    not(feature = "backend-ringbuf"),
    not(feature = "backend-triple-buffer")
))]
pub(crate) use crate::rtrb_backend::RingBuffer;
#[cfg(all(
    feature = "backend-triple-buffer",
    not(feature = "backend-ringbuffer"),
    not(feature = "backend-ringbuf")
))]
pub(crate) use crate::triple_buffer_backend::RingBuffer;
#[cfg(all(
    not(feature = "fifo-backend"),
    not(any(
        feature = "backend-ringbuffer",
        feature = "backend-ringbuf",
        feature = "backend-triple-buffer"
    ))
))]
pub(crate) use custom::RingBuffer;

#[cfg(test)]
mod tests {
    use super::*;

    // The default backend keeps the classic 1 MiB capacity so the public
    // layout stays stable; RingBuffer itself is re-exported from `custom`.
    #[test]
    fn default_size_is_reexported() {
        assert_eq!(DEFAULT_RING_SIZE, 1024 * 1024);
    }

    #[test]
    fn reservation_roundtrips_fields() {
        let ptr = 0x1234 as *mut u8;
        let r = Reservation { ptr, head: 42 };
        assert_eq!(r.ptr, ptr);
        assert_eq!(r.head, 42);
    }
}
