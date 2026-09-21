//! Gungraun version of the ticklog call-site benchmark.
//!
//! This is a *different* measurement methodology from the classic parallel
//! harness in `cross-lang-bench/rust/ticklog`. Gungraun measures one-shot,
//! toollevel event counts (instructions / L1 / LL / RAM hits / estimated
//! cycles) under Valgrind, single-threaded, instead of wall-clock throughput
//! and RDTSC latencies. It can only be executed on Linux/WSL where Valgrind
//! is available.
//!
//! Mirrors the classic harness workloads: `single_int`, `mixed`, `string`,
//! each running one batch of `BATCH` `info!` calls against the custom ring
//! with the Drop policy (runs only in WSL/Linux).

use std::hint::black_box;
use std::io;
use std::sync::Once;

extern crate gungraun;
use gungraun::prelude::*;
use ticklog::{info, Backpressure, Level, LogSink};

/// Number of log calls per measured batch (matches the classic harness).
const BATCH: usize = 1000;

// Bridges for the `info!` family, fixed to the Drop policy (mirrors the
// `policy-drop` feature of the classic harness).
#[allow(non_local_definitions)]
macro_rules! __ticklog_backpressure {
    () => {
        ticklog::Backpressure::Drop
    };
}
#[allow(non_local_definitions)]
macro_rules! __ticklog_max_level {
    () => {
        ticklog::Level::Trace
    };
}

/// Discard-only sink, like the classic harness default.
struct NullSink;

impl LogSink for NullSink {
    fn accept(&mut self, _line: &[u8], _level: Level) -> io::Result<()> {
        Ok(())
    }
}

/// Configure the ticklog runtime exactly once per process. Runs inside the
/// gungraun setup hook, i.e. before the instrumented region, so the ring
/// allocation and drain thread are not attributed to the measured metrics.
fn setup_config(seed: u64) -> u64 {
    static CONFIGURED: Once = Once::new();
    CONFIGURED.call_once(|| {
        let guard = ticklog::__private::__configure_rt(
            Box::new(NullSink),
            0i32,
            None,
            ticklog::__private::DEFAULT_RING_SIZE,
            "",
            Backpressure::Drop,
        )
        .expect("ticklog build");
        std::mem::forget(guard);
        let _ = ticklog::warm_up();
    });
    seed
}

/// `single_int`: one `u64` argument whose value depends on `call_index`.
fn bench_body_single_int(seed: u64) -> u64 {
    for i in 0..BATCH {
        let v = seed.wrapping_mul(BATCH as u64) + i as u64;
        info!("x={}", black_box(v));
    }
    seed
}

/// `mixed`: three differently-typed arguments.
fn bench_body_mixed(seed: u64) -> u64 {
    for _ in 0..BATCH {
        info!(
            "{} {} {}",
            black_box(42u64),
            black_box(3.14159f64),
            black_box("hello world"),
        );
    }
    seed
}

/// `string`: one `&str` argument.
fn bench_body_string(seed: u64) -> u64 {
    for _ in 0..BATCH {
        info!("{}", black_box("hello world"));
    }
    seed
}

#[library_benchmark]
#[bench::seed_0(args = (0u64), setup = setup_config)]
#[bench::seed_1(args = (1u64), setup = setup_config)]
fn bench_single_int(seed: u64) -> u64 {
    black_box(bench_body_single_int(seed))
}

#[library_benchmark]
#[bench::seed_0(args = (0u64), setup = setup_config)]
#[bench::seed_1(args = (1u64), setup = setup_config)]
fn bench_mixed(seed: u64) -> u64 {
    black_box(bench_body_mixed(seed))
}

#[library_benchmark]
#[bench::seed_0(args = (0u64), setup = setup_config)]
#[bench::seed_1(args = (1u64), setup = setup_config)]
fn bench_string(seed: u64) -> u64 {
    black_box(bench_body_string(seed))
}

library_benchmark_group!(
    name = ticklog_ring_group,
    benchmarks = [bench_single_int, bench_mixed, bench_string]
);

main!(library_benchmark_groups = [ticklog_ring_group]);