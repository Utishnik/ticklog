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
#[cfg(not(ticklog_loom))]
pub(crate) const SPIN_LOOPS: u32 = 64;

/// Adaptive backoff scoped to a single wait.
pub(crate) struct Backoff {
    #[cfg(not(ticklog_loom))]
    counter: u32,
    /// Under model checking, counts `wait` steps so `try_ensure_capacity`
    /// can bound an otherwise-infinite Block spin (loom explodes its branch
    /// budget on paths where the consumer is delayed).
    #[cfg(ticklog_loom)]
    loom_spins: u32,
}

impl Backoff {
    /// Start a fresh wait.
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            #[cfg(not(ticklog_loom))]
            counter: 0,
            #[cfg(ticklog_loom)]
            loom_spins: 0,
        }
    }

    /// Advance the wait by one step: pause while the wait is short, yield the
    /// timeslice once it turns long. Call repeatedly until the wait succeeds.
    #[inline]
    pub(crate) fn wait(&mut self) {
        #[cfg(ticklog_loom)]
        {
            self.loom_spins = self.loom_spins.wrapping_add(1);
            // One yield per step: enough scheduling points for the consumer
            // model, without a pause-hint ramp the checker must also explore.
            std::thread::yield_now();
            return;
        }

        // `spin::relax::Spin` is the pause-hint strategy; `spin::relax::Yield`
        // (std) hands the timeslice back to the scheduler.
        #[cfg(not(ticklog_loom))]
        {
            use spin::relax::RelaxStrategy;
            if self.counter < SPIN_LOOPS {
                spin::relax::Spin::relax();
            } else {
                spin::relax::Yield::relax();
            }
            self.counter = self.counter.wrapping_add(1);
        }
    }

    /// How many times [`wait`](Self::wait) has been called (loom only).
    #[cfg(ticklog_loom)]
    #[inline]
    pub(crate) fn loom_spins(&self) -> u32 {
        self.loom_spins
    }
}
