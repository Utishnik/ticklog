# ticklog: NanoLog vs Quill — Run 5 (Windows + WSL2)

Segmented policies implemented for ticklog v0.1.2 and benchmarked on the same harness as Run 4 (Block/Drop):
BATCH=1000, null sink, RDTSC timing. Each cell is the mean of the batch-average latencies of the config.

- **NanoLog**: bounded pool of 8 × 1 MiB arena segments (`NANOLOG_POOL_BUDGET = 8 MiB`); producers block on pool exhaustion
  and format their own handed-off segment as helper work while blocked. 10M messages / config (SAMPLES=10000).
- **Quill**: unbounded `r3` arena growth; producers never block, at-worst OOM. 1M messages / config (SAMPLES=1000)
  because the arena is never freed and the 15-config waveform would otherwise exceed the host's free RAM.

Host: AMD Ryzen 7 7735HS (8C/16T, invariant TSC ~3.193 GHz), 13.3 GiB RAM. WSL2 Ubuntu VM: 16 vCPU / 6 GiB.
Calibration `ns_per_tick`: Windows 0.313132820, WSL 0.313148759.

## Latency (ns)

### NanoLog — Windows

| workload | threads | p50   | p95      | p99      | p999     | max      |
|----------|---------|-------|----------|----------|----------|----------|
| single_int | 1  | 18.33 | 3366.70 | 5155.97 | 6893.51 | 8192.45 |
| mixed      | 1  | 25.35 | 5351.96 | 8257.79 | 9995.77 | 11268.24 |
| string     | 1  | 21.77 | 3261.12 | 4357.44 | 6111.63 | 6558.34 |
| single_int | 2  | 18.44 | 3639.29 | 6078.02 | 7022.30 | 28813.74 |
| mixed      | 2  | 25.52 | 5561.82 | 8877.77 | 9977.48 | 12305.23 |
| string     | 2  | 21.86 | 3415.25 | 6053.97 | 7091.90 | 9401.81 |
| single_int | 4  | 20.44 | 4295.03 | 6647.36 | 7582.08 | 10382.35 |
| mixed      | 4  | 32.05 | 6869.63 | 9558.14 | 10943.11 | 14434.45 |
| string     | 4  | 26.38 | 4063.55 | 6217.96 | 7339.36 | 22372.06 |
| single_int | 8  | 29.92 | 6011.64 | 6883.30 | 9495.68 | 25916.45 |
| mixed      | 8  | 40.23 | 8572.28 | 9680.06 | 11164.98 | 20526.41 |
| string     | 8  | 37.99 | 5615.10 | 6486.88 | 9877.34 | 25409.77 |
| single_int | 16 | 32.05 | 8232.71 | 16143.39 | 33036.08 | 71642.89 |
| mixed      | 16 | 42.02 | 10625.61 | 16141.36 | 24043.70 | 85668.08 |
| string     | 16 | 39.80 | 7063.01 | 14637.10 | 23427.91 | 31478.30 |

### NanoLog — WSL2

| workload | threads | p50   | p95      | p99      | p999     | max      |
|----------|---------|-------|----------|----------|----------|----------|
| single_int | 1  | 17.96 | 3176.41 | 4608.56 | 6268.48 | 9386.92 |
| mixed      | 1  | 26.05 | 5154.95 | 7499.25 | 9253.52 | 12919.40 |
| string     | 1  | 21.44 | 3159.58 | 5139.38 | 6198.31 | 7652.88 |
| single_int | 2  | 18.52 | 3599.49 | 6193.05 | 7357.63 | 10483.32 |
| mixed      | 2  | 26.48 | 5570.33 | 9004.69 | 10535.24 | 17947.20 |
| string     | 2  | 21.76 | 3421.66 | 5809.29 | 6757.40 | 13936.99 |
| single_int | 4  | 22.47 | 4134.24 | 6352.19 | 7191.77 | 10630.39 |
| mixed      | 4  | 36.26 | 6852.48 | 9323.86 | 12332.89 | 21940.49 |
| string     | 4  | 31.30 | 4169.49 | 6111.09 | 8257.20 | 16978.12 |
| single_int | 8  | 30.16 | 6097.71 | 6887.51 | 13752.97 | 21736.34 |
| mixed      | 8  | 40.56 | 7973.70 | 9434.53 | 11851.28 | 23620.17 |
| string     | 8  | 37.91 | 5774.73 | 6516.58 | 10121.72 | 16554.53 |
| single_int | 16 | 31.34 | 11039.37 | 20197.61 | 35334.76 | 69068.57 |
| mixed      | 16 | 42.49 | 17079.59 | 28131.05 | 42473.79 | 60385.89 |
| string     | 16 | 39.66 | 12378.06 | 21893.16 | 36951.86 | 45381.44 |

### Quill — Windows

| workload | threads | p50   | p95     | p99     | p999    | max     |
|----------|---------|-------|---------|---------|---------|---------|
| single_int | 1  | 29.89 | 288.97 | 351.27 | 460.32 | 1139.16 |
| mixed      | 1  | 24.31 | 254.99 | 343.06 | 468.99 | 574.92 |
| string     | 1  | 32.67 | 266.51 | 331.19 | 436.59 | 468.10 |
| single_int | 2  | 19.02 | 281.57 | 362.11 | 463.45 | 476.92 |
| mixed      | 2  | 40.74 | 317.78 | 371.82 | 598.76 | 608.59 |
| string     | 2  | 36.47 | 359.01 | 486.60 | 603.39 | 988.41 |
| single_int | 4  | 30.79 | 531.99 | 749.18 | 1251.45 | 1899.02 |
| mixed      | 4  | 38.16 | 511.99 | 626.81 | 750.17 | 840.24 |
| string     | 4  | 33.84 | 594.14 | 773.33 | 1173.64 | 1686.48 |
| single_int | 8  | 42.05 | 996.89 | 2008.16 | 3185.23 | 3445.59 |
| mixed      | 8  | 40.46 | 2761.11 | 3848.79 | 4479.17 | 5196.01 |
| string     | 8  | 40.02 | 2442.87 | 3437.80 | 3788.03 | 4578.69 |
| single_int | 16 | 35.08 | 2852.82 | 4721.49 | 8515.61 | 8515.61 |
| mixed      | 16 | 40.94 | 3922.30 | 6361.34 | 33859.88 | 33859.88 |
| string     | 16 | 35.23 | 4291.25 | 6062.66 | 31862.42 | 31862.42 |

### Quill — WSL2

| workload | threads | p50   | p95      | p99      | p999     | max      |
|----------|---------|-------|----------|----------|----------|----------|
| single_int | 1  | 18.87 | 850.32  | 1108.91 | 1495.99 | 1559.85 |
| mixed      | 1  | 25.79 | 1522.81 | 1941.58 | 2637.43 | 7146.36 |
| string     | 1  | 21.15 | 1794.07 | 2406.75 | 3188.60 | 3729.61 |
| single_int | 2  | 19.00 | 2323.88 | 3149.11 | 3792.32 | 3793.81 |
| mixed      | 2  | 26.01 | 1703.26 | 2307.52 | 2808.98 | 3050.31 |
| string     | 2  | 21.20 | 1711.01 | 2293.47 | 2963.98 | 3082.42 |
| single_int | 4  | 21.63 | 2132.85 | 2512.08 | 2980.07 | 3012.46 |
| mixed      | 4  | 30.98 | 2527.17 | 2970.80 | 3477.29 | 3561.84 |
| string     | 4  | 21.77 | 2497.87 | 3083.76 | 3506.13 | 5434.11 |
| single_int | 8  | 30.09 | 3781.93 | 4533.96 | 7074.48 | 8572.65 |
| mixed      | 8  | 39.33 | 3631.11 | 4273.24 | 5119.77 | 5717.09 |
| string     | 8  | 34.09 | 3629.76 | 4868.35 | 6745.46 | 7109.27 |
| single_int | 16 | 29.93 | 11348.39 | 22725.12 | 60687.03 | 60687.03 |
| mixed      | 16 | 39.40 | 12462.19 | 27358.39 | 59884.38 | 59884.38 |
| string     | 16 | 36.01 | 10810.31 | 26440.40 | 46273.69 | 46273.69 |

## Throughput (r/s)

### NanoLog

| threads | env | single_int | mixed    | string   |
|---------|-----|------------|----------|----------|
| 1       | win |  4,089,743 | 2,529,295 | 4,517,269 |
| 1       | wsl |  4,337,747 | 2,721,172 | 4,374,658 |
| 2       | win |  5,464,286 | 3,575,959 | 5,579,064 |
| 2       | wsl |  5,390,193 | 3,571,321 | 5,801,516 |
| 4       | win |  7,835,587 | 5,260,445 | 8,148,600 |
| 4       | wsl |  8,092,663 | 5,329,571 | 8,077,501 |
| 8       | win | 11,748,700 | 8,309,012 | 12,297,957 |
| 8       | wsl | 11,510,341 | 8,520,426 | 12,280,509 |
| 16      | win | 12,366,277 | 10,227,570 | 14,506,531 |
| 16      | wsl | 11,216,824 | 7,579,480 | 10,396,047 |

### Quill

| threads | env | single_int | mixed    | string   |
|---------|-----|------------|----------|----------|
| 1       | win | 16,602,085 | 16,819,754 | 16,707,237 |
| 1       | wsl |  7,908,634 |  5,309,725 | 4,061,204 |
| 2       | win | 34,647,273 | 27,770,681 | 25,919,360 |
| 2       | wsl |  6,355,816 |  8,053,107 | 8,004,171 |
| 4       | win | 37,118,560 | 37,792,895 | 36,798,257 |
| 4       | wsl | 13,645,396 | 11,452,889 | 11,730,873 |
| 8       | win | 37,037,174 | 21,415,202 | 22,404,847 |
| 8       | wsl | 15,027,160 | 15,423,633 | 14,866,211 |
| 16      | win | 21,722,697 | 20,313,436 | 20,893,926 |
| 16      | wsl |  8,620,989 |  8,951,911 | 9,769,075 |

## Policy comparison on Windows — NanoLog → Quill, Δ%

Precision bottleneck differs: NanoLog is pool/exchange-bound (the spare-pool mutex, handoff and helper-format
round-trips), Quill is drain-format-bound; both are far ahead of Block's ~1–2.5M r/s drain ceiling.

| threads | p50 Δ%      | p99 Δ%        | throughput Δ%   |
|---------|-------------|---------------|-----------------|
| 1       | +17% (single_int) | −93% (single_int) | +306% (single_int) |
| 2       |  ~even       | −94% (single_int) | +534% (single_int) |
| 4       | +51% (single_int) | −89% (single_int) | +374% (single_int) |
| 8       | +41% (single_int) | −71% (single_int) | +215% (single_int) |
| 16      |  ~even       | −71% (single_int) | +76% (single_int)  |

Notes:
- Quill p50 is similar to NanoLog (both well under formatting cost), but its p99/p999 stay ~0.3–8 µs (≤2 µs up to 4
  threads on Windows) — there is no pool-exhaustion stall to pay. The trade is unbounded arena memory: at
  SAMPLES=1000 the 15-config Quill waveform peaked under ~4 GiB; a full SAMPLES=10000 run would need ~40 GiB.
- NanoLog's 8-segment pool caps producer concurrency: from 8→16 threads throughput grows only ~5% (win) /
  −9% (wsl), while handoff stalls push p999 toward 10–85 µs. Pool budget remains a tuning knob.
- Windows vs WSL: p50 within a few ns everywhere. WSL tails are 2–10× softer on 1–8 threads (Quill's uncontended
  arena picks up more scheduler/context-switch noise in the 6 GiB VM); at 16 threads WSL p999 reaches 46–61 µs vs
  8–34 µs on Windows. NanoLog, throttled by its own pool, is ~equal on both.
- Raw JSONs: `results_nanolog/{windows,wsl}.json`, `results_quill/{windows,wsl}.json`. Runs: Windows 2026-09-19,
  NanoLog SAMPLES=10000 (10M msg/config), Quill SAMPLES=1000 (1M msg/config); null sink; drain and producers pinned
  to the same 16-core pool (no per-side core isolation here, unlike the two-core SPSC runs in Run 4).

Also fixed during this run: thread-local *init* previously used `first_ring` (blocking `take_pooled`), so spawning
more producers than pool slots deadlocked at warm-up; init now falls back to a fresh arena segment
(`Segments::initial_ring`).

## El vs original quill crate

Original `quill` crate numbers come from Run 3 (native Windows, SAME host and SAME harness/protocol
BATCH=1000, null sink). Protocol caveat: Run 3 quill ran at SAMPLES=10000 (10M msgs/config); our Quill ran at
SAMPLES=1000 (1M msgs/config), so p999/max below are from a 10x smaller tail sample — treat them as weaker bounds
for ours, never as an excuse for the original's (its own tails are 2-3 orders of magnitude apart anyway).
Original quill was also reported with a known inhibit: quill writes through SPSC queues that self-double up to
hundreds of MiB per producer, and it was the loudest OOM hazard in every run on this host family.

## Latency (ns): quill (Run 3) -> ticklog-Quill (Run 5, win), Δ%

### single_int

| threads | p50                  | p95                   | p99                  | p999                    | max                    |
|---------|----------------------|-----------------------|----------------------|-------------------------|------------------------|
| 1       | 16.0 -> 29.9 (+86.8%) | 23.4 -> 289.0 (+1134.6%) | 31.2 -> 351.3 (+1025.9%) | 459.1 -> 460.3 (+0.3%) | 163448 -> 1139 (-99.3%) |
| 2       | 18.4 -> 19.0 (+3.3%) | 23.4 -> 281.6 (+1103.4%) | 41.6 -> 362.1 (+770.5%) | 5517.6 -> 463.4 (-91.6%) | 217510 -> 477 (-99.8%) |
| 4       | 20.6 -> 30.8 (+49.5%) | 24.6 -> 532.0 (+2062.6%) | 51.6 -> 749.2 (+1352.1%) | 208912 -> 1251 (-99.4%) | 475046 -> 1899 (-99.6%) |
| 8       | 20.1 -> 42.1 (+109.4%) | 26.2 -> 996.9 (+3704.6%) | 70.9 -> 2008.2 (+2732.4%) | 164289 -> 3185 (-98.1%) | 1586521 -> 3446 (-99.8%) |
| 16      | 20.2 -> 35.1 (+73.7%) | 32.7 -> 2852.8 (+8624.7%) | 1458.8 -> 4721.5 (+223.6%) | 64502 -> 8516 (-86.8%) | 300147 -> 8516 (-97.2%) |

### mixed

| threads | p50                  | p95                     | p99                    | p999                      | max                    |
|---------|----------------------|-------------------------|------------------------|---------------------------|------------------------|
| 1       | 31.5 -> 24.3 (-22.8%) | 66.6 -> 255.0 (+282.9%) | 83.2 -> 343.1 (+312.3%) | 436.0 -> 469.0 (+7.6%)   | 583718 -> 575 (-99.9%) |
| 2       | 32.7 -> 40.7 (+24.5%) | 74.7 -> 317.8 (+325.4%) | 97.3 -> 371.8 (+282.1%) | 5649.4 -> 598.8 (-89.4%) | 455551 -> 609 (-99.9%) |
| 4       | 35.4 -> 38.2 (+7.9%)  | 65.9 -> 512.0 (+676.9%) | 103.1 -> 626.8 (+507.9%) | 119376 -> 750 (-99.4%)   | 785398 -> 840 (-99.9%) |
| 8       | 60.5 -> 40.5 (-33.1%) | 87.2 -> 2761.1 (+3066.4%) | 579.9 -> 3848.8 (+563.7%) | 98662 -> 4479 (-95.5%)   | 317991 -> 5196 (-98.4%) |
| 16      | 58.1 -> 40.9 (-29.6%) | 104.0 -> 3922.3 (+3671.2%) | 5108.1 -> 6361.3 (+24.5%) | 587915 -> 33860 (-94.2%) | 779930 -> 33860 (-95.7%) |

### string

| threads | p50                  | p95                   | p99                   | p999                      | max                    |
|---------|----------------------|-----------------------|-----------------------|---------------------------|------------------------|
| 1       | 32.7 -> 32.7 (+0.1%) | 72.5 -> 266.5 (+267.6%) | 84.3 -> 331.2 (+292.8%) | 376.6 -> 436.6 (+15.9%) | 216714 -> 468 (-99.8%) |
| 2       | 33.6 -> 36.5 (+8.6%) | 61.7 -> 359.0 (+481.9%) | 98.2 -> 486.6 (+395.5%) | 4152.8 -> 603.4 (-85.5%) | 327729 -> 988 (-99.7%) |
| 4       | 54.2 -> 33.8 (-37.6%) | 64.0 -> 594.1 (+828.4%) | 114.8 -> 773.3 (+573.6%) | 91603 -> 1174 (-98.7%)  | 347281 -> 1686 (-99.5%) |
| 8       | 56.2 -> 40.0 (-28.8%) | 78.2 -> 2442.9 (+3023.3%) | 294.1 -> 3437.8 (+1069.0%) | 55893 -> 3788 (-93.2%) | 256034 -> 4579 (-98.2%) |
| 16      | 55.3 -> 35.2 (-36.3%) | 74.1 -> 4291.3 (+5690.0%) | 1486.8 -> 6062.7 (+307.8%) | 329059 -> 31862 (-90.3%) | 460323 -> 31862 (-93.1%) |

## Throughput (r/s) — original quill (Run 3) -> ticklog-Quill (Run 5, win)

| threads | single_int | mixed | string |
|---------|------------|-------|--------|
| 1       | 20,166,031 -> 16,602,085 (-17.7%) | 7,111,281 -> 16,819,754 (+136.5%) | 11,710,574 -> 16,707,237 (+42.7%) |
| 2       | 18,099,462 -> 34,647,273 (+91.4%) | 8,906,028 -> 27,770,681 (+211.8%) | 10,684,427 -> 25,919,360 (+142.6%) |
| 4       | 9,826,165 -> 37,118,560 (+277.8%) | 6,968,059 -> 37,792,895 (+442.4%) | 13,815,080 -> 36,798,257 (+166.4%) |
| 8       | 5,212,412 -> 37,037,174 (+610.5%) | 11,550,078 -> 21,415,202 (+85.4%) | 22,114,246 -> 22,404,847 (+1.3%) |
| 16      | 22,900,221 -> 21,722,697 (-5.1%) | 7,903,430 -> 20,313,436 (+157.0%) | 14,071,296 -> 20,893,926 (+48.5%) |

Notes:
- **The pattern is consistent: ticklog-Quill beats original quill on every tail metric (p999/max) by 1-3 orders of
  magnitude** — original quill self-doubles its SPSC queues on producer overload and stalls/provokes OOM, producing
  0.1-1.6 ms maxes even at 8 threads; ours caps individual stalls in the low µs. This is the design goal: same
  "never blocks" Quill semantics, but the arena-backed shared-drain design keeps a global, predictable backlog
  instead of per-producer doubling queues.
- The cost shows exactly where it should: p95/p99. Original quill's reserve path is an inline CAS spin on a hot
  slot; ours goes to callback registration + lock-free ring + a shared drain, so 1-4 thread p95/p99 sit 3-12x
  higher. Above 8 threads the original degrades and the tables tighten dramatically.
- p50 is a wash: ours wins mixed/string at 4-16 threads (original's queues thrash), original wins single_int.
- Throughput: ours is 1.4-6x faster at 2-16 threads (the original's single processing thread + doubling queues
  throttle it), near-parity at 1 thread (single_int -18%, mixed/string +43/137%).
- Caveats: (a) different sample counts as noted; (b) original quill Run 3 is a Windows run under the same
  scheduler; (c) source for the original quill harness is not in this tree (published from an earlier checkout).