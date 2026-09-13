# Cross-Language Benchmark Results

Candidates: nanolog, ticklog, zap, zerolog, quill (single_int only)

**OS:** linux  

**Arch:** x86_64  

**Batch size:** 1000  

**Samples per config:** 10000  

**Total messages per config:** 10,000,000  


## Latency

### single_int: 1 thread(s)

| Candidate | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------|----------|----------|----------|-----------|----------|
| nanolog   | 15.6     | 17.9     | 78.5     | 213.1     | 641.3    |
| ticklog   | 21.3     | 46.1     | 79.9     | 225.2     | 407.9    |
| quill     | 20.3     | 26.4     | 53.7     | 481.7     | 715512.0 |
| zap       | 603.2    | 1246.6   | 2114.2   | 3169.9    | 4216.2   |
| zerolog   | 94.4     | 186.7    | 295.5    | 660.8     | 1090.8   |

### single_int: 2 thread(s)

| Candidate | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------|----------|----------|----------|-----------|----------|
| nanolog   | 15.9     | 30.5     | 542.1    | 986.9     | 1083.7   |
| ticklog   | 21.2     | 41.3     | 70.1     | 226.4     | 1191.4   |
| quill     | 20.4     | 25.8     | 63.4     | 7609.0    | 424065.4 |
| zap       | 658.1    | 1425.4   | 2568.5   | 3430.0    | 8193.2   |
| zerolog   | 96.7     | 217.8    | 346.1    | 590.2     | 1837.4   |

### single_int: 4 thread(s)

| Candidate | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------|----------|----------|----------|-----------|----------|
| nanolog   | 16.4     | 40.6     | 2573.8   | 3724.7    | 4217.2   |
| ticklog   | 23.7     | 42.5     | 82.0     | 234.6     | 411.1    |
| quill     | 23.9     | 30.9     | 72.9     | 56598.2   | 227595.3 |
| zap       | 870.5    | 2497.1   | 3239.1   | 3991.6    | 4939.8   |
| zerolog   | 110.6    | 213.7    | 335.5    | 793.3     | 1417.2   |

### mixed: 1 thread(s)

| Candidate | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------|----------|----------|----------|-----------|----------|
| nanolog   | 17.0     | 74.9     | 524.0    | 901.4     | 1165.0   |
| ticklog   | 22.4     | 55.3     | 94.3     | 355.4     | 3313.1   |
| zap       | 790.0    | 2167.6   | 3066.9   | 4219.9    | 6793.3   |
| zerolog   | 208.3    | 411.9    | 613.7    | 1071.3    | 3464.8   |

### mixed: 2 thread(s)

| Candidate | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------|----------|----------|----------|-----------|----------|
| nanolog   | 17.9     | 348.8    | 893.6    | 2801.0    | 24254.7  |
| ticklog   | 43.0     | 59.4     | 80.0     | 187.6     | 271.4    |
| zap       | 974.0    | 2696.9   | 3355.0   | 4465.8    | 6225.6   |
| zerolog   | 236.2    | 426.3    | 586.7    | 1105.8    | 10492.0  |

### mixed: 4 thread(s)

| Candidate | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------|----------|----------|----------|-----------|----------|
| nanolog   | 17.6     | 167.7    | 2139.8   | 16514.8   | 57476.4  |
| ticklog   | 38.1     | 45.4     | 80.6     | 218.9     | 536.6    |
| zap       | 1293.1   | 3411.8   | 3920.2   | 4893.9    | 6284.0   |
| zerolog   | 229.0    | 423.9    | 588.0    | 1072.3    | 2571.1   |

### string: 1 thread(s)

| Candidate | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------|----------|----------|----------|-----------|----------|
| nanolog   | 15.8     | 25.6     | 226.2    | 711.9     | 1017.9   |
| ticklog   | 21.3     | 47.1     | 75.8     | 138.7     | 1805.7   |
| zap       | 598.0    | 1236.3   | 2171.8   | 3134.9    | 4200.3   |
| zerolog   | 92.8     | 183.7    | 303.5    | 536.6     | 2191.0   |

### string: 2 thread(s)

| Candidate | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------|----------|----------|----------|-----------|----------|
| nanolog   | 16.2     | 84.8     | 701.1    | 1121.1    | 25329.7  |
| ticklog   | 21.3     | 41.4     | 66.2     | 118.1     | 175.3    |
| zap       | 652.8    | 1468.5   | 2618.8   | 3373.0    | 6072.8   |
| zerolog   | 94.8     | 196.5    | 294.8    | 524.7     | 975.8    |

### string: 4 thread(s)

| Candidate | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------|----------|----------|----------|-----------|----------|
| nanolog   | 16.5     | 52.8     | 2497.0   | 2777.2    | 25145.8  |
| ticklog   | 36.6     | 43.1     | 86.0     | 269.8     | 1690.5   |
| zap       | 734.2    | 2502.9   | 3159.8   | 3887.2    | 4872.0   |
| zerolog   | 94.9     | 205.5    | 325.1    | 810.5     | 1861.8   |

## Throughput

### 1 thread(s)

| Candidate | single_int (r/s) | mixed (r/s) | string (r/s) |
|-----------|------------------|-------------|--------------|
| nanolog   | 54,993,120       | 28,952,454  | 43,425,563   |
| ticklog   | 35,385,110       | 29,768,115  | 36,025,082   |
| quill     | 6,483,680        | -           | -            |
| zap       | 1,349,207        | 977,913     | 1,356,924    |
| zerolog   | 9,087,944        | 4,119,691   | 9,036,533    |

### 2 thread(s)

| Candidate | single_int (r/s) | mixed (r/s) | string (r/s) |
|-----------|------------------|-------------|--------------|
| nanolog   | 64,388,128       | 29,058,471  | 43,852,242   |
| ticklog   | 66,564,069       | 52,245,181  | 69,180,640   |
| quill     | 10,542,470       | -           | -            |
| zap       | 2,387,263        | 1,613,346   | 2,364,382    |
| zerolog   | 14,646,279       | 7,240,888   | 16,289,773   |

### 4 thread(s)

| Candidate | single_int (r/s) | mixed (r/s) | string (r/s) |
|-----------|------------------|-------------|--------------|
| nanolog   | 55,194,358       | 30,926,201  | 44,141,267   |
| ticklog   | 114,947,992      | 104,592,094 | 110,233,553  |
| quill     | 19,305,046       | -           | -            |
| zap       | 3,986,593        | 2,542,221   | 4,125,549    |
| zerolog   | 26,603,223       | 13,419,308  | 28,450,813   |

## Thread Scaling

| Candidate | 1 thread    | 2 threads   | 4 threads   | scale 1->4 |
|-----------|-------------|-------------|-------------|------------|
| nanolog   | 127,371,137 | 137,298,841 | 130,261,826 | 1.02x      |
| ticklog   | 101,178,307 | 187,989,890 | 329,773,639 | 3.26x      |
| zap       | 3,684,044   | 6,364,991   | 10,654,363  | 2.89x      |
| zerolog   | 22,244,168  | 38,176,940  | 68,473,344  | 3.08x      |

## Jitter (1 thread, worst across workloads)

| Candidate | p99 (ns) | p999 (ns) | max (ns) | p99/p50 |
|-----------|----------|-----------|----------|---------|
| nanolog   | 524.0    | 901.4     | 1165.0   | 30.8x   |
| ticklog   | 94.3     | 355.4     | 3313.1   | 4.2x    |
| zap       | 3066.9   | 4219.9    | 6793.3   | 3.9x    |
| zerolog   | 613.7    | 1071.3    | 3464.8   | 3.3x    |

## Ring Capacity (ticklog vs others, single_int)

The ticklog harness accepts `--ring-capacity <bytes>` (default 1 MiB). Same
protocol as above: BATCH=1000, 10M messages per config.

### Latency (ns)

| Threads | ticklog 64K | 256K | 1M | 4M | 16M | nanolog | quill |
|---------|-------------|------|----|----|-----|---------|-------|
| 1       | 21.1        | 21.1 | 21.2 | 20.9 | 21.0 | 15.6 | 20.3 |
| 2       | 21.6        | 36.4 | 39.6 | 20.9 | 21.0 | 15.9 | 20.4 |
| 4       | 31.8        | 38.8 | 40.2 | 21.0 | 39.5 | 16.4 | 23.9 |

### Throughput (rec/s)

| Threads | ticklog 64K | 256K | 1M | 4M | 16M | nanolog | quill |
|---------|-------------|------|----|----|-----|---------|-------|
| 1       | 38.3M       | 33.1M | 33.1M | 32.7M | 23.9M | 55.0M | 6.5M |
| 2       | 66.3M       | 48.9M | 48.4M | 59.4M | 27.6M | 64.4M | 10.5M |
| 4       | 86.4M       | 88.1M | 75.8M | 81.4M | 14.2M | 55.2M | 19.3M |

Ring capacity barely changes ticklog's per-call latency (it only matters when
the producer must pause for the drain). Throughput peaks at 64K-256K in 1-4
threads; the 16M rows are distorted because a freshly zeroed 16 MiB ring per
thread is allocated inside the measured window for every config, so prefer
64K-4M for comparisons.

## Notes

- macOS results are best-effort (no core isolation, no frequency locking).
- Go and C++ harnesses self-calibrate via CNTFRQ_EL0 due to SDK-linkage differences.
- Quill single_int format ("x={}") is slower than simple "{}"; real fmt behavior.
- Canonical numbers require Linux x86_64 with `perf stat` and core isolation.
