//! `Backpressure::Block` with a raw sink: producers that outrun the ring
//! spin until the drain frees space, and every record must arrive exactly
//! once — the property that distinguishes Block from Drop, observed on the
//! raw capture path rather than on formatted lines.

use ticklog::{Backpressure, InMemorySink, Level, configure, info};

const THREADS: usize = 4;
/// Far beyond the 4 KiB ring, so every thread blocks repeatedly.
#[cfg(not(miri))]
const PER_THREAD: usize = 2_000;
#[cfg(miri)]
const PER_THREAD: usize = 50;

#[test]
fn block_backpressure_delivers_every_raw_record() {
    let sink = InMemorySink::new();
    let handle = sink.handle();

    let guard = configure! {
        sink: sink,
        max_level: Level::Trace,
        backpressure: Backpressure::Block,
        ring_capacity: 4 * 1024,
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
        "the raw fast path must never render a line"
    );
    assert_eq!(
        handle.records_len(),
        THREADS * PER_THREAD,
        "Block must not drop a record; lost {}",
        THREADS * PER_THREAD - handle.records_len()
    );
    assert!(
        handle.levels().iter().all(|l| *l == Level::Info),
        "only info! was logged"
    );
}
