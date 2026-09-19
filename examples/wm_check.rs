//! End-to-end record-loss check for the `quick`-style `head` publish features
//! (`fast-tail-cache`, `watermark-head`). Logs `n` records with a small ring
//! and the Quill policy (forcing many segment handoffs), drops the guard for a
//! graceful drain + sink flush, and reports whether the sink file contains
//! exactly `n` lines.
//!
//! ```text
//! cargo run --release --example wm_check --features watermark-head -- out.txt 250000
//! ```

use std::fs::File;
use std::io::{BufRead, BufReader};

use ticklog::{Backpressure, FileSink, Level, info};

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().unwrap_or_else(|| "wm_check.out".to_string());
    let n: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(250_000);

    let guard = ticklog::configure! {
        sink: FileSink::truncate(&path).unwrap(),
        max_level: Level::Trace,
        backpressure: Backpressure::Quill,
        // 1 MiB ring: a 48-byte slot wraps ~21k records, forcing repeated
        // handoffs that exercise the watermark-flush path.
        ring_capacity: 1usize << 20,
    }
    .expect("ticklog builds once per process");

    for i in 0..n as u64 {
        info!("record {}", i);
    }

    // Graceful shutdown: drains remaining rings and flushes the file sink.
    drop(guard);

    let lines = BufReader::new(File::open(&path).unwrap()).lines().count();
    let delta = lines as isize - n as isize;
    println!("lines={} expected={} delta={}", lines, n, delta);
    if lines != n {
        std::process::exit(2);
    }
}
