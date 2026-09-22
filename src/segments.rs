//! Arena-backed segmented buffers for the [`Backpressure::NanoLog`] and
//! [`Backpressure::Quill`] policies.
//!
//! Instead of spinning in place when its buffer is full, a producer *hands
//! the full segment to the drain* and switches to a fresh one. Buffer
//! storage is carved from the user's `r3::Arena` bump allocator (one arena
//! region per thread, no cross-thread lock on the allocation hot path).
//!
//! There are two flavors, selected by the policy:
//!
//! - **NanoLog** keeps a *bounded* pool of preallocated segments. When the
//!   whole pool is exhausted the producer blocks — but instead of spinning
//!   idly it does the drain's decoding/formatting work on its own
//!   handed-off segments (`[`Segments::format_helper_segment`]`), feeding
//!   the shared [`lines`] queue that the drain writes to the sink. Each
//!   formatted segment is recycled back into the pool, so one unit of work
//!   frees exactly the spare the producer needs. Memory is bounded by the
//!   pool; records are never dropped, only delayed.
//!
//! - **Quill** grows without bound: every handoff allocates a fresh segment
//!   from the arena, so the producer never blocks and never drops. Memory
//!   grows until the drain catches up (or the process runs out of memory).
//!
//! # Concurrency protocol
//!
//! A handed-off segment stays registered (live) in the registry until the
//! drain has fully drained it. The drain recycles it back to the pool
//! (NanoLog) or unregisters it (Quill) only after `handed_off` is set and
//! `tail == head`. A blocked NanoLog producer that takes over its own
//! segment to format it marks it `helper_reserved` *before* it formats, and
//! waits for the drain's `draining` flag to clear first, so the drain never
//! reads a segment the producer is concurrently formatting (and vice
//! versa). Re-registration bumps the segment's `serial`, which keeps the
//! drain from draining a reincarnated (reused) segment through a stale list
//! entry.
//!
//! Strict per-thread record order is guaranteed for the **Quill** policy
//! and for **NanoLog** while a thread never recycles a pool segment to a
//! different registry position; the helper's shared line queue is drained
//! once per poll pass, so records that flow through it may interleave with
//! records drained inline from other positions.

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex, OnceLock};

use crate::builder::Backpressure;
use crate::drain::format_segment;
use crate::format::Template;
use crate::level::Level;
use crate::ring::RingBuffer;
use crate::sync::Ordering;
use crate::timestamp::Calibration;

/// Total memory budget for the NanoLog segment pool, in bytes. The pool size
/// (number of segments) is derived from this budget and `ring_capacity`, so a
/// bigger per-thread buffer means fewer, bigger segments.
const NANOLOG_POOL_BUDGET: usize = 8 * 1024 * 1024;

/// Number of primary arena regions (one per allocating thread) for the Quill
/// arena. Additional allocations spill to freshly committed regions.
const QUILL_PRIMARY_REGIONS: usize = 8;

/// The global arena-backed segment pool, installed by
/// [`crate::builder::__configure_rt`] for the segmented policies. Absent for
/// the non-segmented policies.
pub(crate) static SEGMENTS: OnceLock<Segments> = OnceLock::new();

/// Arena-backed segmented-buffer machinery shared by the producer threads and
/// the drain (see the module docs for the protocol).
pub(crate) struct Segments {
    /// The shared `r3` arena all segment storage is carved from. Kept as an
    /// `Arc` so every `RingBuffer` alive pins the arena (and thus its bytes).
    pub(crate) arena: Arc<r3::Arena<u8>>,
    /// Policy this pool was built for. NanoLog is bounded; Quill is not.
    policy: Backpressure,
    /// Per-segment byte capacity (power of two).
    ring_capacity: usize,

    /// -- NanoLog bounded pool ------------------------------------------------
    /// Preallocated segments waiting to be claimed by a producer.
    spares: Mutex<Vec<Arc<RingBuffer>>>,
    /// Signals a producer that a segment was recycled back to the pool.
    spare_cv: Condvar,

    /// -- Shared line queue ---------------------------------------------------
    /// Formatted `(line, level)` pairs produced by blocked NanoLog producers
    /// helping with their own handed-off segments. The drain flushes this
    /// queue to the sink once per poll pass (the sink stays single-writer).
    lines: Mutex<VecDeque<(Vec<u8>, Level)>>,

    /// -- Formatter inputs for the helper path --------------------------------
    /// The producer-side helpers must reproduce the drain's formatting, so
    /// the configuration is copied here at setup.
    timezone_offset: i32,
    calibration: Calibration,
    line_pattern: Template,
}

impl Segments {
    /// Builds the pool machinery for a segmented policy. For NanoLog this
    /// preallocates the whole bounded pool up front; for Quill it only creates
    /// the arena (segments are carved on demand).
    pub(crate) fn init(
        policy: Backpressure,
        ring_capacity: usize,
        timezone_offset: i32,
        calibration: Calibration,
        line_pattern: Template,
    ) -> Segments {
        let pool_size = (NANOLOG_POOL_BUDGET / ring_capacity).max(1);
        Self::init_with_pool_size(
            policy,
            ring_capacity,
            pool_size,
            timezone_offset,
            calibration,
            line_pattern,
        )
    }

    /// Same as [`init`](Self::init) with an explicit pool size (primarily for
    /// tests that need a deliberately tiny pool to force exhaustion).
    pub(crate) fn init_with_pool_size(
        policy: Backpressure,
        ring_capacity: usize,
        pool_size: usize,
        timezone_offset: i32,
        calibration: Calibration,
        line_pattern: Template,
    ) -> Segments {
        match policy {
            Backpressure::NanoLog => {
                // One arena region per pool segment keeps every segment in a
                // single, thread-private region of the bump arena.
                let arena = Arc::new(r3::Arena::<u8>::with_regions(pool_size, ring_capacity));
                let mut spares = Vec::with_capacity(pool_size);
                for _ in 0..pool_size {
                    spares.push(Arc::new(RingBuffer::with_capacity_arena(
                        ring_capacity,
                        Arc::clone(&arena),
                        true,
                    )));
                }
                Segments {
                    arena,
                    policy,
                    ring_capacity,
                    spares: Mutex::new(spares),
                    spare_cv: Condvar::new(),
                    lines: Mutex::new(VecDeque::new()),
                    timezone_offset,
                    calibration,
                    line_pattern,
                }
            }
            Backpressure::Quill => {
                // Regions are sized to a few segments so a hot thread gets
                // several carve-alls-out of its primary region before spilling.
                let arena = Arc::new(r3::Arena::<u8>::with_regions(
                    QUILL_PRIMARY_REGIONS,
                    ring_capacity,
                ));
                Segments {
                    arena,
                    policy,
                    ring_capacity,
                    spares: Mutex::new(Vec::new()),
                    spare_cv: Condvar::new(),
                    lines: Mutex::new(VecDeque::new()),
                    timezone_offset,
                    calibration,
                    line_pattern,
                }
            }
            // The segmented machinery is only built for segmented policies;
            // the match is exhaustive to keep the invariants local.
            Backpressure::Drop | Backpressure::Block => unreachable!(),
        }
    }

    /// True if the global pool is installed (i.e. a segmented policy is active
    /// on the crate-local ring backend).
    pub(crate) fn installed() -> bool {
        SEGMENTS.get().is_some()
    }

    /// The installed pool. Only valid after a segmented configure ran.
    pub(crate) fn get() -> &'static Segments {
        SEGMENTS.get().expect(
            "invariant: segmented pool not installed; a NanoLog/Quill configure! must run first",
        )
    }

    /// Creates a producer's *first* ring for a segmented policy.
    ///
    /// The **NanoLog** variant claims a pooled segment, blocking until one is
    /// freed (with cooperative formatting of any outstanding helper segment).
    /// The **Quill** variant carves a fresh unbounded segment from the arena.
    ///
    /// `live` is passed for the handoff path so a producer blocked on an empty
    /// pool can detect that the drain was shut down (its current ring went
    /// dead) and bail instead of waiting forever. `None` is used at
    /// initialization, when the pool is guaranteed non-empty.
    pub(crate) fn first_ring(
        &self,
        helper: &mut Option<Arc<RingBuffer>>,
        live: Option<&crate::sync::AtomicBool>,
    ) -> Option<Arc<RingBuffer>> {
        match self.policy {
            Backpressure::NanoLog => self.take_pooled(helper, live),
            Backpressure::Quill => Some(self.alloc_fresh()),
            _ => unreachable!(),
        }
    }

    /// The NanoLog bounded pool: claim a spare, formatting a reserved helper
    /// segment first so the pool is replenished by our own work. `None` is
    /// returned if logging is being torn down (the caller's current ring went
    /// dead), so a producer never waits forever for a drain that was stopped.
    fn take_pooled(
        &self,
        helper: &mut Option<Arc<RingBuffer>>,
        live: Option<&crate::sync::AtomicBool>,
    ) -> Option<Arc<RingBuffer>> {
        loop {
            // Do the drain's formatting work on our own handed-off segment
            // first: it both frees a spare and makes the drain's backlog
            // progress without the drain thread.
            if let Some(ring) = helper.take() {
                self.format_helper_segment(ring);
            }
            if let Some(ring) = self.try_take_spare() {
                return Some(ring);
            }
            if let Some(l) = live
                && !l.load(Ordering::Relaxed)
            {
                return None;
            }
            let mut guard = self.spare_cv.wait(self.spares.lock().unwrap()).unwrap();
            if let Some(ring) = guard.pop() {
                ring.reset_for_reuse();
                return Some(ring);
            }
        }
    }

    /// Pops one pooled segment without blocking, rewinding its counters so the
    /// claimer starts from a clean, empty ring. (Only the drain recycles via
    /// `mark_recycled` without resetting, so every claim must rewind.)
    pub(crate) fn try_take_spare(&self) -> Option<Arc<RingBuffer>> {
        let ring = self.spares.lock().unwrap().pop()?;
        ring.reset_for_reuse();
        Some(ring)
    }

    /// Returns a fully-drained segment to the NanoLog pool and wakes any
    /// producer blocked on the pool.
    pub(crate) fn recycle(&self, ring: Arc<RingBuffer>) {
        self.spares.lock().unwrap().push(ring);
        self.spare_cv.notify_all();
    }

    /// The segment a producer thread starts with. Claims a pool spare if one
    /// is free, otherwise carves a fresh arena segment. Never blocks: a
    /// brand-new thread has no helper segment to format, so waiting on an
    /// empty pool could deadlock (e.g. a batch of threads spawned together
    /// where the pool holders have not started logging yet). A fresh segment
    /// is handed off and recycled into the pool after the drain retires it.
    pub(crate) fn initial_ring(&self) -> Arc<RingBuffer> {
        self.try_take_spare().unwrap_or_else(|| self.alloc_fresh())
    }

    /// Carves a fresh, reusable / disposable segment from the `r3` arena.
    pub(crate) fn alloc_fresh(&self) -> Arc<RingBuffer> {
        Arc::new(RingBuffer::with_capacity_arena(
            self.ring_capacity,
            Arc::clone(&self.arena),
            false,
        ))
    }

    /// Formats a handed-off segment that this producer reserved for the
    /// helper, pushes the formatted lines into the shared queue, and recycles
    /// the segment back to the pool.
    ///
    /// Runs on the *producer* instead of the drain (NanoLog pool exhaustion).
    /// The segment must be `helper_reserved` so the drain skips it, and the
    /// reservation is kept set for the whole call so the drain can never read
    /// the segment concurrently.
    fn format_helper_segment(&self, ring: Arc<RingBuffer>) {
        let lines = format_segment(
            &ring,
            self.timezone_offset,
            &self.calibration,
            &self.line_pattern,
        );
        self.push_lines(lines);
        // The drain skipped this segment while the reservation was set; it is
        // still skipped until the reset clears the flag below. Rewind the
        // control state (which also clears the reservation and `handed_off`,
        // so the drain can no longer decide to retire/recycle it) and park the
        // segment in the pool. The scratch head/tail caches are reset while
        // the ring is absent from the drain's local list, so no read overlaps.
        ring.reset_for_reuse();
        self.recycle(ring);
    }

    /// Pushes a batch of formatted lines into the shared queue (helper path).
    pub(crate) fn push_lines(&self, lines: Vec<(Vec<u8>, Level)>) {
        if !lines.is_empty() {
            self.lines.lock().unwrap().extend(lines);
        }
    }

    /// Takes everything currently in the shared line queue. Called by the
    /// drain once per poll pass while a segmented policy is active.
    pub(crate) fn take_lines(&self) -> Vec<(Vec<u8>, Level)> {
        let mut q = self.lines.lock().unwrap();
        std::mem::take(&mut *q).into_iter().collect()
    }

    /// Recycles a fully drained pool segment from the drain side. Guards with
    /// `mark_recycled` so a segment that was already returned (or re-taken and
    /// reset) is never pushed twice.
    pub(crate) fn drain_recycle(&self, ring: Arc<RingBuffer>) -> bool {
        if ring.recyclable() && ring.mark_recycled() {
            self.recycle(ring);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::DEFAULT_LINE_PATTERN;

    /// A tiny pool so `take_pooled` has to block (and the helper path can be
    /// exercised) without allocating much.
    fn test_segments(pool_size: usize) -> Segments {
        let pattern =
            Template::parse(DEFAULT_LINE_PATTERN).expect("invariant: default pattern is valid");
        Segments::init_with_pool_size(
            Backpressure::NanoLog,
            4096,
            pool_size,
            0,
            crate::timestamp::Calibration {
                counter_to_ns: 1.0,
                counter_base: 0,
                wall_base_ns: 0,
            },
            pattern,
        )
    }

    #[test]
    fn pool_preallocates_its_whole_budget() {
        let segs = test_segments(3);
        let count = segs.spares.lock().unwrap().len();
        assert_eq!(count, 3, "the pool must preallocate exactly 3 segments");
    }

    #[test]
    fn first_ring_nanolog_claims_pooled_segment() {
        let segs = test_segments(2);
        let mut helper = None;
        let ring = segs
            .first_ring(&mut helper, None)
            .expect("pool has a spare");
        assert!(ring.recyclable(), "NanoLog segments are pool-backed");
        assert_eq!(segs.spares.lock().unwrap().len(), 1);
    }

    #[test]
    fn recycle_returns_segment_to_pool_and_notifies() {
        let segs = test_segments(1);
        let ring = segs.try_take_spare().expect("pool has a spare");
        // Drain the segment (empty), then return it: the pool must accept it.
        ring.mark_handed_off();
        let recycled = segs.drain_recycle(ring);
        assert!(recycled);
        assert_eq!(
            segs.spares.lock().unwrap().len(),
            1,
            "recycle restores the spare"
        );
    }

    #[test]
    fn quill_segments_are_not_recyclable() {
        let pattern = Template::parse(DEFAULT_LINE_PATTERN).unwrap();
        let segs = Segments::init_with_pool_size(
            Backpressure::Quill,
            4096,
            0,
            0,
            crate::timestamp::Calibration {
                counter_to_ns: 1.0,
                counter_base: 0,
                wall_base_ns: 0,
            },
            pattern,
        );
        assert!(
            !segs.alloc_fresh().recyclable(),
            "Quill segments never return to a pool"
        );
        assert!(
            segs.spares.lock().unwrap().is_empty(),
            "Quill has no bounded pool"
        );
    }
}
