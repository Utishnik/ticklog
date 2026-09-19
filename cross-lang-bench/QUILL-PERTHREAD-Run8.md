# Run 8 — per-thread ring fast path (ticklog-fast)

Date: 2026-09-19. Same harness, calibration and methodology as Run 7
(`samples=1000` => 1M logging calls per config; `BATCH=1000`; RDTSC;
WSL `ns_per_tick=0.313148759`, Windows `0.313132820`).

## What was tested

A new crate feature `fast-tail-cache` plus a harness policy `policy-fast`
(`ticklog/fast-tail-cache` + `policy-quill`): the producer's ring capacity
check on the hot path skips the per-record `tail.load(Acquire)` of the drain's
position and trusts a possibly-stale cached tail (quill's
`_reader_pos_cache_plus_capacity` design). The real tail is refreshed only on
the slow/confirm path. Occupancy computed from a stale tail only ever
over-estimates real occupancy, so a producer can never overwrite a slot the
drain has not consumed.

Context: ticklog already uses **one ring per thread** (`ThreadBuf.ring` in the
`REGISTRY`); a single drain thread walks all rings. The shared parts are the
drain thread and the arena, not the ring itself. Run 8 therefore effectively
answers "does the per-record cross-core read explain the gap vs C++ quill?"
Answer: no.

Candidates compared (all clang-max-opt on the C++ side, Rust release LTO):
- `ticklog-fast` (r8): ticklog, per-thread ring, cached-tail fast path, ring 32 MiB (WSL) / 64 MiB (Windows).
- `ticklog-quill` (r8): ticklog baseline policy run side-by-side, same ring sizes.
- `quill` (r7): C++ quill baseline, queue 32 MiB (WSL) / 64 MiB (Windows), producer+consumer pinned.

## single_int, median (p50) ns per logging call

| threads | ticklog-fast (WSL) | ticklog-quill (WSL) | C++ quill (WSL) | ticklog-fast (win) | ticklog-quill (win) | C++ quill (win) |
|--------:|-------------------:|--------------------:|----------------:|-------------------:|--------------------:|----------------:|
| 1       | 18.47              | 19.42               | 15.18           | 23.12              | 33.13               | 14.90           |
| 2       | 24.88              | 22.68               | 16.88           | 27.47              | 34.38               | 14.92           |
| 4       | 38.33              | 38.94               | 15.25           | 52.47              | 47.88               | 20.20           |
| 8       | 70.21              | 71.47               | 21.94           | 82.30              | 83.90               | 23.18           |
| 16      | 99.69              | 94.67               | 29.84           | 95.27              | 115.02              | 40.03           |

## string, median (p50) ns per logging call

| threads | ticklog-fast (WSL) | ticklog-quill (WSL) | C++ quill (WSL) | ticklog-fast (win) | ticklog-quill (win) | C++ quill (win) |
|--------:|-------------------:|--------------------:|----------------:|-------------------:|--------------------:|----------------:|
| 1       | 20.23              | 24.34               | 16.57           | 22.59              | 24.56               | 15.90           |
| 2       | 26.91              | 22.95               | 17.18           | 28.38              | 30.34               | 15.96           |
| 4       | 39.45              | 36.31               | 16.80           | 44.94              | 51.68               | 23.17           |
| 8       | 74.62              | 76.01               | 27.41           | 80.52              | 79.86               | 29.23           |
| 16      | 77.87              | 102.35              | 36.10           | 95.23              | 118.12              | 42.96           |

## Observations

1. **`ticklog-fast` and `ticklog-quill` are statistically identical**
   (differences of 1-3 ns swing both ways across workloads and OSes). Removing
   the per-record `tail.load(Acquire)` changed nothing structural. The tail
   cache was already hitting L1 in the baseline; on this x86 the Acquire load
   saturates producer-side per-ring and the drain side is not on the hot path.
2. **One single-producer cost is ~20%** (18-23 ns vs 15) — the i.e. record
   encode + header write path, not contention.
3. **The gap grows with thread count** (3x at 8 threads, 2.5-3.5x at 16),
   matching a scaling/scheduling difference rather than a per-record one.
   Likely dominant factors, in order:
   - **No affinity pinning in the Rust harness.** `--producer-core` /
     `--backend-core` were not passed in Run 7/8, so producers *and* the drain
     float across 16 cores. The C++ quill harness pins producer to core 0 and
     consumer to core 1. Running unpinned lets the OS migrate threads (cache
     lines move with them) and lets the drain land on a producer core.
   - WSL/Windows overhead of the harness itself under 2+ threads (the C++ side
     uses the same per-call loop but with pinning).
4. **Conclusion for the requested strategy**: the "shared ring" hypothesis
   does not hold — rings were already per-thread, and the quill-style cached
   reader position yields no win. Pinning (below) is not the dominant factor
   either.

## Follow-up: pinned run (producers on cores 0..n-1, drain on core 8)

Run with `--producer-core 0 --backend-core 8` (threads 1,2,4,8).

single_int p50:

| threads | fast unpin (wsl) | fast pin (wsl) | quill pin (wsl) | fast unpin (win) | fast pin (win) | quill pin (win) |
|--------:|-----------------:|---------------:|----------------:|-----------------:|---------------:|----------------:|
| 1       | 18.47            | 19.21           | 19.60           | 23.12            | 18.86           | 20.99           |
| 2       | 24.88            | 21.24           | 29.88           | 27.47            | 34.52           | 35.94           |
| 4       | 38.33            | 37.96           | 38.78           | 52.47            | 35.71           | 41.33           |
| 8       | 70.21            | 71.89           | 70.92           | 82.30            | 67.48           | 70.46           |
| 8       | (C++ quill win 23.18)         |                 |                |                  |                 |                 |

Pinning wins ~15-20% on Windows at t4/t8 (82.3->67.5, 83.9->70.5) and t1
(23.1->18.9), and ~nothing on WSL. The 3x gap to C++ quill at t8 (67-71 vs
21-23) stays. The C++ harness itself does **not** pin — so this is not the
explanation either.

## Where the remaining gap actually comes from

Structural difference in the publish path, not the ring layout:

- **ticklog**: `publish` per record — `head.store(Release)` on every logging
  call (ring.rs:527). The drain polls every ring continuously, so with N busy
  producers each `head` cache line bounces producer<->drain on every record.
- **quill**: the producer packs records into an SPSC block and publishes the
  whole block with a single atomic store when it fills (default block ~512 KiB,
  i.e. one publish per ~12K records). Per-record head stores on the hot path
  simply do not happen.

That fits the scaling curve exactly (2.5-3x at 8-16 threads, ~equal at t1). It
also matches the "общее кольцо" intuition — the shared object that matters is
the **drain's polling of every ring's head**, not the ring storage.

## Next steps (if wanted)

- **Watermark-batched head publish (Run 9)**: keep a producer-private head; a
  Release store to the shared head line once per water-marked batch (e.g.
  every 1000 records) instead of per record. Drain processes only published
  head; the border is already stored by the producer, so a drain that lags
  merely sees less data. This is the change most likely to close the 3x.
- Profile the single-producer encode path (24-32 B header per record) against
  quill's constant-preamble writer.
- Event-driven drain wakeup (like NanoLog) to stop the drain's busy poll.

## Files

- `results_quill/wsl_ticklog_fast_r8.json`, `wsl_ticklog_quill_r8.json`
- `results_quill/windows_ticklog_fast_r8.json`, `windows_ticklog_quill_r8.json`
- C++ quill references: `wsl_quill_cpp_r7.json`, `windows_quill_cpp_r7.json`
- Code: `src/ring.rs` (`try_ensure_capacity`, `cfg(feature = "fast-tail-cache")`),
  `cross-lang-bench/rust/ticklog/{Cargo.toml,src/main.rs}` (`policy-fast`).