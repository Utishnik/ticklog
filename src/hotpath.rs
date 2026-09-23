//! Benchmark-only hot-path cycle profiler.
//!
//! Behind Cargo feature `hotpath-profiler`. Every producer log call records
//! rdtsc deltas per stage into a thread-local array; the benchmark harness
//! reads the counters back with [`take`] after its measurement loop. When the
//! feature is disabled every function is a no-op (or returns 0), so standard
//! builds pay nothing on the hot path.

use core::cell::Cell;

/// Total cycles inside `dispatch` (whole log-call entry point).
pub const L_TOTAL: usize = 0;
/// Cycles spent in the record timestamp read.
pub const L_TIMESTAMP: usize = 1;
/// Cycles spent reserving space (policy + ring reserve / handoff).
pub const L_RESERVE: usize = 2;
/// Cycles spent encoding the record (header + args).
pub const L_ASSEMBLE: usize = 3;
/// Cycles spent publishing the slot to the drain.
pub const L_PUBLISH: usize = 4;
/// Cycles spent confirming the drain tail on the cold confirm path
/// (cache-line miss cost plus backoff), a subset of [`L_RESERVE`].
pub const L_COLD_CONFIRM: usize = 5;
/// Number of records dispatched by this thread.
pub const C_CALLS: usize = 6;
/// Number of records dropped (reserve returned `None`).
pub const C_DROPS: usize = 7;

/// Total number of lanes (both cycle accumulators and plain counters).
pub const LANES: usize = 8;

/// Lane names for report columns.
pub const LANE_NAMES: [&str; LANES] = [
    "total",
    "timestamp",
    "reserve",
    "assemble",
    "publish",
    "cold_confirm",
    "calls",
    "drops",
];

/// Whether the profiler is compiled in.
pub const ENABLED: bool = cfg!(feature = "hotpath-profiler");

thread_local! {
    static TLS: Cell<[u64; LANES]> = const { Cell::new([0u64; LANES]) };
}

/// Reads the platform hardware counter (rdtsc / cntvct). Returns 0 when the
/// feature is disabled so callers can trim dead code.
#[inline(always)]
pub fn tick() -> u64 {
    if ENABLED {
        crate::timestamp::raw_timestamp()
    } else {
        0
    }
}

/// Accumulates `delta` cycles into lane `lane`.
#[inline]
pub fn add(lane: usize, delta: u64) {
    if !ENABLED {
        return;
    }
    TLS.with(|c| {
        let mut lanes = c.get();
        lanes[lane] = lanes[lane].wrapping_add(delta);
        c.set(lanes);
    });
}

/// Adds one to lane `lane`.
#[inline]
pub fn bump(lane: usize) {
    add(lane, 1);
}

/// Returns (and zeroes) this thread's accumulated counters.
pub fn take() -> [u64; LANES] {
    TLS.with(|c| c.replace([0u64; LANES]))
}
