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
