//! Producer hot-path latency of a single `info!` call, isolated from
//! the drain by the crate's raw fast path: `ticklog::NullSink` opts into
//! `RawLogSink`, so the drain thread only validates records and advances
//! the tail — no decoding, no rendering, no sink work.

#[path = "common/affinity.rs"]
mod affinity;

use criterion::{Criterion, criterion_group};
use ticklog::{Backpressure, Level, NullSink, info};

fn bench_single_record(c: &mut Criterion) {
    affinity::pin_producer_from_env();

    let drain_affinity = affinity::drain_core_from_env().map(|c| vec![c]);
    let guard = ticklog::configure! {
        sink: NullSink,
        max_level: Level::Trace,
        backpressure: Backpressure::Block,
        drain_affinity: drain_affinity,
    }
    .expect("ticklog build");
    std::mem::forget(guard);

    c.bench_function("info_one_u64", |b| {
        b.iter(|| info!("x={}", criterion::black_box(1u64)));
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .significance_level(0.01)
        .noise_threshold(0.02)
        .sample_size(200);
    targets = bench_single_record
}

// Report provider: with the `hotpath/hotpath` feature this prints the
// per-function hot-path report on exit; without it the macro is a no-op.
#[hotpath::main]
fn main() {
    benches();
    // Drop-rate probe under the crate's low-overhead `hotpath-profiler`:
    // reserve-fails are what let a lagging drain make a producer bench look
    // fast while delivering nothing.
    if cfg!(feature = "hotpath-profiler") {
        use ticklog::__private::hotpath as hp;
        let p = hp::take();
        let calls = p[hp::C_CALLS];
        let drops = p[hp::C_DROPS];
        eprintln!(
            "PROF calls={calls} drops={drops} (drop rate {:.1}%)",
            100.0 * drops as f64 / (calls + drops).max(1) as f64
        );
        let total = p[hp::L_TOTAL].max(1);
        for (i, name) in hp::LANE_NAMES.iter().enumerate() {
            if *name == "calls" || *name == "drops" {
                continue;
            }
            eprintln!(
                "  lane {name:<12} {:>16} ticks  {:>6.1}%  {:>10.1} ticks/call",
                p[i],
                100.0 * p[i] as f64 / total as f64,
                p[i] as f64 / calls.max(1) as f64,
            );
        }
    }
}
