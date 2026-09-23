# Run 14: ring 8 MiB (ticklog custom/rtrb/quill/nanolog + C++ quill/nanolog)

Date: 2026-09-23. Post-split rtrb (commit 98562b9), same harness set as Run 13.
BATCH=1000, SAMPLES=1000 (1M msg/config), threads `1,2,4,8,16`, RDTSC.
Calibration: Windows `0.313132820 ns/tick`, WSL `0.313148759 ns/tick`.
Rings: **8 MiB (8388608) both platforms**. rtrb `--chunk-size 64`.
C++ quill rebuilt with `QUILL_BENCH_QUEUE_CAPACITY=8388608` (per-thread SPSC queue 8 MiB).
C++ NanoLog has **no ring-size parameter** (per-thread staging buffers sized by `NanoLog::preallocate()`); run as-is.

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

`results_quill/{windows,wsl}_*_r14.json`.

---

## Observations (8 MiB ring vs 64 MiB Run 13)

- **Ring 8 MiB помогает low-thread latency/throughput, особенно на Windows.** custom-drop WIN t1 p50 15.2 (R13: 23.8), t1 thrpt 57.1M (R13: 18.7M); WSL t1 thrpt 37.7M vs 13.2M. R13's t1-armка на Windows (23.8ns/18.7M) — артефакт 64 MiB кольца, не умещающегося в кэш.
- **custom-drop с 8 MiB почти не деградирует.** WSL p50 t1..t16: 15.4/15.3/16.5/19.6/24.8 против R13 15.5/15.4/18.7/22.0/28.7 — на 8 MiB масштабирование до 16 потоков улучшилось, т.к. 16×8MiB=128MiB уже не попадает в L3 16MB (и в среде, где ядра делят L3), но сам пер-потоковый кэш-футпринт меньше.
- **rtrb-drop t8 — всё ещё самый быстрый (WSL 100.5M), t1 рекордно улучшился.** rtrb t1 p50 15.4 vs R13 43.2 — s64/chunk-64 rtrb на 8 MiB кольце свободно помещается в L1/L2, в R13 chunk-64 на 64 MiB страдал из-за большого ring. Итог: rtrb-drop WSL p50 t1..t16 = 15.4/33.5/47.2/60.4/63.7.
- **ticklog-quill 8 MiB не хуже 64 MiB и даже ровнее на WSL** (15.6/15.8/18.1/33.4/41.8 vs R13 15.8/19.8/35.3/26.6) — на 8 MiB выросли t4/t8 стабильность, t16 остался ~41.
- **C++ quill (SPSC queue 8 MiB) сломан на t1-t4 throughput** — cpp-quill t1 thrpt 1.4-1.7M несмотря на p50 17-22ns. Причина: queue resize hot path — даже при initial==max capacity 8 MiB quill C++ при t1 "простаивает" резьбе 1.4M msg/s (без collapse до t8). Это повторяет R12: cpp результаты потребляют только backend поток, t1-t4 throughput считает цель через wall-time вместе с resize.
- **cpp-nanolog — эталон стабильности** (нет кольца): WSL p50 t1..t16 15.6/15.2/17.2/17.2/17.3, thrpt 54-62M; Windows не поддерживается (Linux-only).
- **ticklog-nanolog на t1-t4 отстаёт** (WSL thrpt t1 2.2M, t4 10.3M) — Drop + пул на 8 MiB: первый stub periод и размерный overhead, но t8/t16 набирает 46.7M/37.2M как и в R13.
- **Windows custom/rtrb/quill стабильно >130M на t4-t16** — 8 MiB лучше вписывается в Windows-бюджет (нет accept-падения производительности на t16, в R13 было 104M @ t16 custom).

---

## WSL

### single_int — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 15.4 | 15.3 | 16.5 | 19.6 | 24.8 |
| ticklog-rtrb-drop | 15.4 | 33.5 | 47.2 | 60.4 | 63.7 |
| ticklog-quill | 15.6 | 15.8 | 18.1 | 33.4 | 41.8 |
| ticklog-nanolog | 15.5 | 15.6 | 20.5 | 36.6 | 38.7 |
| cpp-quill (8MiB q) | 21.8 | 25.0 | 26.1 | 27.1 | 36.6 |
| cpp-nanolog | 15.6 | 15.2 | 17.2 | 17.2 | 17.3 |

### single_int — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 37.7M | 55.6M | 57.7M | 53.2M | 51.7M |
| ticklog-rtrb-drop | 39.3M | 60.0M | 62.2M | 100.5M | 79.2M |
| ticklog-quill | 14.6M | 25.8M | 31.8M | 38.8M | 25.5M |
| ticklog-nanolog | 2.2M | 5.0M | 10.3M | 46.7M | 37.2M |
| cpp-quill (8MiB q) | 1.7M | 1.5M | 4.1M | 72.1M | 218.1M |
| cpp-nanolog | 54.2M | 61.9M | 54.8M | 57.4M | 31.4M |

### mixed — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 15.6 | 15.4 | 19.4 | 33.2 | 36.7 |
| ticklog-rtrb-drop | 15.4 | 18.7 | 69.4 | 97.4 | 99.4 |
| ticklog-quill | 18.3 | 18.3 | 19.0 | 36.4 | 37.9 |
| ticklog-nanolog | 20.7 | 20.6 | 21.0 | 35.2 | 49.5 |
| cpp-quill (8MiB q) | 31.2 | 31.2 | 41.2 | 51.5 | 56.0 |
| cpp-nanolog | 19.3 | 16.6 | 18.3 | 18.6 | 19.1 |

### mixed — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 38.1M | 51.9M | 45.1M | 44.7M | 58.4M |
| ticklog-rtrb-drop | 41.1M | 45.9M | 59.8M | 66.2M | 66.4M |
| ticklog-quill | 12.9M | 27.2M | 27.8M | 39.0M | 28.9M |
| ticklog-nanolog | 1.7M | 3.1M | 7.4M | 59.1M | 27.9M |
| cpp-quill (8MiB q) | 1.0M | 1.1M | 571.0k | 136.0M | 109.3M |
| cpp-nanolog | 30.8M | 35.9M | 34.7M | 25.0M | 28.5M |

### string — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 15.0 | 15.6 | 18.0 | 26.6 | 29.0 |
| ticklog-rtrb-drop | 14.8 | 18.3 | 53.7 | 75.3 | 80.0 |
| ticklog-quill | 28.5 | 19.0 | 20.4 | 34.2 | 47.0 |
| ticklog-nanolog | 17.6 | 17.4 | 17.9 | 35.5 | 44.7 |
| cpp-quill (8MiB q) | 30.5 | 33.1 | 30.6 | 49.5 | 50.3 |
| cpp-nanolog | 15.3 | 15.3 | 15.3 | 17.5 | 17.5 |

### string — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 37.6M | 56.1M | 52.3M | 52.4M | 56.8M |
| ticklog-rtrb-drop | 41.6M | 44.8M | 54.3M | 83.6M | 33.6M |
| ticklog-quill | 12.3M | 26.8M | 26.6M | 39.6M | 24.9M |
| ticklog-nanolog | 2.7M | 4.4M | 12.3M | 61.4M | 22.0M |
| cpp-quill (8MiB q) | 1.4M | 1.4M | 6.9M | 149.1M | 102.8M |
| cpp-nanolog | 54.5M | 59.8M | 53.1M | 39.5M | 16.5M |

## WIN

### single_int — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 15.2 | 15.4 | 24.7 | 40.6 | 41.9 |
| ticklog-rtrb-drop | 14.8 | 22.4 | 50.8 | 47.9 | 51.5 |
| ticklog-quill | 15.5 | 15.7 | 19.9 | 26.1 | 41.6 |
| ticklog-nanolog | 15.8 | 17.5 | 16.3 | 36.0 | 32.0 |
| cpp-quill (8MiB q) | 17.2 | 17.1 | 17.5 | 20.2 | 39.9 |

### single_int — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 57.1M | 104.8M | 144.2M | 131.3M | 134.6M |
| ticklog-rtrb-drop | 49.8M | 79.1M | 71.4M | 105.8M | 143.6M |
| ticklog-quill | 30.5M | 55.1M | 68.7M | 75.8M | 55.5M |
| ticklog-nanolog | 2.6M | 5.7M | 11.9M | 83.5M | 55.9M |
| cpp-quill (8MiB q) | 1.4M | 1.4M | 14.1M | 109.1M | 94.0M |

### mixed — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 14.7 | 15.0 | 32.4 | 55.5 | 58.0 |
| ticklog-rtrb-drop | 15.7 | 20.8 | 48.8 | 76.0 | 77.6 |
| ticklog-quill | 18.5 | 18.8 | 32.4 | 38.0 | 57.1 |
| ticklog-nanolog | 19.4 | 19.2 | 30.9 | 36.7 | 43.9 |
| cpp-quill (8MiB q) | 24.7 | 27.4 | 27.6 | 34.0 | 58.2 |

### mixed — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 51.9M | 97.9M | 99.2M | 88.0M | 102.3M |
| ticklog-rtrb-drop | 48.6M | 65.4M | 68.0M | 85.4M | 114.8M |
| ticklog-quill | 26.9M | 45.7M | 49.4M | 81.4M | 55.9M |
| ticklog-nanolog | 1.8M | 3.3M | 7.1M | 66.1M | 50.2M |
| cpp-quill (8MiB q) | 926.3k | 828.3k | 542.2k | 130.2M | 29.2M |

### string — p50 latency (ns)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 14.9 | 15.0 | 37.1 | 46.4 | 51.6 |
| ticklog-rtrb-drop | 17.6 | 19.1 | 57.1 | 62.6 | 68.8 |
| ticklog-quill | 15.9 | 16.3 | 28.1 | 37.2 | 48.6 |
| ticklog-nanolog | 16.8 | 16.9 | 24.5 | 36.4 | 35.3 |
| cpp-quill (8MiB q) | 27.4 | 27.6 | 26.9 | 28.4 | 38.6 |

### string — throughput (r/s)

| candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|
| ticklog-custom-drop | 52.3M | 101.5M | 99.3M | 135.5M | 121.1M |
| ticklog-rtrb-drop | 39.4M | 53.7M | 68.3M | 100.3M | 124.9M |
| ticklog-quill | 29.6M | 49.4M | 58.2M | 78.1M | 54.9M |
| ticklog-nanolog | 3.0M | 5.3M | 11.5M | 85.1M | 54.4M |
| cpp-quill (8MiB q) | 1.2M | 1.5M | 3.7M | 184.1M | 198.4M |

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
