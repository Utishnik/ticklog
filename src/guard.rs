//! Owns the drain thread. Dropping the [`Guard`] shuts down the drain and
//! waits for it to complete its final poll cycle.

use std::sync::Arc;

use crate::sync::{AtomicBool, Ordering};
use crate::thread_buf::REGISTRY;

/// Drain-thread handle type: `std` outside loom, `loom` under
/// `RUSTFLAGS="--cfg ticklog_loom"` so `Guard::drop`'s join is model-checked.
#[cfg(not(ticklog_loom))]
type DrainHandle = std::thread::JoinHandle<()>;
#[cfg(ticklog_loom)]
type DrainHandle = loom::thread::JoinHandle<()>;

/// A running logger. Keep it alive for as long as you want to log.
///
/// Returned by [`configure!`][crate::configure!]. When the guard is dropped it marks every
/// ring dead, signals the drain to exit, and joins the drain thread.
pub struct Guard {
    drain_thread: Option<DrainHandle>,
    shutdown: Arc<AtomicBool>,
}

impl Guard {
    /// Creates a new guard from a running drain thread and a shared shutdown
    /// flag.
    ///
    /// `shutdown` must be the same `Arc<AtomicBool>` that the drain thread
    /// reads at the top of its poll loop.
    pub(crate) fn new(drain_thread: DrainHandle, shutdown: Arc<AtomicBool>) -> Self {
        Self {
            drain_thread: Some(drain_thread),
            shutdown,
        }
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        // Mark every registered ring dead before the drain exits. Under
        // `Backpressure::Block` a producer whose ring fills checks `live` in
        // its spin path and bails when false, preventing a hang after the
        // drain is gone. Under `Drop` the ring fills and records are
        // discarded silently.
        if let Some(registry) = REGISTRY.get()
            && let Ok(rings) = registry.lock()
        {
            for ring in rings.iter() {
                ring.set_dead();
            }
        }

        // Signal the drain to exit at the top of its next poll cycle.
        // Release pairs with the drain's Acquire load of `shutdown`,
        // guaranteeing the store is visible before the drain checks it.
        self.shutdown.store(true, Ordering::Release);

        // Wait for the drain to complete its final poll cycle and exit.
        if let Some(handle) = self.drain_thread.take()
            && let Err(e) = handle.join()
        {
            // The drain thread panicked. Try to extract a message from
            // the panic payload and write it to stderr.
            if let Some(msg) = e.downcast_ref::<&str>() {
                eprintln!("ticklog: drain thread panicked: {}", msg);
            } else if let Some(msg) = e.downcast_ref::<String>() {
                eprintln!("ticklog: drain thread panicked: {}", msg);
            } else {
                eprintln!("ticklog: drain thread panicked");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::AtomicBool;
    use std::sync::Arc;
    #[cfg(not(ticklog_loom))]
    use std::thread;

    #[test]
    fn new_wraps_handle_and_shutdown() {
        let shutdown = Arc::new(AtomicBool::new(false));
        #[cfg(not(ticklog_loom))]
        let handle = thread::spawn(|| {});
        #[cfg(ticklog_loom)]
        let handle = loom::thread::spawn(|| {});
        let guard = Guard::new(handle, Arc::clone(&shutdown));
        assert!(guard.drain_thread.is_some());
        assert!(!guard.shutdown.load(Ordering::Relaxed));
    }

    #[test]
    fn drop_sets_shutdown_flag() {
        let shutdown = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&shutdown);
        #[cfg(not(ticklog_loom))]
        let handle = thread::spawn(move || {
            // Busy-wait until shutdown is signaled, then exit. The yield is a
            // real scheduling point so Miri's round-robin scheduler can run the
            // thread that sets the flag.
            while !flag.load(Ordering::Acquire) {
                std::hint::spin_loop();
                std::thread::yield_now();
            }
        });
        #[cfg(ticklog_loom)]
        let handle = loom::thread::spawn(move || {
            while !flag.load(Ordering::Acquire) {
                loom::thread::yield_now();
            }
        });

        let guard = Guard::new(handle, Arc::clone(&shutdown));
        assert!(!shutdown.load(Ordering::Relaxed));
        drop(guard);
        assert!(shutdown.load(Ordering::Relaxed));
    }

    #[test]
    fn drop_joins_drain_thread() {
        let shutdown = Arc::new(AtomicBool::new(false));
        let exited = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&shutdown);
        let done = Arc::clone(&exited);

        #[cfg(not(ticklog_loom))]
        let handle = thread::spawn(move || {
            while !flag.load(Ordering::Acquire) {
                std::hint::spin_loop();
                std::thread::yield_now();
            }
            done.store(true, Ordering::Release);
        });
        #[cfg(ticklog_loom)]
        let handle = loom::thread::spawn(move || {
            while !flag.load(Ordering::Acquire) {
                loom::thread::yield_now();
            }
            done.store(true, Ordering::Release);
        });

        let guard = Guard::new(handle, Arc::clone(&shutdown));
        drop(guard);
        // After Guard::drop returns, the thread must have exited.
        assert!(exited.load(Ordering::Relaxed));
    }

    #[test]
    fn guard_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Guard>();
    }
}
