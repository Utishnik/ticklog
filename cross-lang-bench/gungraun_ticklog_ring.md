# gungraun version of the ticklog call-site benchmark

Methodology note: this is a **different** measurement from the classic
`cross-lang-bench/rust/ticklog` harness. Gungraun (0.19.4) runs each benchmark
exactly once under Valgrind/Callgrind on a single thread and reports tool-level
event counts (instructions, L1/LL/RAM hits, estimated cycles). It does **not**
measure wall-clock throughput or RDTSC latencies. It can only be executed on
Linux/WSL2 where Valgrind is available.

- Harness: `cross-lang-bench/rust/gungraun`
- Binary: `benches/ticklog_ring.rs`, `main!(library_benchmark_groups = [ticklog_ring_group])`
- Benchmark: one batch of `BATCH = 1000` `info!` calls against the custom ring
  with the `Drop` policy (mirrors `--candidate ticklog --policy-drop` in the
  classic harness; ring = `DEFAULT_RING_SIZE`, i.e. 1 MiB).
- Runtime setup (ring allocation + drain thread) runs in the gungraun `setup`
  hook — outside the instrumented region, so it is not counted.
- Workloads: `single_int` (`info!("x={}", v)`), `mixed` (`"{} {} {}"` with
  u64/f64/&str), `string` (`info!("{}", "hello world")`).

Run:

```bash
cd cross-lang-bench/rust/gungraun
GUNGRAUN_NOCAPTURE=true cargo bench
```

Requires `gungraun-runner` in `$PATH`:

```bash
cargo install --version 0.19.4 gungraun-runner
```

## Results — WSL2, 2026-09-21

Host: WSL2 (linux), `gungraun`/`gungraun-runner` 0.19.4, Valgrind 3.26.0.
Counts are per batch of 1000 calls. Raw output:
`cross-lang-bench/results/wsl_gungraun_ticklog_ring.log`.

| workload | Instructions | L1 Hits | LL Hits | RAM Hits | Total r/w | Est. Cycles |
|----------|-------------:|--------:|--------:|---------:|----------:|------------:|
| `single_int` | 49 029 | 74 040 | 2 | 1 006 | 75 048 | 109 260 |
| `mixed`      | 120 031 | 170 037 | 2 | 1 011 | 171 050 | 205 432 |
| `string`     | 102 029 | 142 036 | 2 | 1 010 | 143 048 | 177 396 |

Both `seed_0` and `seed_1` variants per workload report identical counts
(deterministic; `No change` vs the previous run).

### Per-call breakdown (batch / 1000)

| workload | Instructions/call | Est. cycles/call |
|----------|------------------:|-----------------:|
| `single_int` | 49 | 109 |
| `mixed`      | 120 | 205 |
| `string`     | 102 | 177 |

### Interpretation

- Formatting dominates: `single_int` is ~2.5x cheaper in instructions than
  `mixed` and ~2.1x cheaper than `string`, matching the wall-clock ordering in
  the classic harness.
- LL hits (2) and RAM hits (~1 006–1 011) are essentially constant across
  workloads: the per-record touch of the 1 MiB ring dominates the miss profile;
  the ~1000 RAM hits ≈ 1 MiB / 128 B cache line, i.e. cold faults on the ring.
- The counts include a (cheap, deterministic) drain thread; they are invariant
  between seeds, confirming the measured path has no seed-dependent branch.
- These numbers are not directly comparable to the classic throughput/latency
  tables — they quantify instructions/cycles under Callgrind, a complementary
  "one-shot" perspective on the call site.

## Flamegraph run (cache misses)

The `flamegraph` feature makes the same harness emit Callgrind flamegraphs for
the **L1 data cache-miss** event kinds only (`D1mr` = D1 read misses, `D1mw` =
D1 write misses), instead of the default instruction-count perspective. Regular
flamegraphs only; no differential.

```bash
cd cross-lang-bench/rust/gungraun
GUNGRAUN_NOCAPTURE=true cargo bench --features flamegraph
```

Generated SVGs land in `rust/gungraun/target/gungraun/ticklog-gungraun-harness/
ticklog_ring/ticklog_ring_group/<bench>.seed_<n>/callgrind.<bench>.seed_<n>.
total.<D1mr|D1mw>.flamegraph.svg`. Representative files are committed under
`cross-lang-bench/results/flamegraph/`.

### Totals per batch of 1000 calls (seed_0)

| workload | D1mw (write misses) | D1mr (read misses) |
|----------|--------------------:|-------------------:|
| `single_int` | 999 | 2 |
| `mixed` | 999 | 4 |
| `string` | 999 | 4 |

LL-miss event kinds were deliberately excluded: with only ~2 LL misses per batch
the flamegraph generator has "No stack counts found" (no frames to draw).

### Interpretation

- Write misses overwhelmingly dominate: **999 D1mw per batch ≈ 1 per log call**,
  all attributed (inclusive) to the benchmark body. This is the same signature
  seen in the RAM-hit column of the summary (~1006–1011 per batch) — the
  per-record write of a new 128 B ring slot misses L1 each time the ring wraps
  onto a fresh cache line.
- Read misses are noise (`D1mr`=2–4) — the hot path reads format strings and
  ring metadata that stay cached.
- Because LTO/fat codegen inlines ticklog internals into the benchmark body,
  the SVG stack is collapsed to `bench_* -> wrapper -> runner -> main`; the
  miss cost is concretely attached to the body frame rather than spread across
  named `__private` helpers.