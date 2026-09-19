//! Adaptive backoff for spin-waits on a latency-critical path.
//!
//! [`Backpressure::Block`](crate::Backpressure) and the fifo backends wait
//! for the drain to free space. A bare [`core::hint::spin_loop`] burns a
//! whole core for as long as the ring stays full, which degrades the drain's
//! own progress on a shared core. We build the wait on the [`spin`] crate's
//! relax strategies instead: pause hints for the first few iterations, then
//! yield the timeslice to the scheduler. Yielding is also a real scheduling
//! point, so the drain thread (and Miri's round-robin scheduler) can run
//! while a producer is parked on a full ring.

/// Number of pause-hint iterations before the wait starts yielding.
pub(crate) const SPIN_LOOPS: u32 = 64;

/// Cheap adaptive backoff scoped to a single wait.
pub(crate) struct Backoff {
    counter: u32,
}

impl Backoff {
    /// Start a fresh wait.
    #[inline]
    pub(crate) fn new() -> Self {
        Self { counter: 0 }
    }

    /// Advance the wait by one step: pause while the wait is short, yield the
    /// timeslice once it turns long. Call repeatedly until the wait succeeds.
    #[inline]
    pub(crate) fn wait(&mut self) {
        use spin::relax::RelaxStrategy;

        // `spin::relax::Spin` is the pause-hint strategy; `spin::relax::Yield`
        // (std) hands the timeslice back to the scheduler.
        if self.counter < SPIN_LOOPS {
            spin::relax::Spin::relax();
        } else {
            spin::relax::Yield::relax();
        }
        self.counter = self.counter.wrapping_add(1);
    }
}
