//! Segmented (`Backpressure::Quill`) handoff with a raw sink: handed-off
//! segments are drained as raw records, and on pool exhaustion the
//! producer-side helper copies wire bytes into the shared queue instead of
//! rendering them — either way each record is delivered exactly once and no
//! line is ever formatted.
//!
//! The small pool (8 MiB budget / 4 MiB segments = 2 segments) makes
//! exhaustion realistic under concurrent load, so the helper path is
//! exercised. Under a `backend-*` feature Quill degrades to `Block`, which
//! preserves the exact-count property the assertions rely on.

use ticklog::{Backpressure, InMemorySink, Level, configure, info};

const THREADS: usize = 4;
/// Enough to fill and hand off several 4 MiB segments per thread.
#[cfg(not(miri))]
const PER_THREAD: usize = 100_000;
#[cfg(miri)]
const PER_THREAD: usize = 100;

#[test]
fn quill_handoff_delivers_every_raw_record_exactly_once() {
    let sink = InMemorySink::new();
    let handle = sink.handle();

    let guard = configure! {
        sink: sink,
        max_level: Level::Trace,
        backpressure: Backpressure::Quill,
        ring_capacity: 4 * 1024 * 1024,
    }
    .expect("first configure in a fresh process must succeed");

    let threads: Vec<_> = (0..THREADS)
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

    drop(guard);

    assert_eq!(
        handle.lines_len(),
        0,
        "helper-delivered payloads must arrive raw too — nothing was rendered"
    );
    assert_eq!(
        handle.records_len(),
        THREADS * PER_THREAD,
        "Quill must not drop a record; lost {}",
        THREADS * PER_THREAD - handle.records_len()
    );
}
