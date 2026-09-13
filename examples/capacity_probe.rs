//! Sustained end-to-end logging throughput at a given per-thread ring
//! capacity, for one ring backend.
//!
//! Measures the drain-coupled pipeline: `T` producer threads log `N` records
//! each under [`Backpressure::Block`] (nothing is dropped; a full ring only
//! throttles the producer), while the drain formats to a null writer. The
//! reported `ns_per_log` is wall-clock time divided by total records, i.e.
//! the sustained per-record pipeline cost with all threads running.
//!
//! This is deliberately a **sustained-load** metric, not the isolated hot-path
//! latency that [`bench single_record`](../../benches/single_record.rs)
//! measures (on this machine ~16 ns). At a small ring the producer stalls on
//! the full buffer; at a large ring that absorbs the whole burst it never
//! does. `scripts/capacity_sweep.sh` uses `N` large enough that even the
//! biggest ring throttles, so every cell measures the same regime and only
//! buffering differs.
//!
//! Run it per backend and capacity; the sweep script
//! `scripts/capacity_sweep.sh` does the full matrix:
//!
//! ```text
//! cargo run --release --example capacity_probe -- 262144 16 300000        # custom ring, 256 KiB
//! cargo run --release --example capacity_probe --features backend-ringbuffer -- 262144 16 300000
//! ```
//!
//! Output is one CSV line: `backend,capacity,threads,records,elapsed_ms,ns_per_log,recs_per_sec`.

use std::hint;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use ticklog::{Backpressure, Level, info, warm_up};

#[cfg(feature = "backend-ringbuffer")]
const BACKEND: &str = "ringbuffer";
#[cfg(not(feature = "backend-ringbuffer"))]
const BACKEND: &str = "custom";

fn parse<T: std::str::FromStr>(args: &[String], i: usize, default: T) -> T {
    args.get(i).and_then(|s| s.parse().ok()).unwrap_or(default)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let capacity: usize = parse(&args, 0, 1_048_576);
    let threads: usize = parse(&args, 1, 16);
    let per_thread: usize = parse(&args, 2, 300_000);

    // `ring_capacity` may be a runtime value (it is consumed only at this
    // configure! site); `backpressure` must be a path constant because the
    // logging macros re-expand it at every log call site.
    let guard = ticklog::configure! {
        sink: ticklog::WriterSink::new(std::io::sink()),
        max_level: Level::Info,
        backpressure: Backpressure::Block,
        ring_capacity: capacity,
    }
    .expect("ticklog configures once per process");

    // Snowball gate: all producers spin until the main thread releases them,
    // so the clock starts at the same instant for every thread.
    let gate = Arc::new(AtomicBool::new(false));
    let mut handles = Vec::with_capacity(threads);
    for t in 0..threads {
        let gate = Arc::clone(&gate);
        handles.push(std::thread::Builder::new()
            .name(format!("probe-{t}"))
            .spawn(move || {
                // Move the one-time thread-local allocation off the timed path.
                warm_up().ok();
                while !gate.load(Ordering::Acquire) {
                    hint::spin_loop();
                }
                let mut i = 0u64;
                for _ in 0..per_thread {
                    i = i.wrapping_add(1);
                    info!("x={}", i);
                }
            })
            .expect("spawn producer"));
    }

    let t0 = Instant::now();
    gate.store(true, Ordering::Release);
    for h in handles {
        h.join().expect("producer panicked");
    }
    let elapsed_ns = t0.elapsed().as_nanos() as f64;

    drop(guard); // flush remaining records and join the drain

    let total = (threads as u64) * (per_thread as u64);
    let ns_per_log = elapsed_ns / total as f64;
    let recs_per_sec = (total as f64) / (elapsed_ns / 1e9);
    println!(
        "{BACKEND},{capacity},{threads},{total},{:.3},{:.1},{:.0}",
        elapsed_ns / 1e6,
        ns_per_log,
        recs_per_sec,
    );
}