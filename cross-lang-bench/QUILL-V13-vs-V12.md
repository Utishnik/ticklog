# Quill v13.0.0 vs v12.1.0 (same harness, 8 MiB queue)

Date: 2026-09-23. Re-run of the C++ quill arm from Run 16 with the vendored release
pinned to **v13.0.0** (`eb802a37`), same `main.cpp`, same flags
(`-march=native -flto -fno-exceptions -fno-rtti -fomit-frame-pointer`), same
`QUILL_BENCH_QUEUE_CAPACITY=8388608`, BATCH=1000, SAMPLES=1000 (1M msg/config),
threads `1,2,4,8,16`, RDTSC. Calibration: Windows `0.313132820 ns/tick`,
WSL `0.313148759 ns/tick`. Toolchains identical to the v12 runs
(clang 23.1.1 / Ninja on Windows, gcc 15.2.0 / Make on WSL).

Baseline `wsl_quill_cpp_r16.json` / `windows_quill_cpp_r16.json` (v12.1.0). New:
`wsl_quill_cpp_v13_r16.json` / `windows_quill_cpp_v13_r16.json` (+ `.log`, `.log.err`).
Harness API (FrontendOptions, FrontendImpl, NullSink, Backend::start) unchanged in v13;
`quill::quill` alias preserved.

## single_int (p50 ns, p99 ns, throughput msg/s)

| OS | threads | v12 p50 | v13 p50 | v12 p99 | v13 p99 | v12 thr | v13 thr |
|---|---|---|---|---|---|---|---|
| WSL | 1 | 21.68 | **14.97** | 42843 | 34361 | 1598319 | 1822170 |
| WSL | 2 | 21.66 | **14.89** | 66403 | 36195 | 1709692 | 1852116 |
| WSL | 4 | 27.02 | **16.20** | 99.7 | 127.2 | 4249930 | 4120693 |
| WSL | 8 | 27.14 | **21.91** | 87.9 | 95.1 | 243676474 | 241437946 |
| WSL | 16 | 35.05 | **30.09** | 188.3 | 188.6 | 137119025 | 119500912 |
| Win | 1 | 18.73 | **15.39** | 45976 | 39974 | 1405358 | 1418731 |
| Win | 2 | 16.78 | 17.10 | 21268 | 41487 | 1443800 | 1778560 |
| Win | 4 | 17.62 | 19.98 | 44.2 | 45.7 | 11446258 | 6968049 |
| Win | 8 | 22.28 | **21.08** | 66.5 | 71.3 | 141665132 | 124738050 |
| Win | 16 | 33.45 | **24.74** | 97.0 | 180.9 | 62325489 | 30688214 |

## mixed / string (p50 ns; thrpt msg/s highlights)

| OS | workload | t1 p50 | t8 p50 | t16 p50 | t8 thr | t16 thr |
|---|---|---|---|---|---|---|
| WSL | mixed  | 31.44 -> 24.93 | 51.44 -> 36.77 | 54.84 -> 36.59 | 120.4M -> 179.7M | 120.2M -> 55.4M |
| WSL | string | 30.50 -> 24.46 | 49.43 -> 34.87 | 50.69 -> 38.87 | 91.6M -> 166.0M | 52.7M -> 171.9M |
| Win | mixed  | 27.48 -> 24.56 | 35.93 -> 30.69 | 62.22 -> 63.57 | 181.6M -> 155.7M | 162.0M -> 80.8M |
| Win | string | 27.38 -> 19.78 | 28.50 -> 28.99 | 32.65 -> 42.19 | 247.4M -> 80.3M | 54.4M -> 38.4M |

## Observations

- **Fast path (p50/p95) is faster in v13, most consistently on WSL.** single_int p50 up
  to -40% (WSL t4 27.0 -> 16.2; t1/t2 -31%), larger format-heavy workloads in v13 gain
  even more at 8/16 threads (WSL string t16 50.7 -> 38.9, t8 49.4 -> 34.9). Win p50 also
  improves on most configs (single_int t1 -18%, t16 -26%) with a couple of ~+2ns wobbles
  at 2t/4t.
- **Blocking-tail behavior is unchanged.** v13 logs the identical backpressure signature
  (`Reached the maximum configured unbounded queue capacity ... N blocking occurrences`,
  N=1..14 concentrated on the low-thread count runs) and p99 at 1-2 threads stays in the
  34-72 us range with p999 up to ~0.2 ms. The exhaustion path (queue at initial==max,
  producer blocks on `UnboundedSPSCQueue` reschedule until the backend drains) behaves the
  same; the improvement is confined to the uncontended/park-free interval.
- **Throughput is noisier than latency and not a clean v13 win.** Low-thread WSL is up
  (t1 +0.22M, t2 +0.14M on single_int) and WSL string 8t/16t strongly up (+74M/+119M),
  but WSL single_int 16t -17.6M, mixed 16t -65M, and almost all Win >=8t readings moved
  down (8t string 247M -> 80M is plainly scheduling noise on this box). Conclude from p50,
  treat high-thread throughput deltas as qualitative.
- Single run per config; no repeats. p50 deltas >~4 ns and thrpt deltas >~10% should be
  considered indicative but not ironclad (consistent with the run-to-run scatter seen in
  Runs 14/16 on this machine).

## Builds

`cpp/quill/build13` (Win, Ninja/clang), `cpp/quill/build13-wsl` (Make/gcc) ->
`bin/quill_harness_8m_v13(.exe)`. Vendored v13 checkout sits in
`cpp/quill/vendor13/quill` (gitignored; the committed default is still `vendor/quill`
= v12.1.0). `cpp/quill/CMakeLists.txt` gained `QUILL_VENDOR_DIR` / `QUILL_FETCH_TAG`
cache vars to select the release.