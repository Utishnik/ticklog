//! `warm_up` front-loads a thread's one-time costs so the first log after it
//! allocates nothing on the caller.
//!
//! Two guarantees are checked under one `configure!` (the registry is claimed
//! once per process):
//!
//! 1. `warm_up` is idempotent -- a second call on an already-warmed thread is a
//!    no-op that still returns `Ok`.
//! 2. The first `info!` after `warm_up` performs zero heap allocations on the
//!    calling thread. `warm_up` has already allocated the ring and scratch
//!    buffer, initialized the thread-local slot, and registered its destructor,
//!    so the hot path only reads the clock, encodes into the existing scratch,
//!    and copies into the ring.
//!
//! A process-wide counting allocator makes producer-thread allocations
//! observable. It counts only while a thread-local flag is set, and only on the
//! thread that sets it, so the drain thread's formatting allocations (a
//! different thread) are never counted. The flag and counter use const-
//! initialized thread-locals, which need no allocation to access -- the counting
//! path itself must not allocate, or it would recurse into the allocator.
//!
//! The counting allocator cannot run under Miri: Miri models `System` (Windows
//! `HeapFree`) reading its own header through the user pointer's tag, which
//! trips Stacked/Tree Borrows for any allocation made via the wrapper. Under
//! Miri the test reduces to the functional half (configure, idempotent
//! `warm_up`, one log call) without the allocation-counting assertion.

use std::io;

use ticklog::{Level, WriterSink, info, warm_up};

#[cfg(not(miri))]
mod counting {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::cell::Cell;

    thread_local! {
        /// Heap allocations observed on this thread while counting is enabled.
        static ALLOCATIONS: Cell<u64> = const { Cell::new(0) };
        /// Whether this thread is currently counting its allocations.
        static COUNTING: Cell<bool> = const { Cell::new(false) };
    }

    /// Records one allocation when the current thread has counting enabled. Uses
    /// only const-initialized thread-locals, so it never allocates and never
    /// re-enters the allocator.
    fn note_alloc() {
        COUNTING.with(|on| {
            if on.get() {
                ALLOCATIONS.with(|n| n.set(n.get() + 1));
            }
        });
    }

    /// Delegates to the system allocator, tallying each allocation on the
    /// counting thread.
    pub(super) struct CountingAlloc;

    unsafe impl GlobalAlloc for CountingAlloc {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            note_alloc();
            unsafe { System.alloc(layout) }
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            unsafe { System.dealloc(ptr, layout) }
        }

        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            note_alloc();
            unsafe { System.alloc_zeroed(layout) }
        }

        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
            note_alloc();
            unsafe { System.realloc(ptr, layout, new_size) }
        }
    }

    /// Counts this thread's allocations; used from the configured harness below.
    pub(super) fn start_counting() {
        COUNTING.with(|on| on.set(true));
    }

    pub(super) fn stop_counting() {
        COUNTING.with(|on| on.set(false));
    }

    pub(super) fn alloc_count() -> u64 {
        ALLOCATIONS.with(|n| n.get())
    }

    /// Prime the counting thread-locals so their first access is outside the
    /// measured window (const init means this does not allocate anyway).
    pub(super) fn zero_count() {
        COUNTING.with(|on| on.set(false));
        ALLOCATIONS.with(|n| n.set(0));
    }
}

#[cfg(not(miri))]
#[global_allocator]
static ALLOCATOR: counting::CountingAlloc = counting::CountingAlloc;

fn configure_and_warm() -> ticklog::Guard {
    // configure! and the sink live outside the measured window; a null sink keeps
    // the drain's own work irrelevant (it runs on another thread regardless).
    let guard = ticklog::configure! {
        sink: WriterSink::new(io::sink()),
        max_level: Level::Trace,
    }
    .expect("first configure in a fresh process must succeed");

    // First call allocates the ring, scratch, and thread-local state.
    warm_up().expect("warm_up after configure must succeed");
    // Second call is a no-op on an already-warmed thread.
    warm_up().expect("warm_up must be idempotent");

    guard
}

#[cfg(not(miri))]
#[test]
fn warm_up_is_idempotent_and_first_log_is_allocation_free() {
    let guard = configure_and_warm();

    counting::zero_count();

    // Measure exactly one log call on the warmed thread.
    counting::start_counting();
    info!("warm {}", 1u64);
    counting::stop_counting();

    let allocations = counting::alloc_count();
    assert_eq!(
        allocations, 0,
        "first log after warm_up allocated {allocations} time(s) on the caller"
    );

    drop(guard);
}

#[cfg(miri)]
#[test]
fn warm_up_is_idempotent_and_logs_under_miri() {
    let _guard = configure_and_warm();
    info!("warm {}", 1u64);
}
