# Cross-Language Benchmark Results

Candidates: nanolog, quill, ticklog, ticklog_file, zap, zerolog

**OS:** linux  

**Arch:** x86_64  

**Batch size:** 1000  

**Samples per config:** 10000  

**Total messages per config:** 10,000,000  


## Latency

### single_int: 1 thread(s)

| Candidate    | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|--------------|----------|----------|----------|-----------|----------|
| nanolog      | 16.5     | 20.5     | 70.9     | 130.0     | 401.6    |
| quill        | 20.4     | 35.9     | 64.5     | 624.5     | 814846.6 |
| ticklog      | 21.6     | 48.1     | 79.8     | 169.3     | 750.8    |
| ticklog_file | 21.2     | 46.0     | 80.8     | 252.4     | 577.5    |
| zap          | 590.4    | 1149.3   | 2224.1   | 3156.1    | 9740.9   |
| zerolog      | 94.4     | 187.9    | 281.8    | 569.0     | 1674.1   |

### single_int: 2 thread(s)

| Candidate    | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|--------------|----------|----------|----------|-----------|----------|
| nanolog      | 17.1     | 38.5     | 491.8    | 966.9     | 2376.8   |
| quill        | 20.4     | 26.7     | 67.3     | 15989.8   | 414028.2 |
| ticklog      | 21.4     | 41.7     | 75.2     | 220.5     | 802.1    |
| ticklog_file | 21.1     | 41.5     | 88.0     | 415.2     | 1433.4   |
| zap          | 642.7    | 1540.1   | 2841.3   | 3800.6    | 5246.8   |
| zerolog      | 94.4     | 204.1    | 341.1    | 786.9     | 1457.7   |

### single_int: 4 thread(s)

| Candidate    | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|--------------|----------|----------|----------|-----------|----------|
| nanolog      | 16.9     | 20.1     | 1730.6   | 3112.5    | 23655.5  |
| quill        | 25.6     | 38.5     | 74.3     | 57284.9   | 257480.6 |
| ticklog      | 40.6     | 50.6     | 83.8     | 273.6     | 745.1    |
| ticklog_file | 40.5     | 42.9     | 78.0     | 217.1     | 1151.2   |
| zap          | 723.4    | 2668.9   | 3202.2   | 3733.7    | 4221.2   |
| zerolog      | 153.0    | 244.4    | 355.5    | 641.2     | 1330.8   |

### single_int: 8 thread(s)

| Candidate    | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|--------------|----------|----------|----------|-----------|----------|
| nanolog      | 17.2     | 42.8     | 5982.9   | 6310.9    | 6572.8   |
| quill        | 25.8     | 35.9     | 153.3    | 84958.1   | 208678.4 |
| ticklog      | 40.7     | 56.2     | 121.9    | 925.0     | 4463.1   |
| ticklog_file | 40.7     | 54.8     | 112.5    | 1976.1    | 6616.2   |
| zap          | 1035.8   | 3596.3   | 4121.6   | 5633.2    | 8285.5   |
| zerolog      | 180.0    | 307.3    | 856.0    | 6423.4    | 8918.1   |

### single_int: 16 thread(s)

| Candidate    | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|--------------|----------|----------|----------|-----------|----------|
| nanolog      | 17.3     | 43.8     | 27701.2  | 34289.0   | 42159.4  |
| quill        | 25.6     | 36.8     | 3888.0   | 145597.1  | 186783.0 |
| ticklog      | 40.7     | 61.0     | 175.9    | 3442.5    | 21104.0  |
| ticklog_file | 40.7     | 61.2     | 286.7    | 4999.3    | 9170.3   |
| zap          | 1437.1   | 7638.9   | 13385.4  | 27745.1   | 36927.8  |
| zerolog      | 180.1    | 304.4    | 1104.0   | 6028.7    | 12644.6  |

### mixed: 1 thread(s)

| Candidate    | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|--------------|----------|----------|----------|-----------|----------|
| nanolog      | 16.1     | 93.4     | 521.8    | 839.4     | 1215.6   |
| ticklog      | 22.8     | 46.4     | 76.9     | 141.7     | 490.2    |
| ticklog_file | 22.2     | 44.1     | 85.7     | 322.5     | 1483.0   |
| zap          | 765.9    | 2220.7   | 3311.4   | 4017.4    | 4794.8   |
| zerolog      | 207.2    | 399.4    | 606.3    | 972.9     | 1778.9   |

### mixed: 2 thread(s)

| Candidate    | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|--------------|----------|----------|----------|-----------|----------|
| nanolog      | 17.0     | 333.0    | 855.8    | 2558.9    | 2985.3   |
| ticklog      | 22.6     | 44.6     | 83.7     | 199.4     | 618.4    |
| ticklog_file | 22.3     | 44.3     | 86.9     | 338.4     | 1130.7   |
| zap          | 849.8    | 2801.2   | 3446.1   | 3982.8    | 5469.3   |
| zerolog      | 208.7    | 415.6    | 587.7    | 1123.4    | 2104.7   |

### mixed: 4 thread(s)

| Candidate    | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|--------------|----------|----------|----------|-----------|----------|
| nanolog      | 18.0     | 197.0    | 2179.6   | 4276.0    | 40788.6  |
| ticklog      | 43.5     | 50.5     | 90.9     | 282.5     | 1036.4   |
| ticklog_file | 38.7     | 45.6     | 87.8     | 314.7     | 1143.8   |
| zap          | 1288.3   | 3541.1   | 4027.4   | 4483.0    | 5471.3   |
| zerolog      | 335.7    | 446.9    | 621.9    | 1129.5    | 2582.8   |

### mixed: 8 thread(s)

| Candidate    | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|--------------|----------|----------|----------|-----------|----------|
| nanolog      | 18.3     | 165.7    | 4858.4   | 25034.3   | 65325.8  |
| ticklog      | 43.5     | 56.7     | 105.1    | 374.1     | 1854.8   |
| ticklog_file | 43.5     | 57.2     | 95.5     | 378.9     | 1624.8   |
| zap          | 2845.7   | 4281.0   | 4727.8   | 8319.7    | 9280.1   |
| zerolog      | 359.0    | 461.4    | 593.2    | 839.3     | 1769.2   |

### mixed: 16 thread(s)

| Candidate    | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|--------------|----------|----------|----------|-----------|----------|
| nanolog      | 19.1     | 192.7    | 19963.9  | 61429.2   | 71597.7  |
| ticklog      | 43.6     | 60.6     | 185.3    | 3711.8    | 11593.8  |
| ticklog_file | 43.6     | 63.0     | 302.8    | 4062.6    | 8091.3   |
| zap          | 4518.3   | 12911.0  | 18707.7  | 25789.7   | 37794.3  |
| zerolog      | 385.6    | 510.1    | 2576.7   | 9674.3    | 12770.2  |

### string: 1 thread(s)

| Candidate    | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|--------------|----------|----------|----------|-----------|----------|
| nanolog      | 16.6     | 26.8     | 155.3    | 610.7     | 857.1    |
| ticklog      | 21.9     | 44.2     | 71.1     | 134.7     | 539.0    |
| ticklog_file | 21.3     | 42.0     | 78.3     | 354.5     | 807.2    |
| zap          | 583.7    | 1140.8   | 2222.9   | 2951.7    | 11449.8  |
| zerolog      | 92.8     | 185.4    | 315.7    | 589.3     | 998.5    |

### string: 2 thread(s)

| Candidate    | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|--------------|----------|----------|----------|-----------|----------|
| nanolog      | 16.5     | 105.3    | 853.8    | 2368.2    | 3104.7   |
| ticklog      | 21.8     | 43.4     | 75.3     | 148.7     | 354.5    |
| ticklog_file | 21.5     | 42.0     | 78.0     | 334.8     | 1866.6   |
| zap          | 614.0    | 1404.7   | 2872.7   | 3713.3    | 11432.7  |
| zerolog      | 94.8     | 205.6    | 353.7    | 661.3     | 1022.0   |

### string: 4 thread(s)

| Candidate    | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|--------------|----------|----------|----------|-----------|----------|
| nanolog      | 16.6     | 59.6     | 2460.6   | 3933.8    | 17755.2  |
| ticklog      | 38.9     | 43.4     | 80.0     | 240.9     | 2484.4   |
| ticklog_file | 40.8     | 43.0     | 81.1     | 309.1     | 1175.3   |
| zap          | 682.1    | 2652.2   | 3280.1   | 3907.4    | 5326.1   |
| zerolog      | 129.6    | 250.1    | 324.1    | 544.5     | 819.7    |

### string: 8 thread(s)

| Candidate    | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|--------------|----------|----------|----------|-----------|----------|
| nanolog      | 17.4     | 51.6     | 5705.4   | 42685.4   | 68398.9  |
| ticklog      | 40.5     | 41.3     | 73.2     | 190.9     | 708.0    |
| ticklog_file | 40.8     | 41.5     | 81.7     | 185.6     | 898.1    |
| zap          | 1032.8   | 3613.5   | 4157.7   | 4918.9    | 7286.1   |
| zerolog      | 175.8    | 290.3    | 363.7    | 563.0     | 2793.9   |

### string: 16 thread(s)

| Candidate    | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|--------------|----------|----------|----------|-----------|----------|
| nanolog      | 17.6     | 98.1     | 24080.5  | 25398.7   | 29029.1  |
| ticklog      | 40.9     | 60.2     | 211.1    | 1005.6    | 15756.0  |
| ticklog_file | 40.9     | 56.3     | 279.3    | 4078.1    | 7363.5   |
| zap          | 1684.7   | 7236.9   | 12984.8  | 20832.5   | 29713.0  |
| zerolog      | 181.2    | 280.1    | 428.4    | 4177.1    | 8181.3   |

## Throughput

### 1 thread(s)

| Candidate    | single_int (r/s) | mixed (r/s) | string (r/s) |
|--------------|------------------|-------------|--------------|
| nanolog      | 54,016,320       | 29,289,246  | 43,988,328   |
| quill        | 5,154,683        | -           | -            |
| ticklog      | 32,579,578       | 31,741,389  | 36,993,038   |
| ticklog_file | 32,888,619       | 32,100,231  | 34,443,277   |
| zap          | 1,434,791        | 1,042,076   | 1,447,374    |
| zerolog      | 9,317,041        | 4,214,300   | 9,105,683    |

### 2 thread(s)

| Candidate    | single_int (r/s) | mixed (r/s) | string (r/s) |
|--------------|------------------|-------------|--------------|
| nanolog      | 64,106,178       | 34,583,326  | 44,202,085   |
| quill        | 10,301,479       | -           | -            |
| ticklog      | 66,213,462       | 64,810,429  | 60,980,487   |
| ticklog_file | 62,231,560       | 61,669,678  | 62,343,446   |
| zap          | 2,403,323        | 1,722,685   | 2,520,044    |
| zerolog      | 15,567,667       | 7,679,761   | 15,792,443   |

### 4 thread(s)

| Candidate    | single_int (r/s) | mixed (r/s) | string (r/s) |
|--------------|------------------|-------------|--------------|
| nanolog      | 57,003,767       | 29,692,916  | 46,388,069   |
| quill        | 18,330,868       | -           | -            |
| ticklog      | 103,244,126      | 98,419,545  | 102,194,043  |
| ticklog_file | 105,049,346      | 98,447,536  | 107,916,334  |
| zap          | 4,175,085        | 2,481,330   | 4,229,691    |
| zerolog      | 24,066,592       | 12,475,054  | 25,756,147   |

### 8 thread(s)

| Candidate    | single_int (r/s) | mixed (r/s) | string (r/s) |
|--------------|------------------|-------------|--------------|
| nanolog      | 52,712,508       | 29,898,440  | 30,488,589   |
| quill        | 21,816,255       | -           | -            |
| ticklog      | 164,256,927      | 167,880,637 | 186,975,521  |
| ticklog_file | 151,061,489      | 170,845,963 | 210,747,166  |
| zap          | 5,767,602        | 3,079,494   | 5,827,725    |
| zerolog      | 30,936,176       | 22,653,348  | 39,569,095   |

### 16 thread(s)

| Candidate    | single_int (r/s) | mixed (r/s) | string (r/s) |
|--------------|------------------|-------------|--------------|
| nanolog      | 23,907,444       | 20,674,467  | 25,369,182   |
| quill        | 22,581,740       | -           | -            |
| ticklog      | 194,354,960      | 215,910,072 | 230,057,717  |
| ticklog_file | 218,191,384      | 223,744,370 | 207,341,700  |
| zap          | 5,802,436        | 2,808,036   | 5,808,907    |
| zerolog      | 62,132,347       | 33,174,603  | 66,128,900   |

## Thread Scaling

| Candidate    | 1 thread   | 2 threads  | 4 threads   | 8 threads   | 16 threads  | scale 1->16 |
|--------------|------------|------------|-------------|-------------|-------------|-------------|
| nanolog      | 54,016,320 | 64,106,178 | 57,003,767  | 52,712,508  | 23,907,444  | 0.44x       |
| quill        | 5,154,683  | 10,301,479 | 18,330,868  | 21,816,255  | 22,581,740  | 4.38x       |
| ticklog      | 32,579,578 | 66,213,462 | 103,244,126 | 164,256,927 | 194,354,960 | 5.97x       |
| ticklog_file | 32,888,619 | 62,231,560 | 105,049,346 | 151,061,489 | 218,191,384 | 6.63x       |
| zap          | 1,434,791  | 2,403,323  | 4,175,085   | 5,767,602   | 5,802,436   | 4.04x       |
| zerolog      | 9,317,041  | 15,567,667 | 24,066,592  | 30,936,176  | 62,132,347  | 6.67x       |

## Jitter (1 thread, worst across workloads)

| Candidate    | p99 (ns) | p999 (ns) | max (ns) | p99/p50 |
|--------------|----------|-----------|----------|---------|
| nanolog      | 521.8    | 839.4     | 1215.6   | 32.4x   |
| quill        | 64.5     | 624.5     | 814846.6 | 3.2x    |
| ticklog      | 79.8     | 169.3     | 750.8    | 3.7x    |
| ticklog_file | 85.7     | 354.5     | 1483.0   | 3.9x    |
| zap          | 3311.4   | 4017.4    | 11449.8  | 4.3x    |
| zerolog      | 606.3    | 972.9     | 1778.9   | 3.4x    |

## Ring Capacity (ticklog vs others, single_int)

The ticklog harness accepts `--ring-capacity <bytes>` (default 1 MiB). Same

protocol as above: BATCH=1000, 10M messages per config.


### Latency p50 (ns)
| Threads | 64K | 256K | 1024K | 4096K | 16384K | nanolog | quill |
|---|---|---|---|---|---|---|---|
| 1 | 21.0 | 20.9 | 20.9 | 20.9 | 20.9 | 16.5 | 20.4 |
| 2 | 21.0 | 21.0 | 21.0 | 21.1 | 40.2 | 17.1 | 20.4 |
| 4 | 38.9 | 22.1 | 21.6 | 25.4 | 21.1 | 16.9 | 25.6 |
| 8 | 40.2 | 40.2 | 40.2 | 40.3 | 40.2 | 17.2 | 25.8 |
| 16 | 40.2 | 40.2 | 40.2 | 40.2 | 40.2 | 17.3 | 25.6 |

### Throughput (rec/s)
| Threads | 64K | 256K | 1024K | 4096K | 16384K | nanolog | quill |
|---|---|---|---|---|---|---|---|
| 1 | 41.9M | 42.3M | 42.8M | 43.2M | 40.8M | 54.0M | 5.2M |
| 2 | 66.6M | 65.9M | 71.6M | 62.8M | 53.4M | 64.1M | 10.3M |
| 4 | 105.0M | 107.4M | 117.0M | 113.3M | 87.0M | 57.0M | 18.3M |
| 8 | 165.1M | 183.6M | 172.7M | 152.3M | 91.1M | 52.7M | 21.8M |
| 16 | 177.0M | 217.2M | 209.4M | 161.6M | 95.8M | 23.9M | 22.6M |

## File Sink (ticklog null vs file, single_int)

The ticklog harness accepts `--sink-file <path>`: the drain formats every
record and writes it through a buffered `FileSink` (`BufWriter`, 64 KiB)
instead of discarding lines in a null sink. Same protocol as above:
BATCH=1000, 10M messages per config. nanolog writes to its own log file on
one background thread for reference (8/16-thread runs on this box).

### Throughput (rec/s)

| Threads | ticklog | ticklog_file | nanolog |
|---|---|---|---|
| 1 | 32.6M | 32.9M | 54.0M |
| 2 | 66.2M | 62.2M | 64.1M |
| 4 | 103.2M | 105.0M | 57.0M |
| 8 | 164.3M | 151.1M | 52.7M |
| 16 | 194.4M | 218.2M | 23.9M |

### Latency p50 (ns)
| Threads | ticklog | ticklog_file | nanolog |
|---|---|---|---|
| 1 | 21.6 | 21.2 | 16.5 |
| 2 | 21.4 | 21.1 | 17.1 |
| 4 | 40.6 | 40.5 | 16.9 |
| 8 | 40.7 | 40.7 | 17.2 |
| 16 | 40.7 | 40.7 | 17.3 |

## Experimental SPSC Backends (ringbuf crate, triple_buffer crate)

The custom ring stays the default, but the crate ships two experimental
feature-gated streaming backends plus the ringbuffer-crate backend from
[e610aa4] for the single-producer/single-drain case:

| Feature               | Backend                                   |
| --------------------- | ----------------------------------------- |
| `backend-ringbuffer`  | `ringbuffer` crate byte FIFO              |
| `backend-ringbuf`     | `ringbuf` crate (`HeapRb`, byte FIFO)     |
| `backend-triple-buffer`| `triple_buffer` crate single-slot handoff |

The `triple_buffer` backend is fundamentally a **ping-pong, not a buffer**:
`reserve` rendezvouses with the drain on every record, so it only fits lossy
uniprocessor slipstreams, never the block/drop hot path ticklog targets. The
`ringbuf` backend keeps the slot means a real byte FIFO but pays a mutex per
record enqueue/consume plus a memcpy.

Single-thread (1 producer pinned to CPU 0, drain pinned to CPU 1,
BATCH=1000, 10M messages per config):

### Latency @ 1 thread (ns)

| Candidate          | workload | p50    | p95    | p99    | p999   | max    |
| ------------------ | -------- | ------ | ------ | ------ | ------ | ------ |
| ticklog (custom)   | single_int | 15.2 | 18.3   | 38.3   | 158.9  | 315.1  |
| ticklog_ringbuf    | single_int | 15.6 | 43.7   | 61.5   | 170.3  | 325.2  |
| ticklog_triple-buffer | single_int | 718.6 | 1064.9 | 1444.5 | 2352.0 | 10400.4 |
| ticklog (custom)   | mixed      | 15.9  | 17.6   | 43.6   | 123.5  | 458.6  |
| ticklog_ringbuf    | mixed      | 16.0  | 33.5   | 75.2   | 211.2  | 2079.8 |
| ticklog_triple-buffer | mixed   | 841.1 | 1366.4 | 1842.2 | 2600.0 | 10874.9 |
| ticklog (custom)   | string     | 15.8  | 25.9   | 46.7   | 132.8  | 324.5  |
| ticklog_ringbuf    | string     | 15.6  | 45.6   | 72.1   | 158.1  | 294.5  |
| ticklog_triple-buffer | string  | 683.5 | 991.5  | 1331.8 | 2217.4 | 7727.3 |

### Throughput @ 1 thread (rec/s)

| Candidate          | single_int | mixed      | string     |
| ------------------ | ---------- | ---------- | ---------- |
| ticklog (custom)   | 60,487,254 | 58,066,093 | 57,610,285 |
| ticklog_ringbuf    | 52,000,433 | 46,347,031 | 48,938,700 |
| ticklog_triple-buffer | 1,306,899 | 1,048,392 | 1,364,928 |

Takeaways: the `ringbuf` backend is within ~13-20% of the custom slot ring on
throughput with equal p50, but its p95 tail is 1.5-2x worse (33-46 ns vs
18-26 ns). Compare the custom ring against e610aa4's ringbuffer backend and
the Go/C++ loggers in the tables above. The `triple_buffer` backend's
reserve/consume handshake caps it at ~1M rec/s here and is unsuitable for
this benchmark; it is kept only as a data point for lossy slipstream use.

## Notes

- Numbers are best-effort on an unisolated 16-vCPU WSL2 box (no perf/core isolation).
  Quill's 8/16-thread runs massively inflate SPSC queues and crashed the VM several
  times; treat its high-thread numbers as lower bounds.
- Go and C++ harnesses self-calibrate via CNTFRQ_EL0/rdtsc due to SDK-linkage differences.
- Quill single_int format ("x={}") is slower than simple "{}"; real fmt behavior.
- Canonical numbers require Linux x86_64 with `perf stat` and core isolation.
- `ticklog_file` uses `--sink-file`: the drain writes formatted lines to a real
  file (`BufWriter`, 64 KiB). The harness measure wall time around producers;
  file writes land in page cache on the same host as nanolog's log file.
---

# Run 2 — Different Hardware (WSL2 box)

Second run on **different hardware**: Intel(R) Xeon(R) CPU E5-2689 0 @ 2.60GHz,
16 vCPUs, 12 GB RAM, WSL2 (kernel 5.10.16.3-microsoft-standard-WSL2), Ubuntu 22.04.
Best-effort numbers — no core isolation and no `perf` (not available on the WSL
kernel), performance governor not settable.

Candidates in this run: **ticklog, nanolog, zerolog, zap**. Quill was skipped:
its SPSC queues double up to 128+ MiB per producer and the 8/16-thread configs
grew past ~11 GB RSS and kept OOM-killing the VM.

Toolchain for this run:
- C++ (nanolog): **clang-22** with `-O3 -march=native -mtune=native -flto=thin
  -fuse-ld=/usr/bin/ld.lld-22 -funroll-loops -fstrict-aliasing
  -fomit-frame-pointer -fno-semantic-interposition -ffast-math`.
- Rust (ticklog): `RUSTFLAGS="-C target-cpu=native"` plus the crate profile
  `lto = "fat", codegen-units = 1`.
- Go (zerolog/zap): go1.27.1.

Protocol is unchanged: BATCH=1000, SAMPLES=10000, 10M messages per config.

## Latency (Run 2)

### single_int

| Threads | Candidate | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|---------|-----------|----------|----------|----------|-----------|----------|
| 1 | ticklog   | 8.7      | 11.3     | 21.4     | 37.3      | 115.3    |
| 1 | nanolog   | 8.8      | 24.7     | 200.1    | 635.9     | 719.5    |
| 1 | zerolog   | 113.2    | 130.6    | 161.1    | 241.6     | 284.4    |
| 1 | zap       | 552.6    | 642.0    | 1303.2   | 1621.6    | 4399.0   |
| 2 | ticklog   | 8.7      | 8.7      | 21.9     | 60.0      | 135.6    |
| 2 | nanolog   | 8.8      | 26.8     | 750.5    | 884.0     | 225051.8 |
| 2 | zerolog   | 122.4    | 317.0    | 334.3    | 386.9     | 461.9    |
| 2 | zap       | 568.5    | 880.3    | 1726.8   | 1955.9    | 2234.9   |
| 4 | ticklog   | 8.7      | 13.1     | 22.4     | 61.8      | 102.2    |
| 4 | nanolog   | 12.9     | 31.6     | 2085.3   | 2559.1    | 223855.3 |
| 4 | zerolog   | 113.0    | 266.1    | 330.5    | 379.3     | 799.7    |
| 4 | zap       | 597.6    | 1842.3   | 2223.8   | 2526.7    | 2775.8   |
| 8 | ticklog   | 12.1     | 13.1     | 29.7     | 88.6      | 125.0    |
| 8 | nanolog   | 12.8     | 34.4     | 7137.9   | 7652.2    | 9857748.7 |
| 8 | zerolog   | 132.8    | 302.3    | 413.1    | 520.6     | 10062.2  |
| 8 | zap       | 904.8    | 2798.5   | 3231.4   | 7246.8    | 9129.7   |
| 16 | ticklog  | 13.0     | 13.2     | 77.1     | 476.2     | 8114.7   |
| 16 | nanolog  | 13.2     | 95.9     | 35270.0  | 269485.6  | 301266.6 |
| 16 | zerolog  | 242.7    | 608.6    | 1577.8   | 4270.4    | 13680.8  |
| 16 | zap      | 1354.4   | 6167.7   | 11944.4  | 21776.5   | 29200.1  |

### mixed

| Threads | Candidate | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|---------|-----------|----------|----------|----------|-----------|----------|
| 1 | ticklog   | 9.3     | 9.4      | 24.4     | 44.0      | 81.6     |
| 1 | nanolog   | 10.8    | 166.8    | 251.6    | 659.5     | 1517717.9 |
| 1 | zerolog   | 237.0   | 259.2    | 336.9    | 378.2     | 415.9    |
| 1 | zap       | 814.5   | 1412.2   | 1718.5   | 1924.3    | 2233.9   |
| 2 | ticklog   | 9.3     | 9.4      | 24.6     | 56.3      | 104.5    |
| 2 | nanolog   | 10.9    | 240.8    | 750.1    | 1364.9    | 278943.8 |
| 2 | zerolog   | 239.6   | 389.6    | 414.2    | 513.5     | 529.7    |
| 2 | zap       | 850.1   | 1894.0   | 2157.7   | 2493.0    | 3617.0   |
| 4 | ticklog   | 9.3     | 9.4      | 22.1     | 62.5      | 104.1    |
| 4 | nanolog   | 10.9    | 271.7    | 2171.2   | 264985.1  | 4894285.7 |
| 4 | zerolog   | 249.9   | 425.3    | 609.8    | 657.0     | 740.3    |
| 4 | zap       | 942.5   | 2503.3   | 2812.1   | 3193.0    | 4680.3   |
| 8 | ticklog   | 12.8    | 14.0     | 35.0     | 126.1     | 284.2    |
| 8 | nanolog   | 14.8    | 324.5    | 6646.1   | 1660004.2 | 2775908.1 |
| 8 | zerolog   | 293.2   | 577.8    | 626.1    | 680.4     | 7201.6   |
| 8 | zap       | 2586.0  | 3793.1   | 4629.0   | 10826.0   | 11548.6  |
| 16 | ticklog  | 13.7    | 14.1     | 85.0     | 660.1     | 8888.3   |
| 16 | nanolog  | 15.3    | 437.6    | 36562.2  | 366809.0  | 389993.8 |
| 16 | zerolog  | 471.8   | 734.0    | 1845.1   | 5274.3    | 8496.8   |
| 16 | zap      | 4486.2  | 12588.9  | 19321.0  | 33510.9   | 62163.3  |

### string

| Threads | Candidate | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|---------|-----------|----------|----------|----------|-----------|----------|
| 1 | ticklog   | 8.8    | 8.8      | 24.1     | 41.2      | 69.1     |
| 1 | nanolog   | 9.1    | 99.7     | 260.6    | 349.3     | 252216.8 |
| 1 | zerolog   | 119.0  | 136.7    | 164.3    | 242.5     | 286.0    |
| 1 | zap       | 586.9  | 665.9    | 1321.4   | 1598.0    | 1899.2   |
| 2 | ticklog   | 8.8    | 8.8      | 24.0     | 43.1      | 80.2     |
| 2 | nanolog   | 9.1    | 173.3    | 848.1    | 1021.7    | 258927.7 |
| 2 | zerolog   | 131.6  | 325.1    | 344.9    | 424.6     | 509.0    |
| 2 | zap       | 605.0  | 1018.6   | 1765.2   | 2075.9    | 2291.0   |
| 4 | ticklog   | 8.7    | 8.8      | 19.7     | 43.8      | 101.0    |
| 4 | nanolog   | 9.2    | 171.7    | 2203.9   | 3517.1    | 9811392.1 |
| 4 | zerolog   | 114.6  | 241.3    | 344.3    | 383.5     | 420.2    |
| 4 | zap       | 620.5  | 1868.0   | 2219.8   | 2537.1    | 2738.3   |
| 8 | ticklog   | 13.7   | 13.7     | 30.8     | 83.0      | 149.1    |
| 8 | nanolog   | 13.6   | 143.5    | 7352.9   | 220981.5  | 237119.6 |
| 8 | zerolog   | 136.8  | 350.4    | 408.3    | 469.8     | 547.6    |
| 8 | zap       | 1102.5 | 3257.4   | 4136.4   | 7690.2    | 12642.2  |
| 16 | ticklog  | 13.7   | 14.6     | 87.8     | 721.1     | 1983.0   |
| 16 | nanolog  | 13.7   | 259.6    | 38676.7  | 7598366.1 | 7604913.2 |
| 16 | zerolog  | 217.3  | 401.6    | 553.0    | 1975.0    | 3121.7   |
| 16 | zap      | 1468.2 | 6491.1   | 12407.0  | 18706.5   | 32990.1  |

## Throughput (Run 2)

| Threads | Candidate | single_int (r/s) | mixed (r/s) | string (r/s) |
|---------|-----------|------------------|-------------|--------------|
| 1 | ticklog   | 108,006,407 | 102,351,317 | 107,275,074 |
| 1 | nanolog   | 63,100,838  | 3,300,708   | 14,481,873  |
| 1 | zerolog   | 8,536,660   | 4,115,781   | 8,168,723   |
| 1 | zap       | 1,742,115   | 1,152,082   | 1,647,518   |
| 2 | ticklog   | 217,238,765 | 205,979,587 | 216,858,587 |
| 2 | nanolog   | 26,993,810  | 7,673,395   | 14,284,057  |
| 2 | zerolog   | 11,118,196  | 7,562,616   | 10,229,173  |
| 2 | zap       | 3,147,376   | 1,982,959   | 2,970,617   |
| 4 | ticklog   | 304,715,472 | 405,508,426 | 424,435,607 |
| 4 | nanolog   | 26,628,847  | 1,682,657   | 973,717     |
| 4 | zerolog   | 23,013,662  | 12,150,059  | 27,271,765  |
| 4 | zap       | 5,266,670   | 3,063,766   | 5,070,580   |
| 8 | ticklog   | 529,192,928 | 526,321,330 | 518,726,009 |
| 8 | nanolog   | 994,763     | 1,892,485   | 14,210,345  |
| 8 | zerolog   | 39,352,587  | 20,476,379  | 38,484,851  |
| 8 | zap       | 6,832,685   | 3,332,829   | 5,780,801   |
| 16 | ticklog  | 553,375,869 | 556,445,869 | 574,092,360 |
| 16 | nanolog  | 15,083,429  | 5,641,708   | 892,159     |
| 16 | zerolog  | 44,683,491  | 27,047,503  | 51,162,646  |
| 16 | zap      | 6,441,177   | 2,805,332   | 6,175,505   |

## Thread Scaling (Run 2, single_int, r/s)

| Candidate | 1 | 2 | 4 | 8 | 16 | scale 1->16 |
|-----------|--------|---------|---------|---------|---------|-------------|
| ticklog   | 108M   | 217M    | 305M    | 529M    | 553M    | 5.12x |
| zerolog   | 8.5M   | 11.1M   | 23.0M   | 39.4M   | 44.7M   | 5.23x |
| zap       | 1.7M   | 3.1M    | 5.3M    | 6.8M    | 6.4M    | 3.70x |
| nanolog   | 63.1M  | 27.0M   | 26.6M   | 0.99M   | 15.1M   | 0.24x |

## Jitter (Run 2, 1 thread, worst across workloads)

| Candidate | p99 (ns) | p999 (ns) | max (ns) | p99/p50 |
|-----------|----------|-----------|-----------|---------|
| ticklog   | 24.4    | 44.0      | 115.3     | 2.7x |
| zap       | 1718.5  | 1924.3    | 4399.0    | 2.4x |
| zerolog   | 336.9   | 378.2     | 415.9     | 1.4x |
| nanolog   | 260.6   | 659.5     | 1517717.9 | 28.7x |

## Notes (Run 2)

- Same protocol as the run above (BATCH=1000, 10M messages per config), but on
  **different hardware / environment** — do not compare absolute numbers directly
  across the two runs.
- Quill is absent: its per-producer SPSC queues double up to 128+ MiB and the
  8/16-thread configs grew past ~11 GB RSS, OOM-killing this VM. Treat any
  historical quill high-thread numbers as lower bounds.
- `ticklog_file` (--sink-file) was not part of this run.
- No core isolation and no `perf` on the WSL kernel; cpupower cannot set the
  governor. Numbers are best-effort.

# Run 3 - Windows native desktop

Rerun of the full methodology natively on Windows 11 Pro (build 26200),
AMD Ryzen 7 7735HS (8C/16T), 13.3 GB RAM. Same protocol (BATCH=1000,
10M messages per config, 10000 samples).
**Not comparable to Run 1 or Run 2** (different environment/toolchain/scheduler).
Compare against Run 4 (WSL2, same host and physical CPU) only loosely -- native
Windows has no core isolation and Windows thread scheduling differs.

Toolchain: Rust 1.98.1 (MSVC, lto=fat, codegen-units=1) for ticklog; Go 1.27.0
for zerolog/zap; clang 23.1.1 (LLVM) for quill (built with CMake + Ninja).
Latency clock: rdtsc. ns_per_tick = 0.313097 (calibrated in WSL2 on the same
physical CPU via invariant TSC; the Windows `calibrate` uses clock_gettime,
which is not linkable against the MSVC/LLVM target this harness is built for).


**Host:** AMD Ryzen 7 7735HS (8C/16T), 13.3 GB RAM

**OS:** windows (x86_64)  
**Clock:** rdtsc, ns_per_tick = 0.313097  
**Batch:** 1000 msgs, **samples:** 10000, **total:** 10,000,000/config  

## Latency (Run 3)

### single_int: 1 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| quill                 | 16.0     | 23.4     | 31.2     | 459.1     | 163447.8 |
| ticklog               | 17.2     | 17.4     | 30.8     | 37.7      | 91.2     |
| ticklog_file          | 14.9     | 19.1     | 32.2     | 66.9      | 150.6    |
| ticklog_ringbuf       | 25.1     | 26.1     | 79.3     | 90.9      | 114.7    |
| ticklog_triple-buffer | 1210.6   | 1275.5   | 1350.0   | 1686.8    | 6939.4   |
| zap                   | 445.6    | 1180.3   | 2081.4   | 2773.7    | 5671.6   |
| zerolog               | 100.2    | 195.0    | 262.6    | 456.0     | 875.6    |

### single_int: 2 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| quill                 | 18.4     | 23.4     | 41.6     | 5517.6    | 217509.8 |
| ticklog               | 16.2     | 16.8     | 28.2     | 49.2      | 121.2    |
| ticklog_file          | 18.5     | 20.1     | 34.8     | 79.0      | 274.8    |
| ticklog_ringbuf       | 24.2     | 25.1     | 64.8     | 97.1      | 118.6    |
| ticklog_triple-buffer | 2147.4   | 3628.5   | 4754.4   | 6104.1    | 15099.9  |
| zap                   | 569.3    | 1403.3   | 2168.9   | 14439.8   | 18680.6  |
| zerolog               | 100.2    | 194.2    | 237.2    | 405.7     | 619.7    |

### single_int: 4 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| quill                 | 20.6     | 24.6     | 51.6     | 208912.1  | 475045.8 |
| ticklog               | 16.2     | 17.2     | 28.9     | 86.8      | 271.0    |
| ticklog_file          | 17.3     | 20.1     | 29.9     | 95.0      | 1251.8   |
| ticklog_ringbuf       | 22.9     | 26.2     | 50.6     | 96.0      | 120.0    |
| ticklog_triple-buffer | 4194.4   | 5210.3   | 5306.7   | 5669.3    | 6597.1   |
| zap                   | 754.6    | 1835.7   | 2949.1   | 17449.0   | 25263.1  |
| zerolog               | 164.0    | 202.2    | 293.6    | 754.4     | 7221.4   |

### single_int: 8 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns)  |
|-----------------------|----------|----------|----------|-----------|-----------|
| quill                 | 20.1     | 26.2     | 70.9     | 164288.7  | 1586520.8 |
| ticklog               | 19.8     | 20.2     | 36.2     | 114.5     | 1105.9    |
| ticklog_file          | 18.6     | 22.5     | 65.9     | 334.2     | 8069.2    |
| ticklog_ringbuf       | 25.9     | 29.0     | 93.1     | 124.8     | 659.7     |
| ticklog_triple-buffer | 9275.3   | 13811.0  | 17672.5  | 26338.0   | 49909.9   |
| zap                   | 914.9    | 2912.3   | 9262.4   | 16319.2   | 22451.1   |
| zerolog               | 179.4    | 257.8    | 321.8    | 649.5     | 1136.3    |

### single_int: 16 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| quill                 | 20.2     | 32.7     | 1458.8   | 64501.7   | 300146.7 |
| ticklog               | 19.8     | 20.4     | 77.7     | 789.0     | 11342.7  |
| ticklog_file          | 19.8     | 20.7     | 116.7    | 1125.1    | 10325.6  |
| ticklog_ringbuf       | 25.9     | 95.0     | 117.1    | 806.1     | 14089.4  |
| ticklog_triple-buffer | 19173.5  | 21557.0  | 21891.6  | 23599.5   | 25005.0  |
| zap                   | 1617.1   | 7982.4   | 15682.8  | 31673.9   | 43278.4  |
| zerolog               | 184.3    | 297.8    | 743.4    | 5156.9    | 25172.8  |

### mixed: 1 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| quill                 | 31.5     | 66.6     | 83.2     | 436.0     | 583717.6 |
| ticklog               | 16.9     | 17.6     | 34.6     | 57.9      | 99.9     |
| ticklog_file          | 15.6     | 19.0     | 37.7     | 59.0      | 218.2    |
| ticklog_ringbuf       | 24.7     | 27.0     | 88.8     | 100.9     | 174.2    |
| ticklog_triple-buffer | 1712.2   | 1789.7   | 1853.3   | 2446.9    | 4373.1   |
| zap                   | 785.6    | 2069.4   | 3058.1   | 3630.3    | 13806.9  |
| zerolog               | 198.4    | 420.5    | 525.3    | 811.2     | 25166.8  |

### mixed: 2 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| quill                 | 32.7     | 74.7     | 97.3     | 5649.4    | 455551.3 |
| ticklog               | 16.9     | 19.4     | 27.9     | 51.8      | 145.9    |
| ticklog_file          | 15.5     | 19.0     | 28.8     | 64.6      | 383.4    |
| ticklog_ringbuf       | 24.3     | 26.8     | 57.9     | 105.4     | 231.9    |
| ticklog_triple-buffer | 2779.6   | 3718.3   | 4376.5   | 13086.4   | 113924.8 |
| zap                   | 1128.6   | 2520.8   | 3669.7   | 17619.3   | 19086.7  |
| zerolog               | 198.6    | 418.4    | 492.3    | 807.7     | 1365.1   |

### mixed: 4 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| quill                 | 35.4     | 65.9     | 103.1    | 119375.8  | 785397.6 |
| ticklog               | 17.1     | 19.4     | 25.9     | 80.8      | 196.9    |
| ticklog_file          | 17.9     | 19.5     | 33.6     | 131.7     | 768.2    |
| ticklog_ringbuf       | 24.6     | 27.9     | 36.2     | 104.8     | 487.9    |
| ticklog_triple-buffer | 6067.6   | 7275.2   | 7563.6   | 9259.5    | 11148.5  |
| zap                   | 1320.8   | 2997.9   | 9565.3   | 19257.8   | 28427.0  |
| zerolog               | 200.2    | 418.8    | 491.7    | 736.2     | 1183.8   |

### mixed: 8 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| quill                 | 60.5     | 87.2     | 579.9    | 98661.8   | 317990.8 |
| ticklog               | 19.4     | 19.6     | 37.0     | 137.5     | 738.8    |
| ticklog_file          | 19.3     | 19.8     | 92.5     | 363.4     | 2961.8   |
| ticklog_ringbuf       | 27.8     | 28.0     | 85.6     | 146.9     | 297.6    |
| ticklog_triple-buffer | 12832.0  | 19941.1  | 30151.7  | 172201.8  | 287945.1 |
| zap                   | 2215.9   | 4313.9   | 13795.6  | 23504.6   | 25985.7  |
| zerolog               | 234.2    | 443.8    | 480.1    | 624.1     | 825.9    |

### mixed: 16 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| quill                 | 58.1     | 104.0    | 5108.1   | 587914.9  | 779930.3 |
| ticklog               | 19.4     | 19.9     | 74.5     | 337.2     | 9410.5   |
| ticklog_file          | 19.3     | 20.5     | 167.1    | 769.1     | 8591.4   |
| ticklog_ringbuf       | 27.8     | 90.8     | 121.2    | 1201.8    | 15684.9  |
| ticklog_triple-buffer | 27939.5  | 30188.0  | 30483.8  | 32185.2   | 35669.7  |
| zap                   | 3349.2   | 12532.8  | 22400.5  | 42174.4   | 67081.2  |
| zerolog               | 394.4    | 484.1    | 1412.9   | 18205.9   | 70744.4  |

### string: 1 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| quill                 | 32.7     | 72.5     | 84.3     | 376.6     | 216713.5 |
| ticklog               | 16.5     | 17.4     | 40.2     | 51.5      | 91.0     |
| ticklog_file          | 17.0     | 20.9     | 37.1     | 62.4      | 347.2    |
| ticklog_ringbuf       | 24.2     | 26.2     | 85.7     | 92.7      | 186.5    |
| ticklog_triple-buffer | 1277.5   | 1366.3   | 1619.4   | 2114.5    | 2442.9   |
| zap                   | 458.8    | 1179.4   | 2078.7   | 2499.2    | 6039.6   |
| zerolog               | 94.1     | 186.5    | 241.9    | 423.9     | 1319.0   |

### string: 2 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| quill                 | 33.6     | 61.7     | 98.2     | 4152.8    | 327729.0 |
| ticklog               | 16.4     | 17.4     | 28.0     | 53.5      | 108.8    |
| ticklog_file          | 15.0     | 17.2     | 24.9     | 77.0      | 250.8    |
| ticklog_ringbuf       | 23.9     | 25.9     | 74.4     | 100.8     | 347.0    |
| ticklog_triple-buffer | 1916.9   | 2571.2   | 2724.3   | 3485.1    | 5997.2   |
| zap                   | 542.6    | 1513.9   | 2268.9   | 11722.1   | 17183.2  |
| zerolog               | 94.0     | 186.5    | 220.9    | 392.7     | 877.2    |

### string: 4 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| quill                 | 54.2     | 64.0     | 114.8    | 91603.4   | 347281.1 |
| ticklog               | 16.5     | 19.0     | 25.5     | 82.6      | 226.2    |
| ticklog_file          | 17.2     | 18.9     | 28.7     | 140.2     | 873.4    |
| ticklog_ringbuf       | 23.7     | 26.8     | 36.8     | 105.3     | 130.3    |
| ticklog_triple-buffer | 4068.0   | 4938.9   | 5588.5   | 7776.1    | 11607.6  |
| zap                   | 790.4    | 1960.6   | 5233.7   | 17849.0   | 28253.9  |
| zerolog               | 94.2     | 186.8    | 215.8    | 321.7     | 676.5    |

### string: 8 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| quill                 | 56.2     | 78.2     | 294.1    | 55892.6   | 256034.2 |
| ticklog               | 17.3     | 19.1     | 42.4     | 182.0     | 1366.1   |
| ticklog_file          | 18.1     | 19.0     | 46.8     | 254.2     | 1344.5   |
| ticklog_ringbuf       | 26.1     | 71.2     | 104.3    | 252.7     | 1260.9   |
| ticklog_triple-buffer | 8481.7   | 10109.8  | 10284.4  | 14117.9   | 15684.7  |
| zap                   | 858.8    | 2517.8   | 5647.4   | 18477.7   | 28667.8  |
| zerolog               | 174.8    | 267.2    | 337.3    | 935.3     | 6231.4   |

### string: 16 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| quill                 | 55.3     | 74.1     | 1486.8   | 329058.7  | 460322.5 |
| ticklog               | 18.9     | 19.3     | 75.1     | 592.6     | 9828.3   |
| ticklog_file          | 18.8     | 20.6     | 159.1    | 890.2     | 10462.1  |
| ticklog_ringbuf       | 26.7     | 84.7     | 116.2    | 791.3     | 14287.1  |
| ticklog_triple-buffer | 18061.2  | 20875.8  | 23501.5  | 28509.0   | 46158.2  |
| zap                   | 1626.0   | 8530.2   | 19519.8  | 39956.0   | 60144.3  |
| zerolog               | 179.9    | 282.1    | 424.7    | 7956.1    | 28088.4  |

## Throughput (Run 3)

### 1 thread(s)

| Candidate             | single_int (r/s) | mixed (r/s) | string (r/s) |
|-----------------------|------------------|-------------|--------------|
| quill                 | 20,166,031       | 7,111,281   | 11,710,574   |
| ticklog               | 58,149,779       | 59,273,239  | 57,724,189   |
| ticklog_file          | 61,356,770       | 58,535,637  | 57,841,316   |
| ticklog_ringbuf       | 38,018,043       | 38,753,081  | 40,770,497   |
| ticklog_triple-buffer | 890,912          | 605,771     | 780,671      |
| zap                   | 1,623,818        | 953,386     | 1,603,942    |
| zerolog               | 8,227,591        | 4,000,072   | 8,792,255    |

### 2 thread(s)

| Candidate             | single_int (r/s) | mixed (r/s) | string (r/s) |
|-----------------------|------------------|-------------|--------------|
| quill                 | 18,099,462       | 8,906,028   | 10,684,427   |
| ticklog               | 122,618,894      | 114,968,694 | 115,276,034  |
| ticklog_file          | 106,977,157      | 117,626,993 | 124,640,878  |
| ticklog_ringbuf       | 78,552,189       | 89,726,898  | 75,948,367   |
| ticklog_triple-buffer | 848,667          | 702,106     | 1,035,216    |
| zap                   | 2,693,883        | 1,533,294   | 2,667,162    |
| zerolog               | 16,174,412       | 7,995,332   | 17,263,399   |

### 4 thread(s)

| Candidate             | single_int (r/s) | mixed (r/s) | string (r/s) |
|-----------------------|------------------|-------------|--------------|
| quill                 | 9,826,165        | 6,968,059   | 13,815,080   |
| ticklog               | 228,704,208      | 221,190,002 | 218,006,462  |
| ticklog_file          | 201,636,888      | 210,616,770 | 206,570,598  |
| ticklog_ringbuf       | 160,819,795      | 151,174,322 | 163,058,721  |
| ticklog_triple-buffer | 1,030,009        | 695,949     | 1,007,655    |
| zap                   | 4,395,558        | 2,394,194   | 4,091,957    |
| zerolog               | 25,854,277       | 14,630,584  | 32,336,130   |

### 8 thread(s)

| Candidate             | single_int (r/s) | mixed (r/s) | string (r/s) |
|-----------------------|------------------|-------------|--------------|
| quill                 | 5,212,412        | 11,550,078  | 22,114,246   |
| ticklog               | 359,923,264      | 374,995,313 | 368,828,122  |
| ticklog_file          | 279,068,469      | 338,297,282 | 372,517,173  |
| ticklog_ringbuf       | 275,043,388      | 268,343,275 | 138,185,677  |
| ticklog_triple-buffer | 811,652          | 577,497     | 1,026,096    |
| zap                   | 5,570,300        | 3,199,989   | 6,465,800    |
| zerolog               | 43,528,603       | 25,214,271  | 44,146,095   |

### 16 thread(s)

| Candidate             | single_int (r/s) | mixed (r/s) | string (r/s) |
|-----------------------|------------------|-------------|--------------|
| quill                 | 22,900,221       | 7,903,430   | 14,071,296   |
| ticklog               | 379,136,857      | 419,450,771 | 404,110,613  |
| ticklog_file          | 387,308,670      | 414,770,818 | 394,589,390  |
| ticklog_ringbuf       | 224,120,774      | 201,172,433 | 216,967,275  |
| ticklog_triple-buffer | 817,216          | 571,013     | 857,783      |
| zap                   | 6,138,713        | 3,383,214   | 5,883,630    |
| zerolog               | 49,717,209       | 29,192,205  | 60,534,776   |

## Thread Scaling (Run 3)

| Candidate             | 1 thread   | 2 threads   | 4 threads   | 8 threads   | 16 threads  | scale 1->16 |
|-----------------------|------------|-------------|-------------|-------------|-------------|-------------|
| quill                 | 20,166,031 | 18,099,462  | 9,826,165   | 5,212,412   | 22,900,221  | 1.14x       |
| ticklog               | 58,149,779 | 122,618,894 | 228,704,208 | 359,923,264 | 379,136,857 | 6.52x       |
| ticklog_file          | 61,356,770 | 106,977,157 | 201,636,888 | 279,068,469 | 387,308,670 | 6.31x       |
| ticklog_ringbuf       | 38,018,043 | 78,552,189  | 160,819,795 | 275,043,388 | 224,120,774 | 5.90x       |
| ticklog_triple-buffer | 890,912    | 848,667     | 1,030,009   | 811,652     | 817,216     | 0.92x       |
| zap                   | 1,623,818  | 2,693,883   | 4,395,558   | 5,570,300   | 6,138,713   | 3.78x       |
| zerolog               | 8,227,591  | 16,174,412  | 25,854,277  | 43,528,603  | 49,717,209  | 6.04x       |

## Jitter (Run 3)

| Candidate             | p99 (ns) | p999 (ns) | max (ns) | p99/p50 |
|-----------------------|----------|-----------|----------|---------|
| quill                 | 84.3     | 459.1     | 583717.6 | 2.6x    |
| ticklog               | 40.2     | 57.9      | 99.9     | 2.4x    |
| ticklog_file          | 37.7     | 66.9      | 347.2    | 2.4x    |
| ticklog_ringbuf       | 88.8     | 100.9     | 186.5    | 3.6x    |
| ticklog_triple-buffer | 1853.3   | 2446.9    | 6939.4   | 1.3x    |
| zap                   | 3058.1   | 3630.3    | 13806.9  | 4.7x    |
| zerolog               | 525.3    | 811.2     | 25166.8  | 2.6x    |

## Two-Core SPSC (Run 3)

Ticklog variants: producer pinned core 0, drain pinned core 1. Inline loggers: process affinity core 0.

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns)  | r/s        |
|-----------------------|----------|----------|----------|-----------|-----------|------------|
| quill                 | 15.65    | 22.16    | 38.04    | 1712.33   | 104568.91 | 19,506,425 |
| ticklog               | 17.25    | 17.66    | 34.07    | 471.11    | 1354.59   | 52,127,071 |
| ticklog_file          | 17.25    | 17.48    | 31.12    | 188.52    | 510.79    | 54,944,542 |
| ticklog_ringbuf       | 25.10    | 31.39    | 79.44    | 221.66    | 880.39    | 36,178,560 |
| ticklog_triple-buffer | 873.83   | 1161.29  | 2219.24  | 3514.08   | 9363.41   | 1,067,978  |
| zap                   | 638.31   | 1412.31  | 2271.93  | 34554.02  | 206833.86 | 1,101,104  |
| zerolog               | 118.49   | 208.14   | 306.10   | 886.49    | 2383.93   | 7,604,780  |


## Notes (Run 3)

- Native Windows run: no core isolation, no perf, OS scheduler is Windows'.
  ticklog two-core placement uses --producer-core 0 / --backend-core 1;
  inline loggers (zerolog/zap/quill) ran with process affinity set to core 0.
- Raw JSON outputs are committed under
  `harness-results/windows/` (`sweep/` and `two-core-pinned/`), each file
  tagged `"os": "windows"`.
- quill shows very high max/p999 tails at 2-16 threads (SPSC queue doubling to
  512 MiB per producer) but remains ~low p50; same pattern as prior runs.
- ticklog_triple-buffer does not scale (0.92x) -- the triple_buffer crate's
  farmer thread switches toggled under load; kept for completeness.
- For thread-scaling honesty: only single run per config, no pinning of the
  measuring process beyond the notes above.

# Run 4 - WSL2 Ubuntu (same host)

Clean rerun in WSL2 (Ubuntu 26.04 LTS, kernel 6.18.33.2-microsoft-standard-WSL2)
on the SAME physical machine as Run 3 (AMD Ryzen 7 7735HS). 16 vCPUs exposed to
WSL, 6 GB RAM. No concurrent Windows load during measurement.
Same protocol as Run 1/2/3.
**Run 2 (an earlier WSL2 VM) was on different underlying hardware -- do not
compare Run 2 against Run 3/4 absolute numbers.**

Toolchain: rustc 1.99 nightly (rust-toolchain.toml), gcc/g++ for nanolog, Go 1.27
for zerolog/zap. `perf`/`cpupower` unavailable in this kernel -- no hardware
counters, no governor lock; numbers are best-effort. quill skipped (its SPSC
queues exceed the 6 GB WSL RAM, same OOM hazard that killed Run 2's VM).


**Host:** AMD Ryzen 7 7735HS (8C/16T), 13.3 GB RAM

**OS:** linux (x86_64)  
**Clock:** rdtsc, ns_per_tick = 0.313097  
**Batch:** 1000 msgs, **samples:** 10000, **total:** 10,000,000/config  

## Latency (Run 4)

### single_int: 1 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| nanolog               | 15.3     | 17.9     | 48.4     | 159.5     | 401.1    |
| ticklog               | 15.2     | 19.4     | 39.7     | 160.2     | 345.8    |
| ticklog_file          | 15.3     | 18.1     | 40.5     | 109.1     | 367.2    |
| ticklog_ringbuf       | 15.7     | 43.9     | 68.4     | 129.5     | 250.7    |
| ticklog_triple-buffer | 711.0    | 1070.5   | 1619.1   | 2237.5    | 5793.0   |
| zap                   | 612.9    | 1152.8   | 2102.7   | 3080.8    | 5288.5   |
| zerolog               | 94.5     | 194.3    | 278.7    | 486.9     | 8444.4   |

### single_int: 2 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| nanolog               | 15.7     | 39.9     | 596.1    | 956.6     | 1938.4   |
| ticklog               | 16.6     | 18.6     | 45.7     | 234.3     | 458.3    |
| ticklog_file          | 16.1     | 18.5     | 43.4     | 119.0     | 304.1    |
| ticklog_ringbuf       | 15.7     | 25.1     | 68.6     | 142.6     | 362.1    |
| ticklog_triple-buffer | 1431.0   | 1948.6   | 2403.8   | 3413.1    | 5474.0   |
| zap                   | 676.5    | 1374.1   | 2618.3   | 3179.1    | 3922.9   |
| zerolog               | 104.0    | 211.4    | 324.7    | 631.8     | 1262.6   |

### single_int: 4 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| nanolog               | 16.8     | 41.4     | 2613.0   | 2742.3    | 13973.8  |
| ticklog               | 16.6     | 18.3     | 47.3     | 205.5     | 320.6    |
| ticklog_file          | 16.1     | 18.2     | 44.9     | 143.8     | 262.6    |
| ticklog_ringbuf       | 21.4     | 25.0     | 71.3     | 141.7     | 470.7    |
| ticklog_triple-buffer | 3091.2   | 3509.7   | 3998.3   | 4563.4    | 15857.0  |
| zap                   | 720.8    | 2593.2   | 3146.1   | 3768.7    | 4434.1   |
| zerolog               | 95.7     | 201.8    | 269.9    | 551.6     | 1403.5   |

### single_int: 8 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| nanolog               | 17.2     | 24.4     | 5432.5   | 6265.3    | 6959.4   |
| ticklog               | 17.9     | 18.2     | 60.4     | 472.6     | 3922.0   |
| ticklog_file          | 17.9     | 18.2     | 50.2     | 198.2     | 2473.7   |
| ticklog_ringbuf       | 24.8     | 25.1     | 114.0    | 356.3     | 2446.3   |
| ticklog_triple-buffer | 6137.8   | 6866.2   | 7462.6   | 12652.7   | 14328.0  |
| zap                   | 1022.2   | 3660.6   | 4157.1   | 4751.7    | 5731.1   |
| zerolog               | 179.7    | 271.0    | 408.6    | 3405.1    | 12942.1  |

### single_int: 16 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| nanolog               | 17.3     | 42.2     | 24780.2  | 36279.6   | 54701.3  |
| ticklog               | 17.9     | 18.2     | 64.7     | 764.5     | 6598.7   |
| ticklog_file          | 17.9     | 18.2     | 68.3     | 2000.6    | 8063.5   |
| ticklog_ringbuf       | 24.8     | 48.9     | 191.3    | 2079.8    | 8059.0   |
| ticklog_triple-buffer | 13927.2  | 20453.2  | 31028.7  | 116453.9  | 191520.0 |
| zap                   | 1522.0   | 6768.1   | 12529.0  | 22540.3   | 27072.1  |
| zerolog               | 184.2    | 281.9    | 664.1    | 5070.8    | 11481.8  |

### mixed: 1 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| nanolog               | 16.9     | 75.4     | 488.9    | 820.4     | 1350.1   |
| ticklog               | 16.7     | 24.3     | 48.5     | 190.2     | 1068.0   |
| ticklog_file          | 16.7     | 18.3     | 45.0     | 143.9     | 1747.6   |
| ticklog_ringbuf       | 15.6     | 32.0     | 68.4     | 165.3     | 699.2    |
| ticklog_triple-buffer | 915.1    | 1456.5   | 2049.8   | 3194.8    | 9108.1   |
| zap                   | 815.5    | 2020.6   | 2839.5   | 3814.6    | 4500.2   |
| zerolog               | 208.4    | 415.4    | 589.5    | 1117.1    | 1766.5   |

### mixed: 2 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| nanolog               | 17.1     | 322.4    | 828.2    | 2174.4    | 2889.4   |
| ticklog               | 16.1     | 20.4     | 51.4     | 282.2     | 1209.2   |
| ticklog_file          | 15.0     | 21.9     | 47.8     | 118.5     | 385.0    |
| ticklog_ringbuf       | 15.9     | 25.4     | 67.9     | 151.5     | 745.4    |
| ticklog_triple-buffer | 1986.9   | 2652.9   | 3292.2   | 4389.9    | 15573.4  |
| zap                   | 911.1    | 2712.4   | 3305.4   | 4242.4    | 7520.1   |
| zerolog               | 226.2    | 427.8    | 544.1    | 865.9     | 1321.2   |

### mixed: 4 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| nanolog               | 18.0     | 185.9    | 1958.4   | 6852.9    | 54647.3  |
| ticklog               | 17.9     | 19.4     | 45.9     | 195.3     | 294.9    |
| ticklog_file          | 17.9     | 19.4     | 44.2     | 147.2     | 591.3    |
| ticklog_ringbuf       | 22.1     | 25.7     | 74.8     | 144.0     | 445.9    |
| ticklog_triple-buffer | 4002.7   | 4852.4   | 5423.1   | 7323.4    | 12002.0  |
| zap                   | 1254.7   | 3520.5   | 3914.9   | 4404.3    | 5004.5   |
| zerolog               | 224.6    | 421.6    | 534.1    | 1049.0    | 2460.1   |

### mixed: 8 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| nanolog               | 18.3     | 185.2    | 4908.4   | 38673.6   | 48478.5  |
| ticklog               | 19.2     | 19.4     | 56.7     | 361.0     | 2285.0   |
| ticklog_file          | 19.1     | 19.4     | 61.8     | 258.0     | 1576.9   |
| ticklog_ringbuf       | 24.8     | 26.1     | 111.2    | 291.8     | 1850.8   |
| ticklog_triple-buffer | 8460.2   | 9404.4   | 9850.0   | 11626.9   | 12790.0  |
| zap                   | 2911.6   | 4381.4   | 4862.8   | 7931.4    | 9446.5   |
| zerolog               | 359.3    | 444.1    | 541.2    | 748.0     | 3343.4   |

### mixed: 16 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| nanolog               | 19.1     | 207.2    | 22148.9  | 128749.2  | 138682.8 |
| ticklog               | 19.2     | 19.6     | 106.4    | 1218.9    | 4864.9   |
| ticklog_file          | 19.2     | 19.5     | 86.0     | 2300.2    | 8063.1   |
| ticklog_ringbuf       | 25.5     | 28.7     | 228.3    | 703.2     | 15041.7  |
| ticklog_triple-buffer | 18917.8  | 27432.2  | 38585.4  | 135410.4  | 197633.3 |
| zap                   | 4722.1   | 13273.3  | 18760.5  | 27344.7   | 39619.0  |
| zerolog               | 383.8    | 463.1    | 1012.2   | 3940.5    | 7654.7   |

### string: 1 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| nanolog               | 16.5     | 26.6     | 201.3    | 710.0     | 1501.5   |
| ticklog               | 15.1     | 23.7     | 45.7     | 98.8      | 145.3    |
| ticklog_file          | 16.6     | 21.3     | 41.2     | 108.2     | 421.1    |
| ticklog_ringbuf       | 15.7     | 45.2     | 72.0     | 147.8     | 974.0    |
| ticklog_triple-buffer | 626.5    | 938.5    | 1295.8   | 2637.0    | 4762.6   |
| zap                   | 597.5    | 1141.4   | 2094.9   | 2861.2    | 3978.7   |
| zerolog               | 109.6    | 190.1    | 288.5    | 503.4     | 965.9    |

### string: 2 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| nanolog               | 16.4     | 75.0     | 826.8    | 2431.6    | 17986.4  |
| ticklog               | 16.0     | 17.6     | 45.6     | 116.0     | 787.8    |
| ticklog_file          | 14.9     | 18.3     | 47.7     | 97.8      | 273.8    |
| ticklog_ringbuf       | 15.7     | 25.2     | 65.4     | 105.2     | 164.4    |
| ticklog_triple-buffer | 1296.2   | 1789.2   | 2194.2   | 2894.2    | 5559.8   |
| zap                   | 635.5    | 1289.1   | 2517.4   | 3259.2    | 4154.5   |
| zerolog               | 94.9     | 203.1    | 301.1    | 588.5     | 1038.2   |

### string: 4 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| nanolog               | 16.2     | 66.8     | 2482.3   | 3412.7    | 44534.0  |
| ticklog               | 16.2     | 18.5     | 44.0     | 121.5     | 335.1    |
| ticklog_file          | 16.0     | 18.5     | 42.8     | 166.8     | 3911.5   |
| ticklog_ringbuf       | 15.7     | 24.8     | 73.4     | 149.1     | 510.7    |
| ticklog_triple-buffer | 2914.4   | 3478.6   | 4139.2   | 5330.5    | 6303.6   |
| zap                   | 789.9    | 2711.6   | 3307.9   | 4117.4    | 7060.1   |
| zerolog               | 102.7    | 203.3    | 287.5    | 505.1     | 1157.6   |

### string: 8 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| nanolog               | 17.3     | 51.1     | 5316.5   | 29253.7   | 31208.3  |
| ticklog               | 16.8     | 18.5     | 71.3     | 301.1     | 1044.5   |
| ticklog_file          | 18.2     | 27.5     | 60.6     | 244.2     | 1344.5   |
| ticklog_ringbuf       | 22.4     | 24.9     | 125.4    | 219.1     | 389.9    |
| ticklog_triple-buffer | 5799.7   | 6255.5   | 6764.7   | 7462.9    | 9326.8   |
| zap                   | 1018.3   | 3640.9   | 4217.1   | 5675.1    | 11925.3  |
| zerolog               | 174.1    | 246.1    | 314.3    | 452.2     | 615.4    |

### string: 16 thread(s)

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) |
|-----------------------|----------|----------|----------|-----------|----------|
| nanolog               | 17.6     | 92.6     | 20950.2  | 48966.6   | 59437.0  |
| ticklog               | 18.2     | 18.4     | 252.0    | 552.4     | 7406.5   |
| ticklog_file          | 18.2     | 18.5     | 264.0    | 474.0     | 4054.7   |
| ticklog_ringbuf       | 24.7     | 36.6     | 214.7    | 998.8     | 8054.9   |
| ticklog_triple-buffer | 12384.2  | 17729.5  | 22148.5  | 60523.2   | 197355.5 |
| zap                   | 1683.6   | 7505.5   | 13469.1  | 23384.3   | 31916.6  |
| zerolog               | 181.5    | 275.3    | 483.0    | 3053.0    | 6171.2   |

## Throughput (Run 4)

### 1 thread(s)

| Candidate             | single_int (r/s) | mixed (r/s) | string (r/s) |
|-----------------------|------------------|-------------|--------------|
| nanolog               | 58,219,813       | 30,552,492  | 41,888,333   |
| ticklog               | 60,771,479       | 55,560,649  | 59,340,999   |
| ticklog_file          | 59,725,800       | 55,946,520  | 57,631,913   |
| ticklog_ringbuf       | 49,447,247       | 51,744,752  | 48,393,072   |
| ticklog_triple-buffer | 1,296,989        | 985,850     | 1,475,694    |
| zap                   | 1,366,575        | 990,853     | 1,399,927    |
| zerolog               | 8,581,833        | 4,081,329   | 8,224,165    |

### 2 thread(s)

| Candidate             | single_int (r/s) | mixed (r/s) | string (r/s) |
|-----------------------|------------------|-------------|--------------|
| nanolog               | 62,990,525       | 35,429,455  | 42,475,183   |
| ticklog               | 110,667,349      | 108,093,475 | 116,688,896  |
| ticklog_file          | 115,202,624      | 113,054,399 | 121,105,145  |
| ticklog_ringbuf       | 97,488,836       | 101,284,249 | 95,294,662   |
| ticklog_triple-buffer | 1,351,300        | 989,835     | 1,481,929    |
| zap                   | 2,395,475        | 1,660,733   | 2,516,902    |
| zerolog               | 14,788,263       | 7,217,682   | 16,263,660   |

### 4 thread(s)

| Candidate             | single_int (r/s) | mixed (r/s) | string (r/s) |
|-----------------------|------------------|-------------|--------------|
| nanolog               | 50,173,732       | 30,663,943  | 39,096,559   |
| ticklog               | 204,552,810      | 206,153,037 | 216,575,235  |
| ticklog_file          | 216,170,061      | 198,542,389 | 215,732,574  |
| ticklog_ringbuf       | 160,714,077      | 158,647,649 | 159,421,860  |
| ticklog_triple-buffer | 1,364,619        | 1,038,155   | 1,416,975    |
| zap                   | 4,195,586        | 2,483,184   | 4,057,875    |
| zerolog               | 29,956,838       | 14,312,441  | 28,105,999   |

### 8 thread(s)

| Candidate             | single_int (r/s) | mixed (r/s) | string (r/s) |
|-----------------------|------------------|-------------|--------------|
| nanolog               | 58,615,880       | 28,175,625  | 42,169,042   |
| ticklog               | 312,562,903      | 334,859,870 | 375,788,432  |
| ticklog_file          | 360,158,282      | 362,085,964 | 272,032,504  |
| ticklog_ringbuf       | 263,877,858      | 274,371,366 | 291,008,487  |
| ticklog_triple-buffer | 1,415,190        | 1,027,493   | 1,499,800    |
| zap                   | 5,834,548        | 3,018,413   | 5,849,841    |
| zerolog               | 39,149,552       | 23,363,622  | 46,347,748   |

### 16 thread(s)

| Candidate             | single_int (r/s) | mixed (r/s) | string (r/s) |
|-----------------------|------------------|-------------|--------------|
| nanolog               | 29,442,835       | 16,586,072  | 27,588,984   |
| ticklog               | 389,910,595      | 359,560,413 | 419,294,490  |
| ticklog_file          | 371,773,761      | 378,459,938 | 397,182,846  |
| ticklog_ringbuf       | 308,324,306      | 302,845,164 | 336,134,013  |
| ticklog_triple-buffer | 1,062,224        | 783,127     | 1,206,721    |
| zap                   | 5,750,845        | 2,699,122   | 5,741,937    |
| zerolog               | 62,409,991       | 37,082,014  | 68,666,775   |

## Thread Scaling (Run 4)

| Candidate             | 1 thread   | 2 threads   | 4 threads   | 8 threads   | 16 threads  | scale 1->16 |
|-----------------------|------------|-------------|-------------|-------------|-------------|-------------|
| nanolog               | 58,219,813 | 62,990,525  | 50,173,732  | 58,615,880  | 29,442,835  | 0.51x       |
| ticklog               | 60,771,479 | 110,667,349 | 204,552,810 | 312,562,903 | 389,910,595 | 6.42x       |
| ticklog_file          | 59,725,800 | 115,202,624 | 216,170,061 | 360,158,282 | 371,773,761 | 6.22x       |
| ticklog_ringbuf       | 49,447,247 | 97,488,836  | 160,714,077 | 263,877,858 | 308,324,306 | 6.24x       |
| ticklog_triple-buffer | 1,296,989  | 1,351,300   | 1,364,619   | 1,415,190   | 1,062,224   | 0.82x       |
| zap                   | 1,366,575  | 2,395,475   | 4,195,586   | 5,834,548   | 5,750,845   | 4.21x       |
| zerolog               | 8,581,833  | 14,788,263  | 29,956,838  | 39,149,552  | 62,409,991  | 7.27x       |

## Jitter (Run 4)

| Candidate             | p99 (ns) | p999 (ns) | max (ns) | p99/p50 |
|-----------------------|----------|-----------|----------|---------|
| nanolog               | 488.9    | 820.4     | 1501.5   | 29.0x   |
| ticklog               | 48.5     | 190.2     | 1068.0   | 3.0x    |
| ticklog_file          | 45.0     | 143.9     | 1747.6   | 2.7x    |
| ticklog_ringbuf       | 72.0     | 165.3     | 974.0    | 4.6x    |
| ticklog_triple-buffer | 2049.8   | 3194.8    | 9108.1   | 2.3x    |
| zap                   | 2839.5   | 3814.6    | 5288.5   | 3.5x    |
| zerolog               | 589.5    | 1117.1    | 8444.4   | 2.9x    |

## Two-Core SPSC (Run 4)

Ticklog variants: producer pinned core 0, drain pinned core 1. Inline loggers: taskset core 0. quill skipped (Linux build hazard).

| Candidate             | p50 (ns) | p95 (ns) | p99 (ns) | p999 (ns) | max (ns) | r/s        |
|-----------------------|----------|----------|----------|-----------|----------|------------|
| nanolog               | 15.27    | 123.73   | 710.19   | 6251.56   | 7608.50  | 13,047,976 |
| ticklog               | 15.24    | 18.12    | 38.86    | 98.81     | 148.63   | 60,879,184 |
| ticklog_file          | 15.25    | 18.06    | 45.31    | 134.52    | 680.57   | 58,784,672 |
| ticklog_ringbuf       | 15.77    | 43.69    | 68.43    | 120.97    | 269.94   | 46,582,108 |
| ticklog_triple-buffer | 707.37   | 1034.35  | 1393.31  | 2797.80   | 23464.91 | 1,316,590  |
| zap                   | 604.69   | 1546.81  | 3085.96  | 11837.69  | 24955.97 | 1,176,403  |
| zerolog               | 94.41    | 190.23   | 265.71   | 620.17    | 1528.61  | 9,112,765  |


## Notes (Run 4)

- Two-core placement for ticklog: producer pinned --cpu 0, drain pinned
  --drain-cpu 1 (taskset). Inline loggers pinned with taskset -c 0.
  ticklog_file sink written to the ext4 root fs (avoided the run-2 ENOSPC
  tmpfs failure that had corrupted its pinned numbers).
- nanolog high-thread (8/16) p999/max floods are the known single-free-list
  contention; fine at 1-2 threads.
- WSL advertises 16 CPUs (with hyperthreading) but only ~8 real cores:
  16-thread gains over 8-thread are modest, as expected.

