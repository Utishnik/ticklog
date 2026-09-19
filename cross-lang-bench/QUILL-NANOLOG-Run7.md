# Run 7: ticklog-Quill vs C++ Quill vs C++ NanoLog (WSL + Windows)

Date: 2026-09-19. BATCH=1000, SAMPLES=1000 (1M msg/config), threads `1,2,4,8,16`,
RDTSC. Calibration: Windows `0.313132820 ns/tick`, WSL `0.313148759 ns/tick`.

## Builds & sizing

| Candidate | Compiler | Flags | Buffer |
|---|---|---|---|
| C++ quill v12.1.0 | clang 23.1.1 (Win) / clang++-22 (WSL) | `-march=native -flto=thin -fno-exceptions -fno-rtti -fomit-frame-pointer`, `QUILL_NO_EXCEPTIONS=ON` | per-thread SPSC queue 64 MiB (Win) / 32 MiB (WSL) |
| C++ NanoLog | clang++-22 (WSL; Linux-only) | runtime + harness `-O3 -march=native -flto=thin -fomit-frame-pointer` | default staging buffers |
| ticklog-Quill (Rust) | rustc 1.x, `opt-level=3 lto=fat codegen-units=1` | `--features policy-quill` | ring 64 MiB (Win) / 32 MiB (WSL) |
| ticklog-NanoLog (Rust) | same profile | `--features policy-nanolog` | 8 × 1 MiB pool, producers block |

Notes:
- C++ NanoLog is **Linux-only** (uses Linux-specific runtime); on Windows it is
  represented by ticklog's Rust NanoLog policy.
- Ring/queue sizes are at the platform's RAM limit: WSL has ~5 GiB available, so
  32 MiB; Windows ~7 GiB, so 64 MiB.
- NTFS is case-insensitive, so `cpp/NanoLog` and `cpp/nanolog` collide on the WSL
  mount; the NanoLog runtime lives at `~/NanoLog` on the WSL filesystem and is
  passed to CMake via `-DNANOLOG_RUNTIME_DIR`.

## Result files

`results_quill/{windows,wsl}_{quill_cpp,ticklog_quill,ticklog_nanolog}_r7.json`
and `results_quill/wsl_nanolog_cpp_r7.json`.

---

## WSL (32 MiB rings/queues)

### single_int — p50 latency (ns) [throughput r/s]

| threads | ticklog-Quill | C++ quill | C++ NanoLog | ticklog-NanoLog |
|---|---|---|---|---|
| 1  | 20.42 [4.8M] | 15.18 [5.0M*] | 15.20 [52.8M] | 19.79 [4.1M] |
| 2  | 27.98 [10.1M] | 16.88 [108.6M] | 15.19 [59.0M] | 19.68 [5.1M] |
| 4  | 36.17 [13.5M] | 15.25 [229.8M] | 16.28 [48.3M] | 26.46 [7.3M] |
| 8  | 72.93 [12.2M] | 21.94 [272.0M] | 16.49 [42.9M] | 30.58 [12.0M] |
| 16 | 107.93 [7.8M] | 29.84 [75.7M] | 17.65 [22.9M] | 36.63 [13.0M] |

\* t1 throughput artifact (first-batch/init effect), see notes.

### mixed — p50 latency (ns) [throughput r/s]

| threads | ticklog-Quill | C++ quill | C++ NanoLog | ticklog-NanoLog |
|---|---|---|---|---|
| 1  | 30.34 [4.4M] | 16.97 [1.0M*] | 15.19 [38.2M] | 38.01 [2.4M] |
| 2  | 27.65 [9.8M] | 29.60 [65.0M] | 15.17 [27.9M] | 27.56 [3.3M] |
| 4  | 43.01 [13.4M] | 22.09 [133.5M] | 16.20 [25.7M] | 31.67 [5.1M] |
| 8  | 74.36 [12.4M] | 32.30 [180.2M] | 19.35 [20.4M] | 40.90 [7.6M] |
| 16 | 121.30 [7.7M] | 38.14 [107.3M] | 19.46 [19.9M] | 41.10 [8.9M] |

### string — p50 latency (ns) [throughput r/s]

| threads | ticklog-Quill | C++ quill | C++ NanoLog | ticklog-NanoLog |
|---|---|---|---|---|
| 1  | 21.51 [4.7M] | 16.57 [1.5M*] | 15.09 [42.2M] | 35.71 [3.5M] |
| 2  | 24.06 [8.3M] | 17.18 [81.0M] | 17.22 [40.1M] | 21.93 [5.2M] |
| 4  | 36.78 [13.8M] | 16.80 [161.3M] | 17.22 [42.8M] | 22.55 [7.4M] |
| 8  | 73.25 [12.4M] | 27.41 [238.8M] | 17.31 [29.5M] | 37.41 [11.9M] |
| 16 | 86.45 [6.7M] | 36.10 [50.0M] | 17.32 [18.3M] | 40.21 [12.5M] |

---

## Windows (64 MiB rings/queues)

C++ NanoLog unavailable (Linux-only) → ticklog-NanoLog only.

### single_int — p50 latency (ns) [throughput r/s]

| threads | ticklog-Quill | C++ quill | ticklog-NanoLog |
|---|---|---|---|
| 1  | 20.20 [22.2M] | 14.90 [61.6M] | 17.95 [4.1M] |
| 2  | 28.91 [37.7M] | 14.92 [128.5M] | 18.10 [5.3M] |
| 4  | 38.35 [39.0M] | 20.20 [181.4M] | 18.51 [7.9M] |
| 8  | 74.18 [30.5M] | 23.18 [291.6M] | 30.35 [10.0M] |
| 16 | 67.74 [16.7M] | 40.03 [237.4M] | 33.92 [16.1M] |

### mixed — p50 latency (ns) [throughput r/s]

| threads | ticklog-Quill | C++ quill | ticklog-NanoLog |
|---|---|---|---|
| 1  | 24.68 [20.8M] | 16.89 [48.3M] | 25.18 [2.4M] |
| 2  | 35.49 [32.8M] | 28.82 [66.7M] | 34.31 [3.4M] |
| 4  | 42.95 [37.7M] | 23.75 [131.3M] | 32.31 [5.4M] |
| 8  | 72.87 [28.1M] | 36.30 [184.2M] | 40.54 [7.5M] |
| 16 | 83.11 [15.6M] | 57.95 [57.3M] | 41.65 [11.0M] |

### string — p50 latency (ns) [throughput r/s]

| threads | ticklog-Quill | C++ quill | ticklog-NanoLog |
|---|---|---|---|
| 1  | 22.84 [21.5M] | 15.90 [60.7M] | 21.22 [4.3M] |
| 2  | 25.21 [36.9M] | 15.96 [114.4M] | 21.85 [5.2M] |
| 4  | 37.20 [38.9M] | 23.17 [144.7M] | 27.54 [7.5M] |
| 8  | 73.97 [27.9M] | 29.23 [221.7M] | 38.31 [10.4M] |
| 16 | 104.83 [15.1M] | 42.96 [120.8M] | 40.54 [13.6M] |

---

## Δ ticklog-Quill vs C++ quill, p50 (×)

| threads | WSL single_int | WSL mixed | Win single_int | Win mixed |
|---|---|---|---|---|
| 1  | 1.35× | 1.79× | 1.36× | 1.46× |
| 2  | 1.66× | 0.93× | 1.94× | 1.23× |
| 4  | 2.37× | 1.95× | 1.90× | 1.81× |
| 8  | 3.32× | 2.30× | 3.20× | 2.01× |
| 16 | 3.62× | 3.18× | 1.69× | 1.43× |

## Δ throughput vs C++ quill (×)

| threads | WSL single_int | WSL mixed | Win single_int | Win mixed |
|---|---|---|---|---|
| 1  | ~1×* | ~4×* | 0.36× | 0.43× |
| 4  | 0.06× | 0.10× | 0.22× | 0.29× |
| 8  | 0.045× | 0.07× | 0.10× | 0.15× |
| 16 | 0.10× | 0.07× | 0.07× | 0.27× |

\* t1 WSL quill run is unusable for throughput (first-sample artifact).

---

## Observations

1. **C++ NanoLog has the best raw call-site p50** (15–19 ns on WSL at every thread
   count) — even better than C++ quill at t16. But its bounded staging (8 log
   blocks) makes tails explode under multi-thread load: WSL t16 `mixed` p99 =
   23.8 µs, p999 = 27.6 µs; throughput caps at ~20–59M r/s and *drops* as threads
   rise. It is a latency-optimal, bounded-throughput design.
2. **ticklog's Rust NanoLog policy inherits the same block-bound behavior** on
   both platforms: p50 stays 18–42 ns but p95 jumps to 3–12 µs at ≥2 threads
   (producers waiting on the 8 MiB pool), throughput peaks ~13–16M r/s at t16.
3. **C++ quill remains the throughput king**, especially on WSL: single_int
   272M r/s at t8, mixed 180M, vs ticklog-Quill's 12–14M. On Windows the gap is
   ~3× in p50 and 10–15× in throughput at 8 threads.
4. **ticklog-Quill vs C++ quill**: p50 1.4–3.6× higher depending on thread count;
   WSL t16 mixed quill 38 ns vs ticklog 121 ns. The cross-core `tail.load` +
   `head.store` per record and the arena handoff path (described in Run6) remain
   the gap; growing the ring 32→64 MiB removes most handoff stalls on Windows but
   cannot fix the per-record sync cost.
5. Same-day Windows check: C++ quill 16-thread single_int is healthy this run
   (237M r/s, p999 157 ns; Run6 had a 10M collapse).
6. WSL 1-thread throughput numbers for C++ quill and ticklog-Quill are
   artifacts of the first-sample warmup/init cost (huge p999) — compare p50 there,
   not throughput.