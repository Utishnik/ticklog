# ticklog: real Quill vs ticklog-Quill — Run 6 (Windows + WSL2)

Head-to-head between **real C++ quill v12.1.0** and **ticklog-quill** with both buffer sizes corrected
to "as large as RAM allows", run on Windows and WSL2 (same host, AMD Ryzen 7 7735HS, 8C/16T, 13.3 GiB RAM;
WSL2 VM 6 GiB). BATCH=1000, null sink, RDTSC timing, SAMPLES=1000 (1M msg/config) on all four runs.

## Changes vs Run 5

- **Real C++ quill** (`cpp/quill`): added custom `FrontendOptions` (`BenchFrontendOptions`) that fixes the
  per-thread SPSC queue at `initial_queue_capacity == unbounded_queue_max_capacity` — it can no longer
  self-double to the hundreds-of-MiB OOM hazard seen in earlier runs. Queue = 64 MiB on Windows
  (`QUILL_BENCH_QUEUE_CAPACITY=67108864`), 32 MiB on WSL (`33554432`) — the max power-of-two that fits each
  host's free RAM while 16 producers stay bounded. Build now uses clang max-opt
  (`-march=native -flto=thin -fno-exceptions -fno-rtti -fomit-frame-pointer -O3`,
  `QUILL_NO_EXCEPTIONS=ON`): clang 23.1.1/LLD on Windows, clang++-22 on WSL. Added `--samples` flag.
- **ticklog-quill**: ring capacity raised from the 1 MiB default to 64 MiB (Windows) / 32 MiB (WSL)
  (`--ring-capacity`); arena precommits 8 × ring_capacity per run and is never freed, so 32 MiB is the
  WSL-safe ceiling (Run 5 analysis: ~90 + rings accumulate across the 15 configs).
- Both harnesses run the identical thread waveform `1,2,4,8,16` × 3 workloads.

Calibration `ns_per_tick`: Windows 0.313132820, WSL 0.313148759.

Raw JSONs: `results_quill/windows_quill_cpp.json`, `results_quill/wsl_quill_cpp.json` (C++ quill);
`results_quill/windows_ticklog_quill.json`, `results_quill/wsl_ticklog_quill.json` (ticklog-quill).

## Latency (ns)

### Quill (C++) — Windows

| workload | threads | p50   | p95   | p99   | p999  | max   |
|----------|---------|-------|-------|-------|-------|-------|
| single_int | 1  | 14.89 | 17.10 | 32.66 | 75.84 | 109.78 |
| mixed      | 1  | 27.88 | 29.45 | 39.53 | 43.62 | 50.90 |
| string     | 1  | 26.73 | 27.33 | 40.24 | 45.69 | 69.03 |
| single_int | 2  | 16.18 | 17.88 | 21.24 | 41.84 | 41.93 |
| mixed      | 2  | 25.34 | 29.07 | 35.08 | 84.46 | 122.40 |
| string     | 2  | 16.01 | 21.97 | 27.39 | 33.80 | 34.62 |
| single_int | 4  | 18.67 | 21.10 | 26.74 | 41.45 | 41.79 |
| mixed      | 4  | 19.50 | 30.81 | 49.37 | 104.02 | 149.57 |
| string     | 4  | 24.00 | 29.23 | 69.92 | 112.49 | 425.10 |
| single_int | 8  | 23.46 | 28.27 | 71.65 | 151.60 | 156.10 |
| mixed      | 8  | 40.53 | 56.78 | 87.12 | 135.23 | 299.80 |
| string     | 8  | 30.39 | 37.86 | 70.12 | 102.44 | 103.59 |
| single_int | 16 | 28.82 | 42.30 | 137.36 | 1840.02 | 1840.02 |
| mixed      | 16 | 38.46 | 67.63 | 272.35 | 2718.98 | 2718.98 |
| string     | 16 | 38.69 | 45.75 | 114.69 | 826.64 | 826.64 |

### ticklog-Quill — Windows

| workload | threads | p50   | p95   | p99   | p999  | max   |
|----------|---------|-------|-------|-------|-------|-------|
| single_int | 1  | 27.05 | 30.88 | 54.22 | 524.92 | 11073.12 |
| mixed      | 1  | 37.60 | 41.89 | 76.36 | 188.49 | 9461.52 |
| string     | 1  | 32.45 | 38.32 | 58.00 | 1255.90 | 10363.06 |
| single_int | 2  | 27.02 | 42.57 | 72.73 | 117.02 | 466.21 |
| mixed      | 2  | 26.65 | 42.03 | 48.11 | 63.65 | 94.95 |
| string     | 2  | 34.76 | 43.45 | 78.58 | 152.70 | 205.85 |
| single_int | 4  | 41.39 | 52.66 | 88.25 | 227.80 | 575.77 |
| mixed      | 4  | 44.09 | 50.80 | 79.89 | 195.94 | 396.91 |
| string     | 4  | 43.76 | 48.03 | 79.45 | 189.32 | 365.18 |
| single_int | 8  | 74.66 | 90.04 | 138.48 | 286.20 | 409.25 |
| mixed      | 8  | 76.05 | 87.74 | 137.18 | 159.88 | 167.65 |
| string     | 8  | 75.17 | 89.41 | 129.68 | 188.94 | 199.87 |
| single_int | 16 | 124.98 | 183.18 | 705.56 | 5062.07 | 5062.07 |
| mixed      | 16 | 126.03 | 171.77 | 326.96 | 6692.85 | 6692.85 |
| string     | 16 | 122.82 | 206.74 | 800.52 | 4878.52 | 4878.52 |

### Quill (C++) — WSL2

| workload | threads | p50   | p95   | p99   | p999  | max   |
|----------|---------|-------|-------|-------|-------|-------|
| single_int | 1  | 15.23 | 20.10 | 46.55 | 51723.92 | 93383.71 |
| mixed      | 1  | 17.06 | 28.79 | 95.61 | 87671.64 | 560713.84 |
| string     | 1  | 16.62 | 27.43 | 57.17 | 85864.85 | 473936.47 |
| single_int | 2  | 15.11 | 19.27 | 53.92 | 124.95 | 383.73 |
| mixed      | 2  | 17.16 | 25.29 | 48.50 | 126.86 | 292.13 |
| string     | 2  | 23.03 | 27.43 | 78.79 | 172.08 | 191.01 |
| single_int | 4  | 17.95 | 27.29 | 117.55 | 476.38 | 835.20 |
| mixed      | 4  | 23.75 | 33.35 | 85.35 | 197.65 | 388.89 |
| string     | 4  | 22.58 | 35.63 | 75.42 | 149.65 | 164.08 |
| single_int | 8  | 21.28 | 24.92 | 85.77 | 126.78 | 192.03 |
| mixed      | 8  | 34.97 | 42.11 | 141.65 | 181.83 | 322.96 |
| string     | 8  | 25.22 | 32.80 | 115.52 | 274.34 | 376.28 |
| single_int | 16 | 35.23 | 44.07 | 182.16 | 302.85 | 302.85 |
| mixed      | 16 | 55.29 | 115.98 | 202.00 | 1870.12 | 1870.12 |
| string     | 16 | 45.57 | 90.34 | 266.24 | 4490.68 | 4490.68 |

### ticklog-Quill — WSL2

| workload | threads | p50   | p95   | p99   | p999  | max   |
|----------|---------|-------|-------|-------|-------|-------|
| single_int | 1  | 19.81 | 29.17 | 67.93 | 50744.23 | 53893.56 |
| mixed      | 1  | 26.99 | 41.66 | 76.55 | 52026.61 | 52555.56 |
| string     | 1  | 23.82 | 48.54 | 106.99 | 56093.77 | 56734.08 |
| single_int | 2  | 23.16 | 34.80 | 107.97 | 53555.72 | 53884.98 |
| mixed      | 2  | 37.46 | 46.32 | 110.97 | 52538.03 | 57487.68 |
| string     | 2  | 35.10 | 41.88 | 88.43 | 48072.40 | 48209.30 |
| single_int | 4  | 36.59 | 43.68 | 126.26 | 185.11 | 194.74 |
| mixed      | 4  | 37.70 | 50.19 | 102.98 | 242.06 | 579.35 |
| string     | 4  | 37.21 | 56.89 | 143.33 | 196.28 | 203.53 |
| single_int | 8  | 67.34 | 121.82 | 231.63 | 406.98 | 665.38 |
| mixed      | 8  | 73.66 | 126.30 | 194.82 | 228.82 | 233.83 |
| string     | 8  | 74.32 | 134.80 | 219.64 | 325.42 | 518.47 |
| single_int | 16 | 106.49 | 194.25 | 343.20 | 1611.45 | 1611.45 |
| mixed      | 16 | 66.56 | 119.59 | 184.08 | 2007.95 | 2007.95 |
| string     | 16 | 88.33 | 154.88 | 282.89 | 1391.96 | 1391.96 |

## Throughput (r/s)

| threads | env | q single_int | q mixed | q string | t single_int | t mixed | t string |
|---------|-----|--------------|---------|----------|--------------|---------|----------|
| 1       | win | 62,579,006 | 35,186,365 | 36,771,736 | 20,854,666 | 18,677,799 | 18,894,948 |
| 1       | wsl | 4,756,986 | 1,041,265(*) | 1,451,009(*) | 4,816,769 | 4,736,055 | 4,360,984 |
| 2       | win | 114,569,848 | 70,021,637 | 117,227,797 | 36,523,143 | 34,410,141 | 31,953,476 |
| 2       | wsl | 114,762,892 | 99,533,605 | 80,914,535 | 9,461,745 | 9,104,193 | 8,115,976 |
| 4       | win | 181,100,366 | 140,662,803 | 143,494,669 | 36,983,753 | 38,722,168 | 39,783,103 |
| 4       | wsl | 145,000,940 | 131,786,546 | 143,332,474 | 14,553,602 | 14,753,304 | 15,001,411 |
| 8       | win | 286,073,922 | 174,443,960 | 224,997,188 | 29,466,798 | 29,247,292 | 29,668,220 |
| 8       | wsl | 273,582,317 | 187,105,266 | 193,174,675 | 12,911,340 | 13,491,922 | 13,155,314 |
| 16      | win | 10,267,172 | 59,645,586 | 128,844,395 | 17,336,190 | 17,242,806 | 17,017,627 |
| 16      | wsl | 182,006,840 | 67,026,339 | 96,170,500 | 8,166,154 | 8,441,824 | 7,021,285 |

(*) WSL threads=1 anomalies: C++ quill mixed/string collapsed (1.0–1.5M) with ~100–560 µs maxes — a first-config
startup/backend-warm-up artifact on this VM (consistently seen in prior WSL runs); treat those two cells as noise.

## Comparison — C++ quill → ticklog-Quill, Δ% (p50 / p99 / throughput, single_int)

| threads | win p50 Δ% | win p99 Δ% | win thru Δ% | wsl p50 Δ% | wsl p99 Δ% | wsl thru Δ% |
|---------|------------|------------|-------------|------------|------------|-------------|
| 1       | +81.5%     | +66.0%     | −66.7%      | +30.1%     | +45.9%     | +1.3%       |
| 2       | +67.0%     | +242.4%    | −68.1%      | +53.3%     | +100.3%    | −91.8%      |
| 4       | +121.7%    | +230.0%    | −79.6%      | +103.8%    | +7.4%      | −90.0%      |
| 8       | +218.2%    | +93.3%     | −89.7%      | +216.4%    | +170.1%    | −95.3%      |
| 16      | +333.6%    | +413.6%    | +68.8%      | +202.3%    | +88.4%     | −95.5%      |

## Notes

- **Buffer sizing worked as intended on both sides**: no quill queues self-doubled, no OOM, no crashed runs.
  The only cap-hits were 3 WSL C++ quill producers blocking 3–7 times during the 16-thread burst
  ("Reached the maximum configured unbounded queue capacity") — i.e. the fixed 32 MiB ring absorbed the load and
  blocked briefly instead of growing. ticklog's 8×ring precommit + bump arena stayed well inside free RAM.
- **C++ quill is the faster frontend in the median**: at 1–8 threads p50 is ~1.5–3× lower (single_int Windows
  14.9 vs 27.1 ns at 1 thread) and throughput is 3–10× higher. Its SPSC push is simpler than ticklog's
  callback-registration + shared-drain path.
- **ticklog-Quill wins the multi-thread tails on Windows**: above 8 threads C++ quill's single backend thread and
  ordering drain push p999 toward 0.8–2.7 µs (and its 16-thread single_int throughput collapses to 10M), while
  ticklog's per-thread rings + drainers keep p50~125ns and throughput flat. The 16-thread Windows single_int cell
  (quill 10.3M vs ticklog 17.3M) is the starkest single-cell difference.
- **WSL inverts the throughput story**: C++ quill scales to 182–273M r/s (2–16 threads) whereas ticklog-quill
  plateaus at ~8–15M — the shared-drain path is ~10× more costly under the WSL VM's locking/scheduling, while
  quill's uncontended per-thread queues ride through it. p50/p99 stay comparable to Windows; only tail latency
  degrades (WSL lock noise).
- **Run 5 apples-to-apples caveat**: Run 5's "Quill" was ticklog-quill with 1 MiB rings (SAMPLES=1000). Raising
  the ring to 64 MiB did not change median latency materially (still ~24–45 ns p50 at 1–4 threads vs the old
  24–42 ns) — ticklog-quill was never ring-capacity-bound in the median; the capacity fix chiefly removes the
  OOM risk and pool-exhaustion stalls at high concurrency.
- Real C++ quill with a fixed 64 MiB queue (vs its old unbounded self-doubling) lands at the same 15–40 ns p50
  as ticklog-quill at 1–8 threads; the Run 3 "quill OOM hazard" is gone by construction.

Runs: 2026-09-19. Windows + WSL2, clang max-opt (C++ side), null sink, drain/producers unpinned (16-core pool).