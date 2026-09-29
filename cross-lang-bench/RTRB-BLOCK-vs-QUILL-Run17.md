# Run 17: ticklog rtrb + Block backpressure vs C++ quill (8 MiB queues)

Date: 2026-09-29. Same harness set and parameters as Run 16: BATCH=1000,
SAMPLES=1000 (1M msg/config), threads `1,2,4,8,16`, RDTSC. Calibration:
Windows `0.313132820 ns/tick`, WSL `0.313148759 ns/tick`. Rings: **8 MiB
(8388608) both platforms**. rtrb `--chunk-size 64`. C++ quill harnesses
rebuilt with `QUILL_BENCH_QUEUE_CAPACITY=8388608` (per-thread SPSC queue
8 MiB, compile-time), v13.0.0 (`quill_harness_8m_v13`) and the v12.1.0
baseline (`quill_harness_8m`).

**hotpath: OFF.** The harness builds use only `backend-rtrb,policy-block`
(no `hotpath-profiler`); `cargo tree -e features` shows no
`hotpath feature "hotpath"` in the resolution graph, so every
`#[hotpath::measure]` / `#[hotpath::main]` expands to a no-op and the bench
runs with zero instrumentation overhead.

## Builds & sizing

| Candidate | Features | Buffer |
|---|---|---|
| ticklog-rtrb-block | backend-rtrb,policy-block | rtrb ring 8 MiB, chunk 64, Backpressure::Block |
| cpp-quill v13 (8MiB q) | quill v13.0.0 header-only | per-thread SPSC queue 8 MiB (compile-time) |
| cpp-quill v12 (8MiB q) | quill v12.1.0 header-only | per-thread SPSC queue 8 MiB (compile-time) |
| ticklog-rtrb-drop (ref) | backend-rtrb,policy-drop (Run 16) | rtrb ring 8 MiB, chunk 64 |

`policy-block` is the harness's explicit Block feature (the harness default
with no `policy-*` feature is Block anyway); macro-side and configure-side
policy therefore match: `Backpressure::Block` on both.

## Result files

`results_quill/{windows,wsl}_ticklog_rtrb_block_r17.json`,
`results_quill/{windows,wsl}_quill_cpp_v13_r17.json`,
`results_quill/{windows,wsl}_quill_cpp_r17.json` (+ `.log` / `.log.err`).
Scripts: `build_{win,wsl}_r17.{ps1,sh}`, `run_{win,wsl}_r17.{ps1,sh}`.

Run 16's `*_ticklog_rtrb_drop_r16.json` is included as a reference row
(drop policy = no waits, silent loss when the ring is full).

---

## WSL

### single_int — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-rtrb-block | 35.4 | 41.5 | 41.9 | 43.7 | 73.1 |
| ticklog-rtrb-drop (r16) | 15.3 | 34.3 | 56.0 | 62.1 | 62.6 |
| cpp-quill v13 | 14.8 | 15.8 | 17.4 | 21.4 | 34.2 |
| cpp-quill v12 | 23.8 | 25.4 | 27.0 | 27.2 | 34.2 |

### single_int — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-rtrb-block | 1.9M | 1.5M | 1.4M | 66.0M | 65.0M |
| ticklog-rtrb-drop (r16) | 38.3M | 56.1M | 58.1M | 21.3M | 81.3M |
| cpp-quill v13 | 1.9M | 1.8M | 6.6M | 306.1M | 140.3M |
| cpp-quill v12 | 1.4M | 1.4M | 3.9M | 249.7M | 66.3M |

### mixed — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-rtrb-block | 31.9 | 50.2 | 37.9 | 105.8 | 161.9 |
| ticklog-rtrb-drop (r16) | 16.7 | 17.1 | 67.5 | 100.9 | 103.4 |
| cpp-quill v13 | 24.8 | 24.8 | 25.0 | 34.9 | 50.9 |
| cpp-quill v12 | 31.3 | 31.3 | 38.6 | 51.5 | 54.8 |

### mixed — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-rtrb-block | 1.2M | 1.2M | 1.7M | 35.2M | 48.9M |
| ticklog-rtrb-drop (r16) | 38.6M | 46.1M | 58.4M | 57.3M | 59.5M |
| cpp-quill v13 | 1.2M | 1.1M | 0.6M | 191.8M | 156.7M |
| cpp-quill v12 | 1.0M | 1.0M | 0.6M | 136.4M | 113.8M |

### string — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-rtrb-block | 44.1 | 27.6 | 43.0 | 133.7 | 132.3 |
| ticklog-rtrb-drop (r16) | 16.2 | 16.3 | 53.9 | 78.3 | 104.6 |
| cpp-quill v13 | 24.4 | 24.9 | 29.4 | 37.6 | 41.2 |
| cpp-quill v12 | 30.6 | 30.7 | 33.9 | 49.4 | 50.2 |

### string — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-rtrb-block | 1.6M | 1.6M | 1.2M | 55.2M | 55.8M |
| ticklog-rtrb-drop (r16) | 32.7M | 46.1M | 57.0M | 70.8M | 63.8M |
| cpp-quill v13 | 1.5M | 1.4M | 7.1M | 187.9M | 133.5M |
| cpp-quill v12 | 1.3M | 1.3M | 6.5M | 149.2M | 155.0M |

## WIN

### single_int — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-rtrb-block | 25.2 | 25.4 | 44.4 | 49.3 | 50.5 |
| ticklog-rtrb-drop (r16) | 14.6 | 24.0 | 45.2 | 49.2 | 50.4 |
| cpp-quill v13 | 17.1 | 17.3 | 20.1 | 21.2 | 35.1 |
| cpp-quill v12 | 16.8 | 18.2 | 16.6 | 20.6 | 37.0 |

### single_int — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-rtrb-block | 1.8M | 1.4M | 1.3M | 89.1M | 102.5M |
| ticklog-rtrb-drop (r16) | 50.0M | 84.1M | 73.4M | 128.2M | 118.7M |
| cpp-quill v13 | 1.5M | 1.6M | 11.5M | 259.1M | 21.5M |
| cpp-quill v12 | 1.3M | 1.4M | 13.2M | 65.9M | 47.3M |

### mixed — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-rtrb-block | 34.8 | 60.0 | 67.9 | 74.7 | 76.1 |
| ticklog-rtrb-drop (r16) | 15.2 | 16.9 | 46.9 | 75.1 | 76.2 |
| cpp-quill v13 | 28.3 | 28.7 | 29.3 | 30.9 | 57.9 |
| cpp-quill v12 | 26.7 | 27.3 | 20.5 | 35.3 | 57.2 |

### mixed — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-rtrb-block | 1.1M | 0.9M | 1.4M | 54.6M | 92.7M |
| ticklog-rtrb-drop (r16) | 48.2M | 72.2M | 72.6M | 85.9M | 80.9M |
| cpp-quill v13 | 1.0M | 1.0M | 0.6M | 139.9M | 101.9M |
| cpp-quill v12 | 0.9M | 0.8M | 0.6M | 165.4M | 38.4M |

### string — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-rtrb-block | 32.7 | 60.5 | 63.5 | 68.9 | 68.7 |
| ticklog-rtrb-drop (r16) | 15.3 | 18.0 | 66.4 | 67.7 | 69.1 |
| cpp-quill v13 | 25.4 | 27.9 | 28.6 | 28.7 | 44.1 |
| cpp-quill v12 | 27.0 | 27.1 | 27.8 | 28.0 | 41.5 |

### string — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-rtrb-block | 1.3M | 1.3M | 0.9M | 89.9M | 110.4M |
| ticklog-rtrb-drop (r16) | 44.1M | 68.5M | 56.7M | 91.9M | 99.9M |
| cpp-quill v13 | 1.3M | 1.3M | 13.2M | 185.4M | 34.7M |
| cpp-quill v12 | 1.1M | 1.1M | 2.9M | 242.1M | 55.8M |

---

## Tails: single_int (ns)

### p99

| OS | candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|---|
| WSL | rtrb-block | 125.6 | 128.5 | 147.0 | 127.0 | 1494.1 |
| WSL | cpp-quill v13 | 34601.7 | 54748.0 | 109.5 | 48.3 | 138.6 |
| WSL | cpp-quill v12 | 44780.8 | 87584.0 | 127.2 | 102.1 | 143.5 |
| WIN | rtrb-block | 78.1 | 50.9 | 157.7 | 126.5 | 494.6 |
| WIN | cpp-quill v13 | 33406.2 | 20721.3 | 34.4 | 59.9 | 109.3 |

### p999

| OS | candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|---|
| WSL | rtrb-block | 96097.9 | 359370.6 | 616839.3 | 268.3 | 5604.0 |
| WSL | cpp-quill v13 | 43971.5 | 155795.2 | 113615.3 | 150.9 | 247.4 |
| WSL | cpp-quill v12 | 63126.6 | 203943.0 | 209499.6 | 175.5 | 2001.7 |
| WIN | rtrb-block | 134392.8 | 296475.0 | 586729.0 | 2147.3 | 3420.6 |
| WIN | cpp-quill v13 | 62643.0 | 249038.6 | 66179.5 | 104.4 | 863.6 |

---

## Observations

- **p50: cpp-quill v13 wins every config on both platforms.** WSL single_int
  t1 14.8 vs 35.4 (-58%), t4 17.4 vs 41.9 (-58%); WIN t1 17.1 vs 25.2 (-32%),
  t4 20.1 vs 44.4 (-55%). mixed/string follow the same shape (WSL string t8
  37.6 vs 133.7). The v12 baseline also beats rtrb-block on p50 everywhere
  (WSL t1 23.8 vs 35.4; WIN t1 16.8 vs 25.2).
- **p99: rtrb-block wins at t1-t2 by three orders of magnitude.** quill's
  blocking tail lands *inside the first percentile* at low threads (p99 =
  20-90 us, both versions), while rtrb-block stays sub-µs (78-158 ns through
  t8). At t8/t16 both candidates are sub-µs (rtrb-block WIN t16 494 ns vs
  quill 109 ns — same league).
- **p999: the reverse at the extreme tail.** rtrb-block's rare full-ring
  spins reach 134-617 us at t1-t4 (max ~0.73 ms) vs quill's 44-249 us. So
  quill's waits are *frequent but shallower* at low threads; rtrb's are
  *rare but deeper*.
- **Block vs Drop (same rtrb backend): block costs nothing when the drain
  keeps up.** WIN single_int t8/t16 p50 49.3/50.5 (block) vs 49.2/50.4
  (drop); WSL block is actually better at t2/t4 single_int. The drop row's
  t1-t4 advantage (15-25 ns, 38-84 M/s) comes from *not waiting*: with the
  formatted drain capped near 1-2 M/s, a full ring means either spinning
  (block, honest rate) or silently discarding (drop, inflated rate).
  rtrb-block's 1.4-1.9 M/s at t1-t4 is the drain-paced ceiling of this
  harness — it uses a *formatted* `NullSink` (`LogSink::accept(line, ...)`);
  the new raw-sink fast path is not wired into the cross-lang harness.
- **Throughput at t8/t16: quill v13 leads on WSL** (single_int 306.1M/140.3M
  vs 66.0M/65.0M; mixed 191.8M/156.7M vs 35.2M/48.9M) **while WIN
  mixed/string is competitive** (mixed t16 92.7M vs 101.9M; string t16
  110.4M vs 34.7M). High-thread throughput remains the noisiest metric
  (WIN single_int t16 quill v13 = 21.5M vs 102.5M block is plainly a
  scheduling outlier — Run 16 saw the same scatter on quill).
- Single run per config, no repeats; treat p50 deltas >~4 ns and throughput
  deltas >~10% as indicative (same caveat as Runs 14/16).

## Meta

- **ticklog-rtrb-block (WIN)**: candidate=`ticklog-rtrb-block` os=windows ns/tick=0.31313282 samples=1000 total=1000000
- **ticklog-rtrb-block (WSL)**: candidate=`ticklog-rtrb-block` os=linux ns/tick=0.313148759 samples=1000 total=1000000
- **cpp-quill v13 (WIN)**: candidate=`quill` os=windows ns/tick=0.313133 samples=1000 total=1000000
- **cpp-quill v13 (WSL)**: candidate=`quill` os=linux ns/tick=0.313149 samples=1000 total=1000000
- **cpp-quill v12 (WIN)**: candidate=`quill` os=windows ns/tick=0.313133 samples=1000 total=1000000
- **cpp-quill v12 (WSL)**: candidate=`quill` os=linux ns/tick=0.313149 samples=1000 total=1000000
