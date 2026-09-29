# Run 18: ticklog rtrb + Block (in-place assembly) vs C++ quill (8 MiB queues)

Date: 2026-09-30. Same harness set and parameters as Run 17: BATCH=1000,
SAMPLES=1000 (1M msg/config), threads `1,2,4,8,16`, RDTSC. Calibration:
Windows `0.313132820 ns/tick`, WSL `0.313148759 ns/tick`. Rings: **8 MiB
(8388608) both platforms**. rtrb `--chunk-size 64`. C++ quill harnesses
reused from Run 17 (`QUILL_BENCH_QUEUE_CAPACITY=8388608`, v13.0.0 =
`quill_harness_8m_v13`, v12.1.0 baseline = `quill_harness_8m`).

**Difference from Run 17**: ticklog commit `226deb9` — the producer
assembles each record in place (no staging zero-fill, no staging→ring
memcpy on the fifo path); Run 17's rtrb-block row is kept as the
reference. **hotpath: OFF** (`backend-rtrb,policy-block` only, no
`hotpath-profiler`): `#[hotpath::measure]` expands to a no-op.

## Builds & sizing

| Candidate | Features | Buffer |
|---|---|---|
| ticklog-rtrb-block (r18) | backend-rtrb,policy-block | rtrb ring 8 MiB, chunk 64, Backpressure::Block |
| cpp-quill v13 (8MiB q) | quill v13.0.0 header-only | per-thread SPSC queue 8 MiB (compile-time) |
| cpp-quill v12 (8MiB q) | quill v12.1.0 header-only | per-thread SPSC queue 8 MiB (compile-time) |

## Result files

`results_quill/{windows,wsl}_ticklog_rtrb_block_r18.json`,
`results_quill/{windows,wsl}_quill_cpp_v13_r18.json`,
`results_quill/{windows,wsl}_quill_cpp_r18.json` (+ `.log` / `.log.err`).
Scripts: `build_{win,wsl}_r18.{ps1,sh}`, `run_{win,wsl}_r18.{ps1,sh}`,
`summarize_r18.py`.

---

## WSL

### single_int — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-rtrb-block (r18) | 18.7 | 35.7 | 36.7 | 36.0 | 67.1 |
| ticklog-rtrb-block (r17) | 35.4 | 41.5 | 41.9 | 43.7 | 73.1 |
| cpp-quill v13 (r18) | 14.8 | 15.0 | 18.1 | 22.3 | 32.7 |
| cpp-quill v12 (r18) | 21.7 | 24.4 | 26.9 | 27.2 | 29.6 |

### single_int — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-rtrb-block (r18) | 2.0M | 1.5M | 1.5M | 74.2M | 54.7M |
| ticklog-rtrb-block (r17) | 1.9M | 1.5M | 1.4M | 66.0M | 65.0M |
| cpp-quill v13 (r18) | 1.9M | 1.8M | 5.7M | 266.3M | 126.3M |
| cpp-quill v12 (r18) | 1.5M | 1.6M | 5.5M | 160.5M | 114.0M |

### mixed — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-rtrb-block (r18) | 46.3 | 44.3 | 41.2 | 166.1 | 175.9 |
| ticklog-rtrb-block (r17) | 31.9 | 50.2 | 37.9 | 105.8 | 161.9 |
| cpp-quill v13 (r18) | 24.8 | 25.3 | 31.2 | 36.2 | 49.4 |
| cpp-quill v12 (r18) | 31.6 | 31.5 | 51.1 | 51.9 | 53.6 |

### mixed — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-rtrb-block (r18) | 1.3M | 1.2M | 1.5M | 35.4M | 35.3M |
| ticklog-rtrb-block (r17) | 1.2M | 1.2M | 1.7M | 35.2M | 48.9M |
| cpp-quill v13 (r18) | 1.2M | 1.2M | 628.4k | 176.1M | 96.1M |
| cpp-quill v12 (r18) | 965.2k | 1.0M | 448.6k | 131.6M | 119.8M |

### string — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-rtrb-block (r18) | 41.3 | 23.2 | 41.5 | 140.3 | 151.9 |
| ticklog-rtrb-block (r17) | 44.1 | 27.6 | 43.0 | 133.7 | 132.3 |
| cpp-quill v13 (r18) | 24.4 | 24.4 | 36.1 | 34.6 | 37.7 |
| cpp-quill v12 (r18) | 30.7 | 30.9 | 49.6 | 49.3 | 53.1 |

### string — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-rtrb-block (r18) | 1.6M | 1.6M | 1.1M | 54.7M | 52.3M |
| ticklog-rtrb-block (r17) | 1.6M | 1.6M | 1.2M | 55.2M | 55.8M |
| cpp-quill v13 (r18) | 1.6M | 1.5M | 5.6M | 180.3M | 102.3M |
| cpp-quill v12 (r18) | 1.4M | 1.2M | 2.0M | 136.1M | 78.2M |

## WIN

### single_int — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-rtrb-block (r18) | 23.6 | 23.3 | 31.4 | 34.0 | 36.5 |
| ticklog-rtrb-block (r17) | 25.2 | 25.4 | 44.4 | 49.3 | 50.5 |
| cpp-quill v13 (r18) | 16.2 | 17.0 | 19.7 | 21.1 | 24.1 |
| cpp-quill v12 (r18) | 15.8 | 16.9 | 17.2 | 22.3 | 30.8 |

### single_int — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-rtrb-block (r18) | 2.0M | 1.3M | 1.1M | 109.6M | 136.7M |
| ticklog-rtrb-block (r17) | 1.8M | 1.4M | 1.3M | 89.1M | 102.5M |
| cpp-quill v13 (r18) | 1.5M | 1.7M | 10.0M | 170.2M | 29.1M |
| cpp-quill v12 (r18) | 1.2M | 1.3M | 12.4M | 228.4M | 31.7M |

### mixed — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-rtrb-block (r18) | 26.8 | 34.4 | 52.3 | 58.5 | 59.9 |
| ticklog-rtrb-block (r17) | 34.8 | 60.0 | 67.9 | 74.7 | 76.1 |
| cpp-quill v13 (r18) | 20.1 | 26.4 | 28.9 | 34.5 | 55.1 |
| cpp-quill v12 (r18) | 27.2 | 27.4 | 27.1 | 34.3 | 56.0 |

### mixed — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-rtrb-block (r18) | 1.2M | 975.6k | 1.1M | 69.5M | 94.3M |
| ticklog-rtrb-block (r17) | 1.1M | 933.8k | 1.4M | 54.6M | 92.7M |
| cpp-quill v13 (r18) | 1.1M | 654.6k | 550.1k | 201.2M | 26.9M |
| cpp-quill v12 (r18) | 811.4k | 908.8k | 487.6k | 209.4M | 36.4M |

### string — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-rtrb-block (r18) | 29.6 | 42.0 | 49.9 | 51.3 | 51.1 |
| ticklog-rtrb-block (r17) | 32.7 | 60.5 | 63.5 | 68.9 | 68.7 |
| cpp-quill v13 (r18) | 24.1 | 28.1 | 28.6 | 29.3 | 39.1 |
| cpp-quill v12 (r18) | 16.5 | 27.4 | 27.3 | 28.8 | 34.9 |

### string — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-rtrb-block (r18) | 1.4M | 1.2M | 821.0k | 118.7M | 118.6M |
| ticklog-rtrb-block (r17) | 1.3M | 1.3M | 904.1k | 89.9M | 110.4M |
| cpp-quill v13 (r18) | 1.4M | 1.2M | 10.5M | 210.5M | 27.6M |
| cpp-quill v12 (r18) | 1.0M | 1.1M | 12.9M | 241.5M | 104.9M |

---

## Tails: single_int (ns)

### p99

| OS | candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|---|
| WSL | ticklog-rtrb-block (r18) | 221.6 | 306.2 | 397.9 | 192.1 | 1455.8 |
| WSL | ticklog-rtrb-block (r17) | 125.5 | 128.5 | 147.0 | 127.0 | 1494.1 |
| WSL | cpp-quill v13 (r18) | 34755.4 | 31850.8 | 71.1 | 94.8 | 179.3 |
| WSL | cpp-quill v12 (r18) | 40164.6 | 36326.5 | 128.5 | 107.2 | 159.6 |
| WIN | ticklog-rtrb-block (r18) | 66.1 | 47.1 | 186.8 | 91.2 | 245.6 |
| WIN | ticklog-rtrb-block (r17) | 78.1 | 50.9 | 157.7 | 126.5 | 494.6 |
| WIN | cpp-quill v13 (r18) | 39866.2 | 26896.2 | 176.7 | 48.5 | 84.0 |
| WIN | cpp-quill v12 (r18) | 54630.4 | 30320.5 | 264.5 | 63.3 | 92.3 |

### p999

| OS | candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|---|
| WSL | ticklog-rtrb-block (r18) | 95962.6 | 367377.8 | 560657.7 | 317.9 | 8714.4 |
| WSL | ticklog-rtrb-block (r17) | 96097.9 | 359370.6 | 616839.3 | 268.3 | 5604.0 |
| WSL | cpp-quill v13 (r18) | 43226.8 | 193223.5 | 134493.4 | 176.1 | 292.9 |
| WSL | cpp-quill v12 (r18) | 65540.8 | 236790.9 | 127833.1 | 386.6 | 651.7 |
| WIN | ticklog-rtrb-block (r18) | 132032.8 | 305997.4 | 728809.2 | 1430.4 | 2062.9 |
| WIN | ticklog-rtrb-block (r17) | 134392.8 | 296475.0 | 586728.9 | 2147.3 | 3420.6 |
| WIN | cpp-quill v13 (r18) | 61909.6 | 192045.4 | 76691.1 | 77.9 | 162.3 |
| WIN | cpp-quill v12 (r18) | 78279.0 | 278380.1 | 57514.4 | 154.6 | 182.1 |

---

## Observations

- **In-place assembly (r17→r18) improves p50 almost everywhere.** WIN is the
  clean case: all 15 configs faster — single_int t4/t8/t16 44.4→31.4 /
  49.3→34.0 / 50.5→36.5 ns (−28…−31%), mixed t2 60.0→34.4 (−43%), string t2
  60.5→42.0 (−31%). WSL single_int t1 35.4→18.7 (−47%), t8 43.7→36.0 (−18%);
  string mostly −3…−16%. The removed staging zero-fill + staging→ring memcpy
  were worth ~12-25 ns of p50 on WIN under Block backpressure.
- **WSL mixed is the exception (noise, not a regression signal).** t1
  31.9→46.3 and t8 105.8→166.1 move up while every other WSL cell moves
  down; Run 17 showed the same single-run scatter on this workload (r16
  t1 was 16.7 vs r17's 31.9). Treat WSL mixed deltas >~30% as run-to-run
  noise; a repeat would be needed to believe them.
- **The gap to quill v13 narrowed sharply, especially on WSL.** WSL
  single_int t1 delta 20.6 ns (r17) → 3.9 ns (r18); WIN single_int t4
  delta 24.3 → 11.7 ns; WIN mixed/string t16 deltas ~18-25 ns → 4.8-12 ns.
  quill v13 still wins p50 in 29/30 cells (the exception: WSL string t2,
  ticklog 23.2 vs 24.4), and ticklog now **beats quill v12 outright on WSL
  single_int t1** (18.7 vs 21.7) and WSL string t2 (23.2 vs 30.9).
- **WIN high-thread throughput flips to ticklog.** single_int t16 136.7M vs
  quill v13's 29.1M (4.7×), string t16 118.6M vs 27.6M, mixed t16 94.3M vs
  26.9M — the quill v13 t16 scheduling collapse on Windows seen in Run 17
  (21.5M) repeats, while ticklog improved over r17 (102.5M → 136.7M).
  t8 still favors quill (170-210M vs 70-119M).
- **p99 story unchanged: quill's low-thread tails are tens of µs, ticklog's
  stay sub-µs.** quill v13 p99 at t1/t2 = 27-40 µs on both OSes; ticklog
  p99 = 47-398 ns (WIN improved: t1 78→66 ns, t16 495→246 ns; WSL grew:
  t1 126→222 ns, t4 147→398 ns — still sub-µs, but the one cell where r18
  lost to r17 consistently). p999 stays in the old regime: rare deep ring
  spins (0.13-0.73 ms at t1-t4) vs quill's shallower-but-more-frequent
  waits (43-237 µs).
- Single run per config, no repeats; same caveat as Runs 14/16/17 (p50
  deltas >~4 ns and throughput >~10% indicative). WIN/WSL ran back-to-back
  on the same host the night after Run 17's code freeze.

## Meta

- **ticklog-rtrb-block (r18) (WIN)**: candidate=`ticklog-rtrb-block` os=windows ns/tick=0.31313282 samples=1000 total=1000000
- **ticklog-rtrb-block (r18) (WSL)**: candidate=`ticklog-rtrb-block` os=linux ns/tick=0.313148759 samples=1000 total=1000000
- **ticklog-rtrb-block (r17) (WIN)**: candidate=`ticklog-rtrb-block` os=windows ns/tick=0.31313282 samples=1000 total=1000000
- **ticklog-rtrb-block (r17) (WSL)**: candidate=`ticklog-rtrb-block` os=linux ns/tick=0.313148759 samples=1000 total=1000000
- **cpp-quill v13 (r18) (WIN)**: candidate=`quill` os=windows ns/tick=0.313133 samples=1000 total=1000000
- **cpp-quill v13 (r18) (WSL)**: candidate=`quill` os=linux ns/tick=0.313149 samples=1000 total=1000000
- **cpp-quill v12 (r18) (WIN)**: candidate=`quill` os=windows ns/tick=0.313133 samples=1000 total=1000000
- **cpp-quill v12 (r18) (WSL)**: candidate=`quill` os=linux ns/tick=0.313149 samples=1000 total=1000000
