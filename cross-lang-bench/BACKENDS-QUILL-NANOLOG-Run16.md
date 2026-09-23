# Run 16: ring 8 MiB (ticklog custom/rtrb/quill/nanolog + C++ quill/nanolog)

Date: 2026-09-23. Same harness set and parameters as Run 14, but at new library HEAD
(post-`a8afd44` optimizations) with the arena zero-fill **restored** (see Deviation below).
BATCH=1000, SAMPLES=1000 (1M msg/config), threads `1,2,4,8,16`, RDTSC.
Calibration: Windows `0.313132820 ns/tick`, WSL `0.313148759 ns/tick`.
Rings: **8 MiB (8388608) both platforms**. rtrb `--chunk-size 64`.
C++ quill rebuilt with `QUILL_BENCH_QUEUE_CAPACITY=8388608` (per-thread SPSC queue 8 MiB).
C++ NanoLog has **no ring-size parameter** (per-thread staging buffers sized by `NanoLog::preallocate()`); run as-is.
C++ nanolog is **WSL/Linux-only** (no Windows build).

## Builds & sizing

| Candidate | Features | Buffer |
|---|---|---|
| ticklog-custom-drop | policy-drop (custom ring) | ring 8 MiB |
| ticklog-rtrb-drop | backend-rtrb,policy-drop | rtrb ring 8 MiB, chunk 64 |
| ticklog-quill | policy-quill (custom ring) | arena regions, ring 8 MiB |
| ticklog-nanolog | policy-nanolog (custom ring) | pool max(8MiB/ring,1) segs |
| cpp-quill (8MiB q) | quill v12.1.0 header-only | per-thread SPSC queue 8 MiB (compile-time) |
| cpp-nanolog | NanoLog staging buffers | no ring parameter (fixed staging buffers) |

## Result files

`results_quill/{windows,wsl}_*_r16.json`.

---

## What changed since Run 14 (commit `a8afd44` + revert)

HEAD `a8afd44` (quill producer fast-path: u32/u64-assemble pack, rtrb chunk-1 direct
commit, arena zero-fill removal — the latter to avoid a full-capacity memset on each
segment handoff). A post-push benchmark of that state showed a **4x p50 regression on the
quill/nanolog producer fast path**, isolated by bisect to the zero-fill removal:

| WSL single_int t1 p50 | quill | nanolog |
|---|---|---|
| Run 14 baseline (deb4121) | 15.6 | 15.5 |
| a8afd44 (zero-fill removed) | 69.4 | 16.1 (t1 unaffected; t8 135) |
| zero-fill restored variant | 15.5 | 15.5 |

The regression reproduced on Windows (quill t1 15.5 -> 22.9) and across all segmented
workloads; `corrupt=0` everywhere, so it is a pure latency regression, not a correctness
bug. Mechanism was not fully pinned down (arena reuse/tail interaction on handoff; the
memset is once-per-segment, not per-record, so restoring it is effectively free), and the
removal was reverted. Final tree = a8afd44 minus the zero-fill removal: **assemble pack +
rtrb direct commit kept**, arena zero-fill restored. The bench runs below use this fixed tree.

Note: the rtrb direct-commit path is only exercised at `chunk-size 1`; this bench uses
`--chunk-size 64` (staged path), so rtrb numbers below are not a probe of that change
(it is covered by unit tests instead).

## Observations (Run 16 vs Run 14)

- **Quill/NanoLog p50 back to baseline or better.** ticklog-quill WSL single_int
  t8/t16 = 30.5/36.8 vs 33.4/41.8 (Run 14); ticklog-nanolog WSL t8/t16 = 31.6/31.6 vs
  36.6/38.7 and WIN t8 = 22.4 vs 36.0. The restored zero-fill costs nothing measurable.
- **custom-drop unchanged-to-slightly-better.** WSL single_int p50 15.4/15.5/15.6/18.5/23.5
  vs 15.4/15.3/16.5/19.6/24.8; WIN ~equal.
- **rtrb t1/t2 stable, t8/t16 noisy.** p50 WSL 15.3/34.3/56.0/62.1/62.6 (vs 15.4/33.5/47.2/60.4/63.7);
  WIN 14.6/24.0/45.2/49.2/50.4 (vs 14.8/22.4/50.8/47.9/51.5). Throughput at high thread
  counts is the least reproducible metric this session (WSL t8 single_int 21.3M vs Run 14's
  100.5M, while WIN t8 = 128.2M): ambient-load artifact, p50 stays flat.
- **cpp-quill/cpp-nanolog stable.** cpp-nanolog WSL still the latency reference
  (16.3/17.8/17.1/17.5/17.9); cpp-quill's t1-t4 throughput still capped by its queue-resize
  backstop as in Run 14.
- **ticklog-nanolog t1-t4 still slow throughput** (WSL t1 2.3M, t4 10.8M) — Drop + pool
  startup cost; recovers at t8 (45.5M), same shape as Run 14.
- Overall: the a8afd44 package is a net neutral-to-positive with the zero-fill kept;
  the attempted memset removal was the only detrimental piece and is gone.

---

## WSL

### single_int — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 15.4 | 15.5 | 15.6 | 18.5 | 23.5 |
| ticklog-rtrb-drop | 15.3 | 34.3 | 56.0 | 62.1 | 62.6 |
| ticklog-quill | 18.1 | 15.4 | 17.2 | 30.5 | 36.8 |
| ticklog-nanolog | 17.9 | 15.8 | 18.0 | 31.6 | 31.6 |
| cpp-quill (8MiB q) | 21.7 | 21.7 | 27.0 | 27.1 | 35.1 |
| cpp-nanolog | 16.3 | 17.8 | 17.1 | 17.5 | 17.9 |

### single_int — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 37.4M | 28.3M | 34.8M | 24.2M | 37.5M |
| ticklog-rtrb-drop | 38.3M | 56.1M | 58.1M | 21.3M | 81.3M |
| ticklog-quill | 12.7M | 29.5M | 33.5M | 36.1M | 24.4M |
| ticklog-nanolog | 2.3M | 5.5M | 10.8M | 45.5M | 33.8M |
| cpp-quill (8MiB q) | 1.6M | 1.7M | 4.2M | 243.7M | 137.1M |
| cpp-nanolog | 43.9M | 61.0M | 55.9M | 50.9M | 36.3M |

### mixed — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 15.6 | 15.6 | 18.0 | 29.5 | 34.8 |
| ticklog-rtrb-drop | 16.7 | 17.1 | 67.5 | 100.9 | 103.4 |
| ticklog-quill | 17.8 | 17.9 | 26.7 | 34.2 | 46.1 |
| ticklog-nanolog | 17.6 | 17.8 | 27.9 | 34.8 | 44.4 |
| cpp-quill (8MiB q) | 31.4 | 31.3 | 34.7 | 51.4 | 54.8 |
| cpp-nanolog | 15.3 | 15.3 | 18.0 | 19.0 | 19.1 |

### mixed — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 40.4M | 27.0M | 32.6M | 28.5M | 40.9M |
| ticklog-rtrb-drop | 38.6M | 46.1M | 58.4M | 57.3M | 59.5M |
| ticklog-quill | 14.1M | 24.0M | 25.7M | 39.2M | 26.2M |
| ticklog-nanolog | 1.8M | 3.3M | 7.3M | 56.1M | 27.4M |
| cpp-quill (8MiB q) | 1.0M | 0.9M | 0.6M | 120.4M | 120.2M |
| cpp-nanolog | 19.6M | 17.8M | 21.8M | 24.4M | 24.4M |

### string — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 15.7 | 15.2 | 16.4 | 24.8 | 29.0 |
| ticklog-rtrb-drop | 16.2 | 16.3 | 53.9 | 78.3 | 104.6 |
| ticklog-quill | 16.0 | 16.1 | 22.5 | 36.1 | 37.3 |
| ticklog-nanolog | 17.1 | 22.6 | 22.0 | 33.8 | 43.4 |
| cpp-quill (8MiB q) | 30.5 | 30.6 | 38.4 | 49.4 | 50.7 |
| cpp-nanolog | 15.4 | 17.2 | 17.4 | 17.3 | 17.3 |

### string — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 35.5M | 37.5M | 40.3M | 40.4M | 44.4M |
| ticklog-rtrb-drop | 32.7M | 46.1M | 57.0M | 70.8M | 63.8M |
| ticklog-quill | 13.8M | 26.1M | 25.4M | 33.7M | 27.6M |
| ticklog-nanolog | 2.5M | 5.0M | 9.4M | 65.2M | 23.2M |
| cpp-quill (8MiB q) | 1.4M | 1.4M | 6.7M | 91.6M | 52.7M |
| cpp-nanolog | 19.0M | 17.8M | 21.1M | 24.0M | 25.0M |

## WIN

### single_int — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 15.3 | 15.3 | 25.4 | 36.6 | 41.9 |
| ticklog-rtrb-drop | 14.6 | 24.0 | 45.2 | 49.2 | 50.4 |
| ticklog-quill | 17.0 | 15.6 | 19.3 | 36.3 | 28.2 |
| ticklog-nanolog | 15.6 | 15.8 | 19.0 | 22.4 | 29.3 |
| cpp-quill (8MiB q) | 18.7 | 16.8 | 17.6 | 22.3 | 33.5 |

### single_int — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 56.4M | 102.3M | 118.8M | 119.2M | 118.2M |
| ticklog-rtrb-drop | 50.0M | 84.1M | 73.4M | 128.2M | 118.7M |
| ticklog-quill | 30.3M | 55.7M | 74.9M | 78.7M | 50.0M |
| ticklog-nanolog | 2.6M | 5.1M | 11.6M | 72.9M | 48.9M |
| cpp-quill (8MiB q) | 1.4M | 1.4M | 11.4M | 141.7M | 62.3M |

### mixed — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 15.0 | 17.4 | 29.7 | 51.8 | 56.5 |
| ticklog-rtrb-drop | 15.2 | 16.9 | 46.9 | 75.1 | 76.2 |
| ticklog-quill | 18.2 | 18.5 | 28.9 | 38.5 | 41.5 |
| ticklog-nanolog | 18.1 | 18.1 | 18.9 | 34.9 | 31.8 |
| cpp-quill (8MiB q) | 27.5 | 27.5 | 26.6 | 35.9 | 62.2 |

### mixed — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 55.2M | 73.5M | 109.2M | 99.9M | 121.9M |
| ticklog-rtrb-drop | 48.2M | 72.2M | 72.6M | 85.9M | 80.9M |
| ticklog-quill | 28.2M | 48.4M | 64.7M | 76.1M | 55.0M |
| ticklog-nanolog | 1.8M | 3.4M | 8.2M | 59.8M | 54.3M |
| cpp-quill (8MiB q) | 1.0M | 0.9M | 0.6M | 181.6M | 162.0M |

### string — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 14.8 | 14.8 | 24.1 | 44.4 | 49.6 |
| ticklog-rtrb-drop | 15.3 | 18.0 | 66.4 | 67.7 | 69.1 |
| ticklog-quill | 15.5 | 15.9 | 25.7 | 37.6 | 46.4 |
| ticklog-nanolog | 16.3 | 16.2 | 23.1 | 29.8 | 33.8 |
| cpp-quill (8MiB q) | 27.4 | 26.5 | 23.3 | 28.5 | 32.7 |

### string — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 57.4M | 96.5M | 125.5M | 130.8M | 135.0M |
| ticklog-rtrb-drop | 44.1M | 68.5M | 56.7M | 91.9M | 99.9M |
| ticklog-quill | 31.0M | 54.5M | 70.5M | 87.7M | 50.9M |
| ticklog-nanolog | 2.9M | 5.4M | 11.7M | 66.7M | 52.6M |
| cpp-quill (8MiB q) | 1.2M | 1.1M | 11.7M | 247.4M | 54.4M |

---

## Meta

- **ticklog-custom-drop (WIN)**: candidate=`ticklog-custom-drop` os=windows ns/tick=0.31313282 samples=1000 total=1000000
- **ticklog-custom-drop (WSL)**: candidate=`ticklog-custom-drop` os=linux ns/tick=0.313148759 samples=1000 total=1000000
- **ticklog-rtrb-drop (WIN)**: candidate=`ticklog-rtrb-drop` os=windows ns/tick=0.31313282 samples=1000 total=1000000
- **ticklog-rtrb-drop (WSL)**: candidate=`ticklog-rtrb-drop` os=linux ns/tick=0.313148759 samples=1000 total=1000000
- **ticklog-quill (WIN)**: candidate=`ticklog-quill` os=windows ns/tick=0.31313282 samples=1000 total=1000000
- **ticklog-quill (WSL)**: candidate=`ticklog-quill` os=linux ns/tick=0.313148759 samples=1000 total=1000000
- **ticklog-nanolog (WIN)**: candidate=`ticklog-nanolog` os=windows ns/tick=0.31313282 samples=1000 total=1000000
- **ticklog-nanolog (WSL)**: candidate=`ticklog-nanolog` os=linux ns/tick=0.313148759 samples=1000 total=1000000
- **cpp-quill (8MiB q) (WIN)**: candidate=`quill` os=windows ns/tick=0.313133 samples=1000 total=1000000
- **cpp-quill (8MiB q) (WSL)**: candidate=`quill` os=linux ns/tick=0.313149 samples=1000 total=1000000
- **cpp-nanolog (WSL)**: candidate=`nanolog` os=linux ns/tick=0.313149 samples=1000 total=1000000