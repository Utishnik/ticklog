//! `InMemorySink` as the root sink: the raw fast path captures wire records
//! and never renders a line.
//!
//! `lines_len() == 0` after a run that produced records is the direct proof
//! that pattern rendering (and the timestamp/thread rendering around it) was
//! skipped: the sink would otherwise have captured the formatted lines too.

use ticklog::{InMemorySink, Level, configure, debug, error, info, trace, warn};

const THREADS: usize = 4;
#[cfg(not(miri))]
const PER_THREAD: usize = 25;
#[cfg(miri)]
const PER_THREAD: usize = 3;

#[test]
fn in_memory_sink_captures_raw_records_end_to_end() {
    let sink = InMemorySink::new();
    // Clone the handle before `configure!` moves the sink.
    let handle = sink.handle();

    let guard = configure! {
        sink: sink,
        max_level: Level::Trace,
    }
    .expect("first configure in a fresh process must succeed");

    // Single-threaded prefix: a fixed sequence with known levels and
    // timestamps from one ring, so both are checkable exactly.
    info!("a {}", 1u64);
    warn!("b");
    error!("c");
    trace!("d {}", 2.5f64);
    debug!("e");

    // Concurrent phase: the counts must be exact (the raw path loses
    // nothing on the default Drop policy while the drain keeps up; the
    // exactness under real backpressure is `fast_sink_memory_block`).
    let threads: Vec<_> = (0..THREADS)
        .map(|t| {
            std::thread::spawn(move || {
                for i in 0..PER_THREAD {
                    info!("t{} {}", t as u64, i as u64);
                }
            })
        })
        .collect();
    for t in threads {
        t.join().expect("producer threads must not panic");
    }

    // Drop flushes and joins the drain, so every record is captured by now.
    drop(guard);

    assert_eq!(
        handle.lines_len(),
        0,
        "the raw fast path must never render a line"
    );
    let records = handle.records();
    assert_eq!(records.len(), 5 + THREADS * PER_THREAD);

    // Levels arrive exactly as logged, in per-ring order for the prefix.
    let levels = handle.levels();
    assert_eq!(
        &levels[..5],
        [
            Level::Info,
            Level::Warn,
            Level::Error,
            Level::Trace,
            Level::Debug
        ]
    );
    assert!(
        levels[5..].iter().all(|l| *l == Level::Info),
        "the concurrent phase only logged info!"
    );

    // Every captured frame is self-consistent: the header's total_size
    // frames exactly the bytes handed over (16-byte header + site + count
    // + arguments is the minimum record).
    for r in &records {
        assert_eq!(r.total_size() as usize, r.as_bytes().len());
        assert!(r.total_size() >= 25, "record smaller than a bare frame");
    }

    // The single-threaded prefix comes from one ring on one thread, so its
    // producer timestamps never go backwards.
    let stamps: Vec<u64> = records[..5].iter().map(|r| r.timestamp_ticks()).collect();
    for w in stamps.windows(2) {
        assert!(w[0] <= w[1], "timestamps must be monotonic: {stamps:?}");
    }
}
