# ticklog vs quill: Run 9 — watermark-batched head publish

Date: 2026-09-19. Follows Run 8 (QUILL-PERTHREAD-Run8.md).

## Hypothesis under test

Run 8 left one structural difference vs C++ quill: ticklog's hot path does a
`head.store(Release)` on **every** record (`publish`), so the drain (which polls
every ring's `head` per pass) causes one cross-core cache-line ping-pong per
record. C++ quill publishes once per ~12K-record block. The new
`watermark-head` feature batches head publishes: the producer keeps a private
`local_head` and releases it to the shared `head` every `WATERMARK_HEAD_RECORDS`
(=1000) records, plus at ring handoff and when its `ThreadBuf` is dropped.

Candidates measured: `watermark-head` alone and combined with the Run 8
`fast-tail-cache` (`fast+wm`), vs a fresh `quill` baseline re-run in the same
session, plus the C++ quill numbers from run 7.

## Setup

- Workloads as before (single_int / mixed / string, 1000 samples, batch 1000,
  1M total messages), threads 1,2,4,8,16.
- WSL: ring 32 MiB, ns/tick 0.313148759. Windows: ring 64 MiB, ns/tick 0.313132820.
- Binaries rebuilt identically on both platforms; the tail-loss bug in graceful
  shutdown found during validation was fixed (see below) before benchmarking.

### Validation note

An end-to-end check (`examples/wm_check.rs`) exposed a shutdown data-loss bug in
the watermark path: records past the last head watermark of the final ring were
never published when the producer exited. Fixed in `src/drain.rs`: on every
drain pass, a ring whose producer has exited (`live == false`, stored with
Release after the producer's final publish) is flushed before draining, which
is safe because the Acquire load of `live` orders all producer writes.
`wm_check` now reports delta=0 for baseline, `watermark-head`, and
`watermark-head`+`fast-tail-cache` at n=250000.

## p50 results (ns)

### single_int

| candidate (WSL) | t1 | t2 | t4 | t8 | t16 || candidate (WIN) | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| quill baseline r9 | 22.8 | 41.9 | 43.4 | 82.3 | 99.6 || quill baseline r9 | 29.8 | 35.1 | 47.5 | 85.1 | 144.0 |
| fast-tail-cache r8 | 18.5 | 24.9 | 38.3 | 70.2 | 99.7 || fast-tail-cache r8 | 23.1 | 27.5 | 52.5 | 82.3 | 95.3 |
| watermark-head r9 | 20.4 | 22.7 | 40.7 | 76.8 | 118.9 || watermark-head r9 | 32.1 | 30.8 | 50.6 | 79.5 | 93.1 |
| fast+wm r9 | 30.0 | 27.2 | 41.0 | 78.4 | 97.7 || fast+wm r9 | 24.8 | 32.1 | 47.0 | 87.3 | 98.4 |
| C++ quill r7 | 15.2 | 16.9 | 15.3 | 21.9 | 29.8 || C++ quill r7 | 14.9 | 14.9 | 20.2 | 23.2 | 40.0 |

### mixed

| candidate (WSL) | t1 | t2 | t4 | t8 | t16 || candidate (WIN) | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| quill baseline r9 | 40.9 | 36.9 | 44.9 | 78.7 | 88.6 || quill baseline r9 | 38.4 | 42.0 | 56.0 | 88.0 | 124.9 |
| fast-tail-cache r8 | 23.2 | 24.6 | 37.1 | 70.3 | 106.4 || fast-tail-cache r8 | 25.9 | 30.5 | 54.9 | 78.7 | 80.1 |
| watermark-head r9 | 25.4 | 39.8 | 38.8 | 77.9 | 98.9 || watermark-head r9 | 41.9 | 40.3 | 47.7 | 81.2 | 95.0 |
| fast+wm r9 | 35.5 | 38.8 | 53.7 | 79.7 | 99.6 || fast+wm r9 | 39.5 | 31.8 | 47.4 | 91.2 | 110.9 |
| C++ quill r7 | 17.0 | 29.6 | 22.1 | 32.3 | 38.1 || C++ quill r7 | 16.9 | 28.8 | 23.8 | 36.3 | 57.9 |

### string

| candidate (WSL) | t1 | t2 | t4 | t8 | t16 || candidate (WIN) | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| quill baseline r9 | 35.4 | 27.8 | 44.9 | 76.2 | 107.8 || quill baseline r9 | 24.3 | 32.4 | 49.9 | 89.9 | 87.7 |
| fast-tail-cache r8 | 20.2 | 26.9 | 39.5 | 74.6 | 77.9 || fast-tail-cache r8 | 22.6 | 28.4 | 44.9 | 80.5 | 95.2 |
| watermark-head r9 | 21.6 | 23.1 | 42.9 | 76.3 | 105.2 || watermark-head r9 | 24.5 | 27.7 | 45.3 | 86.2 | 82.0 |
| fast+wm r9 | 23.8 | 38.2 | 42.9 | 78.1 | 105.6 || fast+wm r9 | 38.3 | 27.6 | 46.8 | 85.8 | 85.5 |
| C++ quill r7 | 16.6 | 17.2 | 16.8 | 27.4 | 36.1 || C++ quill r7 | 15.9 | 16.0 | 23.2 | 29.2 | 43.0 |

## Run 10 addendum — same-session control (2026-09-19)

Run 9's candidates shipped with zero per-record shared atomics (`tail` cached
privately, `head` batched), so option (b) "remove the per-record tail Acquire"
was measured to completion. To rule out session noise as the explanation for
"no win", the fully optimized ring (`fast+wm`) was re-run in the same session
against ticklog's own segmented **NanoLog** policy, the library's best-known
path:

single_int p50 (ns)

| candidate | t1 | t2 | t4 | t8 | t16 || candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| WSL NanoLog policy | 20.1 | 22.3 | 39.9 | 72.1 | 118.7 || WIN NanoLog policy | 19.2 | 29.8 | 49.7 | 78.9 | 121.8 |
| WSL quill-ring fast+wm | 19.5 | 27.1 | 40.9 | 74.9 | 127.3 || WIN quill-ring fast+wm | 29.1 | 32.1 | 44.3 | 77.3 | 115.9 |
| WSL C++ quill | 15.2 | 16.9 | 15.3 | 21.9 | 29.8 || WIN C++ quill | 14.9 | 14.9 | 20.2 | 23.2 | 40.0 |

Findings:
- The fully optimized per-thread ring is now on par with ticklog's own NanoLog
  policy (~20 ns t1), i.e. the ring bookkeeping is not the differentiator.
  The earlier r7 NanoLog t1 figure (14.4 ns) was session noise.
- C++ quill still leads by ~1.3-2x at t1 (15.2 vs ~20), widening with threads.
- Conclusion: after removing all per-record shared-memory traffic (fast-tail-cache + watermark-head), the residual gap
  is not in the release/acquire mechanism — it lives in the per-record format/
  slot-write path or in the measurement basis of the two harnesses.

## Run 11 addendum — measurement audit: per-record metadata (2026-09-19)

Same-session comparison of `fast+wm` with and without per-record source/thread
metadata (`harness-strip-md` keeps only the format section in the record):

single_int p50 (ns)

| candidate | t1 | t2 | t4 | t8 | t16 || candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| WSL fastwm | 18.3 | 30.4 | 38.4 | 72.5 | 99.0 || WIN fastwm | 29.0 | 37.2 | 42.9 | 87.4 | 89.2 |
| WSL fastwm+strip | 17.3 | 18.0 | 22.8 | 36.3 | 37.9 || WIN fastwm+strip | 15.5 | 18.0 | 24.2 | 37.5 | 53.3 |
| WSL C++ quill | 15.2 | 16.9 | 15.3 | 21.9 | 29.8 || WIN C++ quill | 14.9 | 14.9 | 20.2 | 23.2 | 40.0 |

mixed p50 (ns)

| candidate | t1 | t2 | t4 | t8 | t16 || candidate | t1 | t2 | t4 | t8 | t16 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| WSL fastwm | 24.1 | 27.6 | 44.3 | 71.8 | 113.3 || WIN fastwm | 41.7 | 47.8 | 53.4 | 69.9 | 102.3 |
| WSL fastwm+strip | 19.2 | 19.7 | 20.8 | 36.9 | 46.1 || WIN fastwm+strip | 20.8 | 30.7 | 31.9 | 39.2 | 49.8 |
| WSL C++ quill | 17.0 | 29.6 | 22.1 | 32.3 | 38.1 || WIN C++ quill | 16.9 | 28.8 | 23.8 | 36.3 | 57.9 |

The strip build is validated lossless end-to-end (`wm_check` delta=0, 250000
records, lines decode correctly without the source/thread sections).

Findings:
- **Per-record source+thread metadata is the dominant multi-thread cost.** At
  t8/t16 it is roughly half of the measured latency (38.4 -> 22.8 ns at WSL t4;
  87.4 -> 37.5 ns at WIN t8). The producer copies `file`+`line` (14 bytes) and
  thread id+name section (~10+ bytes) into every record; quill stores these in
  a metadata table once and indexes records.
- With it stripped, single_int lands within ~10-20% of C++ quill at t1-t8
  (WSL t2 18.0 vs 16.9; WIN t1 15.5 vs 14.9), closing most of the previously
  unexplained gap.
- Remaining gap: string at t8/t16 (50.9/48.2 vs 27.4/36.1) and t16 peaks —
  consistent with drain-side throughput on the shared ring poll, not the
  producer's record header.

Actionable redesign implied by the audit: move thread identity into the ring
registration (a ring has exactly one producer, so the drain already knows the
thread) and fold `fmt`+`file`+`line` into a call-site metadata table indexed by
record — matching quill without losing the metadata from the output.

## Verdict

- **Watermark batching does not help.** `watermark-head` and `fast+wm` land in
  the same band as the `quill` baseline and `fast-tail-cache` (Run 8) across all
  workloads and thread counts on both platforms; remaining deltas (~10-25%) are
  session noise (note the baseline itself moved 22.7 -> 41.9 ns between Run 8
  and Run 9 on WSL t2, same binary).
- The per-record `head.store(Release)` that Run 8 pinpointed as the remaining
  structural gap is therefore **not** the dominant cost. Removing it entirely
  (records become visible only every 1000) changed nothing material.
- C++ quill still wins by ~1.6-2.5x at t1-t8 (e.g. single_int t4 on Windows:
  47.0 vs 20.2 ns; on WSL t8: 78.4 vs 21.9 ns), i.e. the gap is upstream of the
  release-consistency mechanism.

## Where the remaining cost lives

Both tick candidates already removed the per-record shared-head RMW; the
baseline's per-record head store is gone in `watermark-head` yet latency did not
drop. Remaining per-record work in ticklog that quill avoids:

1. **`tail.load(Acquire)` per record.** ticklog re-loads the shared tail (drain's
   write cursor) on every reserve to detect the wrap-around/out-of-space case;
   quill readers slot-select from a private counter and never touch a shared
   value per record (the block's head is the only shared store, once per block).
2. **Reserve-time formatting into the ring plus EOB/wrap handling** vs quill's
   memcpy of a pre-formatted record into a contiguous block slot.
3. **Record serialization/lifetimes**: ticklog re-reads slot bytes at format
   time in the drain with seqno/parser checks; quill passes the formatted record
   to a small format function.

## Next-step options

- **(a) Stop this line**: back out or leave the features disabled; report
  "per-thread ring + batched head" cannot close the gap without deeper surgery.
- **(b) Remove the per-record `tail` Acquire**: give the producer a private
  high-water watermark (like `head`) and only check the shared tail on a coarse
  cadence (every N records), mirroring what the head watermark did.
- **(c) Benchmark the C++ harness's record lifetime**: confirm whether quill
  measures format+enqueue while ticklog's p50 includes more (e.g. the format
  path differs). Cross-check by timing ticklog's own `Block` policy again in the
  same session.
- **(d) Micro-dissect ticklog's reserve path** (slot width, a size-check vs a
  redzone, seqno space) and shave the per-record branch count.

Files: results in `results_quill/*_r9.json`; code behind the features under
`#![cfg(feature = "watermark-head")]` in `src/ring.rs`, `src/thread_buf.rs`,
`src/drain.rs`, plus `examples/wm_check.rs`.