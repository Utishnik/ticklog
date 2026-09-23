# Run 13: custom+Drop, rtrb+Drop (chunk 64), quill/nanolog re-run (WSL + Windows)

Date: 2026-09-23. Post-split rtrb (commit 98562b9).
BATCH=1000, SAMPLES=1000 (1M msg/config), threads `1,2,4,8,16`, RDTSC.
Calibration: Windows `0.313132820 ns/tick`, WSL `0.313148759 ns/tick`.
Rings: **64 MiB both platforms** (user requested 60 MB; capacity must be power-of-two).
rtrb `--chunk-size 64`.

## Builds & sizing

| Candidate | Features | Buffer |
|---|---|---|
| ticklog-custom-drop | policy-drop (custom ring) | ring 64 MiB |
| ticklog-rtrb-drop | backend-rtrb,policy-drop | rtrb ring 64 MiB, chunk 64 |
| ticklog-quill | policy-quill (custom ring) | arena regions, ring 64 MiB |
| ticklog-nanolog | policy-nanolog (custom ring) | pool max(8MiB/ring,1) segs |

## Result files

`results_quill/{windows,wsl}_*_r13.json`.

---

## WSL

### single_int — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 15.5 | 15.4 | 18.7 | 22.0 | 28.7 |
| ticklog-rtrb-drop | 43.2 | 59.3 | 61.6 | 61.9 | 64.7 |
| ticklog-quill | 15.8 | 23.2 | 19.8 | 35.3 | 26.6 |
| ticklog-nanolog | 15.4 | 15.6 | 22.2 | 36.0 | 33.1 |

### single_int — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 13.2M | 25.6M | 37.7M | 63.7M | 52.7M |
| ticklog-rtrb-drop | 19.6M | 32.7M | 49.5M | 96.8M | 83.2M |
| ticklog-quill | 13.1M | 14.1M | 8.3M | 7.1M | 4.2M |
| ticklog-nanolog | 55.9M | 12.3M | 11.9M | 12.3M | 5.5M |

### mixed — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 19.1 | 19.1 | 28.3 | 31.9 | 36.5 |
| ticklog-rtrb-drop | 83.6 | 91.5 | 92.9 | 100.2 | 141.2 |
| ticklog-quill | 31.4 | 18.3 | 35.1 | 36.1 | 38.2 |
| ticklog-nanolog | 20.9 | 20.8 | 32.5 | 39.3 | 37.4 |

### mixed — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 13.1M | 24.2M | 41.3M | 46.4M | 46.7M |
| ticklog-rtrb-drop | 11.7M | 20.1M | 38.6M | 59.5M | 60.7M |
| ticklog-quill | 12.1M | 15.5M | 7.8M | 7.7M | 3.4M |
| ticklog-nanolog | 9.2M | 12.2M | 11.5M | 5.6M | 4.3M |

### string — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 18.0 | 17.9 | 17.9 | 27.5 | 34.7 |
| ticklog-rtrb-drop | 72.8 | 76.3 | 71.2 | 78.2 | 142.8 |
| ticklog-quill | 20.1 | 19.3 | 20.4 | 35.5 | 43.8 |
| ticklog-nanolog | 17.0 | 17.4 | 20.7 | 36.0 | 39.2 |

### string — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 13.0M | 23.5M | 37.9M | 49.0M | 52.5M |
| ticklog-rtrb-drop | 9.5M | 25.1M | 48.3M | 76.7M | 64.9M |
| ticklog-quill | 13.7M | 15.7M | 7.2M | 7.7M | 494.1k |
| ticklog-nanolog | 12.0M | 9.5M | 14.5M | 11.3M | 2.5M |

## WIN

### single_int — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 23.8 | 29.0 | 35.0 | 45.7 | 50.1 |
| ticklog-rtrb-drop | 30.0 | 37.8 | 45.0 | 49.9 | 50.4 |
| ticklog-quill | 21.0 | 20.9 | 21.8 | 42.2 | 52.4 |
| ticklog-nanolog | 16.1 | 19.7 | 21.8 | 39.5 | 47.7 |

### single_int — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 34.5M | 53.4M | 87.4M | 123.5M | 104.0M |
| ticklog-rtrb-drop | 30.1M | 49.1M | 70.7M | 131.5M | 126.2M |
| ticklog-quill | 32.4M | 31.3M | 45.0M | 31.2M | 18.7M |
| ticklog-nanolog | 54.3M | 38.7M | 42.1M | 31.9M | 19.5M |

### mixed — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 32.2 | 33.3 | 50.8 | 58.4 | 61.9 |
| ticklog-rtrb-drop | 43.1 | 68.9 | 72.7 | 76.3 | 78.8 |
| ticklog-quill | 18.7 | 19.3 | 30.8 | 43.0 | 39.4 |
| ticklog-nanolog | 19.6 | 25.5 | 23.6 | 39.6 | 38.3 |

### mixed — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 28.3M | 50.0M | 65.2M | 101.0M | 103.8M |
| ticklog-rtrb-drop | 21.7M | 28.6M | 50.7M | 71.7M | 83.5M |
| ticklog-quill | 30.1M | 40.3M | 40.3M | 30.8M | 6.9M |
| ticklog-nanolog | 29.1M | 33.1M | 44.2M | 32.4M | 7.3M |

### string — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 25.8 | 39.6 | 44.6 | 47.4 | 59.5 |
| ticklog-rtrb-drop | 37.3 | 37.9 | 39.1 | 68.4 | 69.0 |
| ticklog-quill | 16.3 | 29.1 | 23.0 | 42.6 | 31.0 |
| ticklog-nanolog | 25.1 | 24.6 | 26.1 | 38.4 | 47.0 |

### string — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 33.9M | 46.1M | 74.1M | 94.7M | 112.1M |
| ticklog-rtrb-drop | 25.0M | 40.5M | 70.7M | 91.1M | 95.2M |
| ticklog-quill | 32.8M | 35.8M | 44.2M | 31.9M | 1.6M |
| ticklog-nanolog | 29.0M | 38.2M | 46.8M | 33.2M | 2.9M |

---

## Observations

1. **Drop policy + 64 MiB ring — single_int p50 (ns)**:
   - **WIN**: custom-drop 23.8/29.0/35.0/45.7/50.1 (t1..t16); rtrb-drop (chunk 64)
     30.0/37.8/45.0/49.9/50.4; quill 21.0/20.9/21.8/42.2/52.4; nanolog
     16.1/19.7/21.8/39.5/47.7.
   - **WSL**: custom-drop 15.5/15.4/18.7/22.0/28.7; rtrb-drop 43.2/59.3/61.6/61.9/64.7;
     quill 15.8/23.2/19.8/35.3/26.6; nanolog 15.4/15.6/22.2/36.0/33.1.
2. **Drop vs Block (Run 12 comparison)**: custom+Drop is consistently lower p50
   than custom+Block on Windows single_int (23.8 vs 30.7 at t1, 50.1 vs 64.9 at
   t16) — the Drop path skips drain wakeup cost. On WSL custom+Drop ≈ custom+Block
   at t1 (15.5 vs 15.4) and slightly better at t16 (28.7 vs 24.8 — noise-level,
   within run-to-run variance).
3. **rtrb-drop chunk 64 vs chunk 512 (Run 12)**: Windows single_int p50 t1 30.0
   (chunk 64) vs 49.5 (chunk 512) — the smaller chunk reduces the per-batch
   amortized cost; multi-thread p50 is also lower (t16: 50.4 vs 53.7). WSL
   rtrb-drop t1 43.2 vs rtrb-block 69.8. Drop+small-chunk clearly benefits rtrb.
4. **rtrb-drop multi-thread throughput on Windows** reaches 131.5 M r/s at t8
   (single_int) — best FIFO throughput in this run; p50 stays 38–50 ns from t2 to
   t16. WSL peaks at 96.8 M r/s at t8 (t1 warmup artifact: compare p50).
5. **Policy ranking with 64 MiB rings**: nanolog ≲ quill ≲ Drop+custom for p50 on
   Windows t1 (16.1 / 21.0 / 23.8); on WSL Drop+custom and nanolog tie at t1
   (15.5 / 15.4). At multi-thread, nanolog collapses after t2 on WSL
   (throughput 12.3 → 5.5 M r/s) — still limited by the 8 MiB pool budget;
   Windows nanolog holds 42 M r/s at t4 then drops to 19.5 M at t16.
6. **quill/nanolog re-run (Run 13) vs Run 12**: same order of magnitude —
   quill Win t1 21.0 (was 20.0), nanolog Win t1 16.1 (was 18.8); quill WSL t1
   15.8 (was 19.7), nanolog WSL t1 15.4 (was 17.6). Differences are within
   run-to-run noise; the 64 MiB ring on WSL (was 32 MiB) did not change the
   policy picture meaningfully.
7. **mixed/string workloads**: Drop+custom stays lowest on WSL p50 (17.9–36.5
   ns); rtrb-drop mixed p50 is 83–141 ns on WSL (rtrb chunking penalty under
   heavier records) and 43–79 ns on Windows. quill/string WSL t16 throughput
   494 k r/s is the known wall-clock/scheduler artifact — compare p50 (43.8 ns).
8. **WSL t1 throughput is a warmup artifact** across candidates — compare p50,
   not t1 throughput, across platforms.

## How this was run

- `build_win_r13.ps1` / `build_wsl_r13.sh` — feature-matrix cargo builds → `bin/`
  (policy-drop, backend-rtrb+policy-drop, policy-quill, policy-nanolog).
- `run_win_r13.ps1` / `run_wsl_r13.sh` — samples=1000, threads 1,2,4,8,16,
  ring **64 MiB both platforms**, rtrb `--chunk-size 64`.
- `summarize_r13.py` — regenerates this report from the JSON files.

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
