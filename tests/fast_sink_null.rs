//! `NullSink` as the root sink: the drain's raw fast path performs zero work
//! per record and keeps consuming under blocking backpressure.
//!
//! A `NullSink` counts nothing, so the assertion here is liveness itself:
//! with a ring far smaller than the total volume, producers must block and
//! unblock repeatedly, which only happens while the drain thread is alive
//! and advancing `tail` through the raw path. A stalled raw drain would hang
//! the run instead of completing.

use ticklog::{Backpressure, Level, NullSink, configure, info};

/// Records per producer thread; four threads write far beyond the 8 KiB ring
/// below, so blocking is guaranteed.
#[cfg(not(miri))]
const PER_THREAD: usize = 5_000;
#[cfg(miri)]
const PER_THREAD: usize = 100;

#[test]
fn null_sink_drain_consumes_under_blocking_backpressure() {
    let guard = configure! {
        sink: NullSink::new(),
        max_level: Level::Trace,
        backpressure: Backpressure::Block,
        ring_capacity: 8 * 1024,
    }
    .expect("first configure in a fresh process must succeed");

    let threads: Vec<_> = (0..4)
        .map(|t| {
            std::thread::spawn(move || {
                for i in 0..PER_THREAD {
                    info!("thread {} seq {}", t as u64, i as u64);
                }
            })
        })
        .collect();
    for t in threads {
        t.join().expect("producer threads must not panic");
    }

    // Guard::drop joins the drain: reaching this line means every blocked
    // producer was released by an advancing tail, i.e. the raw drain
    // consumed every record the producers published.
    drop(guard);
}
