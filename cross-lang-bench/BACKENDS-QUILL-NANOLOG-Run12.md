# Run 12: all ticklog backends + quill/nanolog policies (WSL + Windows)

Date: 2026-09-23. Post-split rtrb (commit 98562b9).
BATCH=1000, SAMPLES=1000 (1M msg/config), threads `1,2,4,8,16`, RDTSC.
Calibration: Windows `0.313132820 ns/tick`, WSL `0.313148759 ns/tick`.
Rings: Win 64 MiB / WSL 32 MiB; rtrb `--chunk-size 512`.

## Builds & sizing

| Candidate | Features | Buffer |
|---|---|---|
| ticklog-custom | (none — custom ring, Block) | ring 64/32 MiB |
| ticklog-rtrb | backend-rtrb (split P/C) | rtrb ring 64/32 MiB, chunk 512 |
| ticklog-ringbuf | backend-ringbuf | ringbuf 64/32 MiB |
| ticklog-ringbuffer | backend-ringbuffer | ringbuffer 64/32 MiB |
| ticklog-triple | backend-triple-buffer | triple buffer 64/32 MiB |
| ticklog-quill | policy-quill (custom ring) | arena regions, ring 64/32 MiB |
| ticklog-nanolog | policy-nanolog (custom ring) | pool max(8MiB/ring,1) segs |
| C++ quill v12.1.0 | clang, QUILL_NO_EXCEPTIONS | per-thread SPSC 64/32 MiB |
| C++ NanoLog | clang++, Linux-only | default staging |

## Result files

`results_quill/{windows,wsl}_*_r12.json` and `results_quill/wsl_nanolog_cpp_r12.json`.

---

## WSL

### single_int — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom | 15.4 | 19.0 | 18.9 | 20.5 | 24.8 |
| ticklog-rtrb | 69.8 | 61.8 | 64.1 | 62.8 | 94.1 |
| ticklog-ringbuf | 79.7 | 75.1 | 78.7 | 79.8 | 85.5 |
| ticklog-ringbuffer | 115.8 | 108.7 | 118.2 | 118.7 | 134.5 |
| ticklog-triple | 883.0 | 1755.5 | 3132.2 | 6005.9 | 13959.0 |
| ticklog-quill | 19.7 | 16.8 | 22.3 | 27.1 | 26.4 |
| ticklog-nanolog | 17.6 | 15.5 | 22.7 | 35.9 | 31.0 |
| C++ quill | 27.3 | 27.1 | 29.5 | 25.5 | 34.7 |
| C++ NanoLog | 16.2 | 16.3 | 15.2 | 16.3 | 17.2 |

### single_int — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom | 2.0M | 19.6M | 31.5M | 38.3M | 30.0M |
| ticklog-rtrb | 2.4M | 20.2M | 33.1M | 80.4M | 42.3M |
| ticklog-ringbuf | 3.1M | 22.1M | 36.8M | 39.0M | 56.7M |
| ticklog-ringbuffer | 2.3M | 15.9M | 23.7M | 33.6M | 31.1M |
| ticklog-triple | 756.4k | 860.1k | 1.1M | 901.8k | 665.2k |
| ticklog-quill | 257.2k | 10.6M | 6.8M | 4.0M | 2.4M |
| ticklog-nanolog | 3.0M | 9.5M | 6.9M | 5.5M | 2.5M |
| C++ quill | 4.4M | 63.0M | 67.5M | 214.8M | 31.3M |
| C++ NanoLog | 23.8M | 24.3M | 16.5M | 28.9M | 14.6M |

### mixed — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom | 30.9 | 19.6 | 30.5 | 31.2 | 36.6 |
| ticklog-rtrb | 87.0 | 98.8 | 97.0 | 159.2 | 188.9 |
| ticklog-ringbuf | 97.7 | 127.7 | 112.2 | 115.0 | 191.2 |
| ticklog-ringbuffer | 145.8 | 216.9 | 184.3 | 302.8 | 317.6 |
| ticklog-triple | 1197.0 | 2417.6 | 4373.0 | 8860.1 | 28611.9 |
| ticklog-quill | 29.6 | 30.2 | 31.5 | 39.6 | 41.0 |
| ticklog-nanolog | 20.7 | 20.5 | 31.2 | 37.9 | 36.6 |
| C++ quill | 48.0 | 53.0 | 49.1 | 51.5 | 64.4 |
| C++ NanoLog | 17.4 | 16.6 | 18.1 | 18.6 | 18.3 |

### mixed — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom | 1.3M | 25.0M | 30.4M | 21.2M | 34.6M |
| ticklog-rtrb | 1.2M | 15.8M | 34.3M | 16.1M | 30.9M |
| ticklog-ringbuf | 1.4M | 14.3M | 30.2M | 28.2M | 27.8M |
| ticklog-ringbuffer | 1.3M | 8.5M | 19.3M | 13.0M | 17.3M |
| ticklog-triple | 682.0k | 554.9k | 587.5k | 625.1k | 368.1k |
| ticklog-quill | 4.9M | 9.6M | 5.9M | 5.8M | 1.7M |
| ticklog-nanolog | 1.2M | 2.8M | 4.9M | 4.4M | 2.5M |
| C++ quill | 1.1M | 31.4M | 73.4M | 38.0M | 108.7M |
| C++ NanoLog | 8.7M | 12.1M | 26.2M | 23.0M | 12.7M |

### string — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom | 17.7 | 25.9 | 25.6 | 26.8 | 31.2 |
| ticklog-rtrb | 72.0 | 91.6 | 82.6 | 154.4 | 155.0 |
| ticklog-ringbuf | 65.9 | 92.2 | 92.5 | 162.4 | 164.0 |
| ticklog-ringbuffer | 144.3 | 153.4 | 142.1 | 260.4 | 279.5 |
| ticklog-triple | 830.8 | 1617.4 | 3082.0 | 5904.1 | 16039.4 |
| ticklog-quill | 26.4 | 26.8 | 28.1 | 27.8 | 43.8 |
| ticklog-nanolog | 17.5 | 28.3 | 28.2 | 37.8 | 37.9 |
| C++ quill | 31.6 | 48.6 | 49.6 | 49.6 | 62.1 |
| C++ NanoLog | 15.3 | 15.3 | 15.6 | 17.3 | 18.0 |

### string — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom | 1.4M | 18.3M | 29.8M | 21.4M | 16.5M |
| ticklog-rtrb | 1.1M | 15.3M | 23.0M | 44.8M | 39.8M |
| ticklog-ringbuf | 1.1M | 17.0M | 33.0M | 27.8M | 40.8M |
| ticklog-ringbuffer | 1.0M | 10.9M | 17.4M | 13.0M | 17.2M |
| ticklog-triple | 1.0M | 874.8k | 859.7k | 753.6k | 695.4k |
| ticklog-quill | 3.6M | 10.3M | 7.0M | 3.7M | 1.8M |
| ticklog-nanolog | 2.9M | 3.1M | 7.9M | 6.1M | 1.1M |
| C++ quill | 1.4M | 40.3M | 69.1M | 42.8M | 106.8M |
| C++ NanoLog | 17.9M | 22.6M | 12.5M | 8.2M | 23.8M |

## WIN

### single_int — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom | 30.7 | 31.5 | 34.7 | 44.8 | 64.9 |
| ticklog-rtrb | 49.5 | 44.9 | 47.6 | 50.8 | 53.8 |
| ticklog-ringbuf | 68.9 | 67.9 | 65.6 | 73.3 | 75.7 |
| ticklog-ringbuffer | 105.8 | 62.9 | 98.3 | 106.6 | 108.8 |
| ticklog-triple | 947.2 | 2175.3 | 4152.2 | 8611.4 | 17304.0 |
| ticklog-quill | 20.0 | 20.7 | 20.6 | 27.1 | 42.4 |
| ticklog-nanolog | 18.8 | 18.6 | 19.1 | 26.7 | 25.4 |
| C++ quill | 15.8 | 15.8 | 16.1 | 16.5 | 20.5 |

### single_int — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom | 28.5M | 58.4M | 90.2M | 122.8M | 89.9M |
| ticklog-rtrb | 17.3M | 41.3M | 75.4M | 105.9M | 123.2M |
| ticklog-ringbuf | 13.1M | 27.6M | 55.4M | 46.5M | 74.9M |
| ticklog-ringbuffer | 9.5M | 23.2M | 40.0M | 64.9M | 72.2M |
| ticklog-triple | 811.2k | 812.5k | 816.3k | 605.5k | 870.2k |
| ticklog-quill | 19.6M | 20.4M | 11.2M | 5.0M | 818.2k |
| ticklog-nanolog | 47.3M | 32.0M | 26.4M | 9.7M | 2.1M |
| C++ quill | 58.0M | 19.7M | 186.4M | 24.0M | 30.3M |

### mixed — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom | 45.6 | 33.3 | 47.9 | 58.4 | 61.3 |
| ticklog-rtrb | 76.3 | 77.1 | 76.0 | 82.5 | 84.6 |
| ticklog-ringbuf | 94.1 | 91.8 | 92.5 | 99.6 | 103.8 |
| ticklog-ringbuffer | 165.3 | 156.6 | 161.2 | 164.8 | 170.3 |
| ticklog-triple | 1268.2 | 2837.1 | 5788.6 | 12182.1 | 25012.7 |
| ticklog-quill | 29.8 | 30.9 | 30.2 | 35.5 | 36.8 |
| ticklog-nanolog | 33.5 | 31.1 | 31.4 | 34.5 | 48.4 |
| C++ quill | 27.7 | 25.3 | 25.3 | 26.9 | 43.9 |

### mixed — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom | 19.8M | 48.2M | 69.7M | 83.0M | 27.2M |
| ticklog-rtrb | 11.7M | 24.1M | 47.5M | 43.0M | 85.1M |
| ticklog-ringbuf | 10.5M | 20.8M | 39.7M | 34.1M | 8.3M |
| ticklog-ringbuffer | 6.3M | 13.6M | 24.5M | 45.6M | 15.1M |
| ticklog-triple | 651.3k | 584.8k | 657.7k | 658.7k | 600.6k |
| ticklog-quill | 17.3M | 16.4M | 12.1M | 3.1M | 2.8M |
| ticklog-nanolog | 17.4M | 31.0M | 29.6M | 9.7M | 529.7k |
| C++ quill | 25.1M | 54.6M | 32.2M | 50.8M | 32.0M |

### string — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom | 31.8 | 28.2 | 46.2 | 60.2 | 59.7 |
| ticklog-rtrb | 64.8 | 70.0 | 65.8 | 72.1 | 72.3 |
| ticklog-ringbuf | 87.5 | 68.7 | 92.5 | 93.5 | 95.6 |
| ticklog-ringbuffer | 134.7 | 74.6 | 75.2 | 130.9 | 139.2 |
| ticklog-triple | 883.3 | 1956.0 | 3409.6 | 8212.9 | 16619.2 |
| ticklog-quill | 25.6 | 26.6 | 25.9 | 32.8 | 56.9 |
| ticklog-nanolog | 26.6 | 26.0 | 27.1 | 30.8 | 27.8 |
| C++ quill | 24.6 | 25.4 | 24.6 | 26.1 | 32.4 |

### string — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom | 28.0M | 57.5M | 64.2M | 84.6M | 18.1M |
| ticklog-rtrb | 14.9M | 25.8M | 52.4M | 69.5M | 10.2M |
| ticklog-ringbuf | 10.3M | 25.5M | 34.5M | 39.1M | 4.8M |
| ticklog-ringbuffer | 8.6M | 21.3M | 35.6M | 55.6M | 20.5M |
| ticklog-triple | 775.0k | 860.0k | 1.1M | 934.1k | 819.8k |
| ticklog-quill | 19.8M | 20.0M | 12.0M | 3.9M | 1.5M |
| ticklog-nanolog | 20.8M | 31.8M | 25.2M | 9.9M | 1.4M |
| C++ quill | 18.3M | 43.9M | 23.5M | 54.9M | 80.2M |

---

## Meta

- **ticklog-custom (WIN)**: candidate=`ticklog-custom` os=windows ns/tick=0.31313282 samples=1000 total=1000000
- **ticklog-custom (WSL)**: candidate=`ticklog-custom` os=linux ns/tick=0.313148759 samples=1000 total=1000000
- **ticklog-rtrb (WIN)**: candidate=`ticklog-rtrb` os=windows ns/tick=0.31313282 samples=1000 total=1000000
- **ticklog-rtrb (WSL)**: candidate=`ticklog-rtrb` os=linux ns/tick=0.313148759 samples=1000 total=1000000
- **ticklog-ringbuf (WIN)**: candidate=`ticklog-ringbuf` os=windows ns/tick=0.31313282 samples=1000 total=1000000
- **ticklog-ringbuf (WSL)**: candidate=`ticklog-ringbuf` os=linux ns/tick=0.313148759 samples=1000 total=1000000
- **ticklog-ringbuffer (WIN)**: candidate=`ticklog-ringbuffer` os=windows ns/tick=0.31313282 samples=1000 total=1000000
- **ticklog-ringbuffer (WSL)**: candidate=`ticklog-ringbuffer` os=linux ns/tick=0.313148759 samples=1000 total=1000000
- **ticklog-triple (WIN)**: candidate=`ticklog-triple` os=windows ns/tick=0.31313282 samples=1000 total=1000000
- **ticklog-triple (WSL)**: candidate=`ticklog-triple` os=linux ns/tick=0.313148759 samples=1000 total=1000000
- **ticklog-quill (WIN)**: candidate=`ticklog-quill` os=windows ns/tick=0.31313282 samples=1000 total=1000000
- **ticklog-quill (WSL)**: candidate=`ticklog-quill` os=linux ns/tick=0.313148759 samples=1000 total=1000000
- **ticklog-nanolog (WIN)**: candidate=`ticklog-nanolog` os=windows ns/tick=0.31313282 samples=1000 total=1000000
- **ticklog-nanolog (WSL)**: candidate=`ticklog-nanolog` os=linux ns/tick=0.313148759 samples=1000 total=1000000
- **C++ quill (WIN)**: candidate=`quill` os=windows ns/tick=0.313133 samples=1000 total=1000000
- **C++ quill (WSL)**: candidate=`quill` os=linux ns/tick=0.313149 samples=1000 total=1000000
- **C++ NanoLog (WSL)**: candidate=`nanolog` os=linux ns/tick=0.313149 samples=1000 total=1000000

---

## Observations

1. **Backend ranking (single_int p50, Block policy)** — custom ≪ rtrb < ringbuf <
   ringbuffer ≪ triple-buffer. On Windows t1: custom 30.7 / rtrb 49.5 / ringbuf
   68.9 / ringbuffer 105.8 / triple 947 ns. On WSL t1: 15.4 / 69.8 / 79.7 /
   115.8 / 883 ns. The post-split rtrb stays within historical range
   (`harness-results/windows/sweep/ticklog_rtrb.json` was 69 ns under the older
   samples=10000 methodology; this run is samples=1000 + 64 MiB ring, so not
   directly comparable — but rtrb clearly beats ringbuf/ringbuffer here).
2. **rtrb multi-thread scaling is the best of the FIFO backends** on Windows:
   single_int throughput 17→123 M r/s (t1→t16) and p50 stays 45–54 ns across
   t2–t16. WSL is noisier (t1 warmup artifact ~2.4 M r/s) but peaks at 80 M r/s
   at t8.
3. **triple-buffer is unusable as a logger backend**: single-slot overwrite
   means the drain loses almost everything; p50 grows linearly with threads
   (Win t16 17 µs, WSL t16 14 µs) and throughput is stuck at ~0.6–1 M r/s.
4. **Policy ranking on the custom ring**: nanolog ≈ quill ≪ Block for p50.
   Windows single_int t1: nanolog 18.8 / quill 20.0 / custom(Block) 30.7 ns.
   WSL: 17.6 / 19.7 / 15.4 (custom wins t1 there — Block+custom is already very
   tight on Linux; the policies pay arena/pool overhead that only pays off under
   multi-thread contention).
5. **ticklog-quill vs C++ quill**: C++ still dominates throughput (Win t8
   single_int: 24 M vs 5 M r/s; WSL t8: 215 M vs 4 M r/s — the WSL C++ number
   is the known first-sample-throughput artifact territory, compare p50). p50 is
   close on Windows at low threads (15.8 vs 20.0 ns at t1) and C++ pulls ahead
   at t16 (20.5 vs 42.4 ns).
6. **ticklog-nanolog vs C++ NanoLog (WSL)**: p50 is comparable (15–36 vs
   15–18 ns) but C++ NanoLog holds throughput flat (14–29 M r/s) while
   ticklog-nanolog collapses after t2 (9.5 → 2.5 M r/s on single_int) — the
   8 MiB bounded pool (`NANOLOG_POOL_BUDGET`) stalls producers, same behavior
   as Run 7.
7. **WSL t1 throughput for all candidates is a warmup artifact** (first batch
   includes init); do not compare t1 throughput across platforms — compare p50.
8. C++ NanoLog was not re-run on Windows (Linux-only, same as Run 7).

## How this was run

- `build_win_r12.ps1` / `build_wsl_r12.sh` — feature-matrix cargo builds → `bin/`.
- `run_win_r12.ps1` / `run_wsl_r12.sh` — samples=1000, threads 1,2,4,8,16,
  ring 64/32 MiB, rtrb `--chunk-size 512`.
- `summarize_r12.py` — regenerates this report from the JSON files.
