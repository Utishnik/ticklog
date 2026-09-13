# ticklog

A fast, minimal logging library for latency-critical Rust applications, such as high-frequency trading, where the cost of a log call on the hot path must stay in the low tens of nanoseconds.

How it works: Log calls run exclusively on the calling thread's hot path: check the level, encode a compact binary record into that thread's private lock-free buffer, and return. A background drain thread does the rest: decoding, formatting, timestamping, and writing each record, keeping all of that cost off the calling thread.

## Features

- **Nanosecond hot path:** 5–7 ns per call, over 30x faster than general-purpose loggers (see [Benchmarks](#benchmarks)), with no per-call allocation, formatting, or I/O on the calling thread, so the cost stays small and predictable on latency-critical paths.
- **Console/file sinks + Fanout:** stdout or stderr (colored by level) and buffered files, with fan-out from one record to several sinks and per-sink level filtering.
- **Ecosystem support:** file rotation, compression, and retention come from existing crates like [logroller](https://crates.io/crates/logroller) and [tracing-appender](https://crates.io/crates/tracing-appender) (see [Examples](examples/sinks/)); anything that implements `io::Write` plugs in as a sink.
- **Zero dependencies:** no runtime dependencies; a minimal, self-contained crate.

## Installation

```toml
[dependencies]
ticklog = "0.1"
```

Requires Rust 1.85 or newer (edition 2024).

## Quick start

```rust
use ticklog::{info, FileSink};

let _guard = ticklog::configure! {
    sink: FileSink::new("app.log").unwrap(),
}
.unwrap();

info!("listening on {}", 8080);
```

`ticklog::configure!` returns a `Guard`. Keep it alive for as long as you want to log: when it is dropped it flushes the sink, stops the background thread, and disables logging, so every log call afterwards is a silent no-op.

## Benchmarks

Per-call latency (p50). Lower is better.

### Benchmark names

| Name    | Log call                                           |
| ------- | -------------------------------------------------- |
| one_u64 | `info!("x={}", 42u64)`                             |
| one_str | `info!("{}", "hello world")`                       |
| mixed   | `info!("{} {} {}", 42u64, 3.14159, "hello world")` |

### Rust Ecosystem

**Mac M4** (Apple M4, 4.4 GHz, macOS 15):

| Logger      |    one_u64 |    one_str |      mixed |
| ----------- | ---------: | ---------: | ---------: |
| **ticklog** | **5.2 ns** | **5.9 ns** | **7.0 ns** |
| env_logger  |     231 ns |     232 ns |     307 ns |
| slog        |     274 ns |     269 ns |     454 ns |
| tracing     |     386 ns |     425 ns |     458 ns |

**Granite Rapids** (Intel Xeon 6982P-C, 3.9 GHz, Ubuntu 24.04):

| Logger      |    one_u64 |    one_str |      mixed |
| ----------- | ---------: | ---------: | ---------: |
| **ticklog** | **8.6 ns** | **8.4 ns** | **9.9 ns** |
| env_logger  |     370 ns |     371 ns |     491 ns |
| slog        |     499 ns |     453 ns |     686 ns |
| tracing     |     837 ns |     854 ns |     937 ns |

Run with `cargo bench --bench latency_vs_baseline` (ticklog) and `cargo bench --bench latency_vs_<logger>` (others).

### Sustained Throughput

End-to-end wall clock (encoding + drain + formatting) divided by records, median of 5 runs: 16 threads × 300k records each into a null sink, `Backpressure::Block`, per-thread ring. Lower is better.

| Ring capacity | custom ring | ringbuffer crate |
| ------------- | ----------: | ---------------: |
| 64 KiB        |      581 ns |           775 ns |
| 256 KiB       |      559 ns |           753 ns |
| 1 MiB         |      585 ns |           736 ns |
| 4 MiB         |      537 ns |           612 ns |
| 16 MiB        |      479 ns |           552 ns |

If the ring is large enough to hold a whole batch (64 MiB), the sustained cost drops to ~30 ns with no backpressure stalls. Reproduce with `examples/capacity_probe.rs` or `scripts/capacity_sweep.sh`.

### Cross-Language Comparison

Shared VM (16 vCPU, no core isolation), identical protocol (BATCH=1000, RDTSC), p50. Single thread:

| Logger      | Language | single_int |      string |      mixed |
| ----------- | -------- | ---------: | ----------: | ---------: |
| nanolog     | C++      |     16.5 ns |      16.6 ns |     16.1 ns |
| **ticklog** | **Rust** | **21.6 ns** | **21.3 ns** | **22.2 ns** |
| quill       | C++      |     20.4 ns |         --  |        --  |
| zerolog     | Go       |     94.4 ns |      92.8 ns |    207.2 ns |
| zap         | Go       |    590.4 ns |     583.7 ns |    765.9 ns |

Throughput, 4 threads (in millions of records/s, single_int):

| Logger      | rec/s |
| ----------- | ----: |
| **ticklog** | **103.2M** |
| nanolog     | 57.0M      |
| quill       | 18.3M      |
| zerolog     | 24.1M      |
| zap         | 4.2M       |

ticklog scales to ~200M rec/s at 16 threads (6x vs 1 thread); nanolog,
zerolog, and quill stop scaling past 4-8 threads (single backend consumer).
Writing to a real file instead of a null sink (`--sink-file`) does not move
call-site latency and keeps the 16-thread throughput at ~200M (vs nanolog's
23.9M), because file I/O stays on the background drain thread.
Full tables including p95/p99/max, 1-16 thread latency, thread scaling, the
ring-capacity sweep, and the null-sink vs file-sink comparison are in
[cross-lang-bench/BENCHMARKS.md](cross-lang-bench/BENCHMARKS.md).

Reproduce: `cd cross-lang-bench && ./setup.sh && ./run.sh --no-perf`. See [cross-lang-bench](cross-lang-bench/) for details.

### Experimental SPSC backends

The default single-producer ring is the crate-local slot ring. Three
feature-gated experimental alternates (`backend-ringbuffer`, `backend-ringbuf`,
`backend-triple-buffer`, mutually exclusive, one crate each) let the
cross-language harness compare byte-FIFO and single-slot handoff designs on the
same single-thread pipeline. In the SPSC run (producer on core 0, drain on
core 1) the `ringbuf` crate backend ties p50 (~15.6 ns) while running ~13-20%
slower and 1.5-2x worse on p95 tail; the `triple_buffer` backend is a
ping-pong, not a buffer — `reserve` rendezvouses with the drain every record,
capping it at ~1M rec/s here. Full numbers, including jitter:
[cross-lang-bench/BENCHMARKS.md](cross-lang-bench/BENCHMARKS.md).

## Configuration

`ticklog::configure!` accepts these keys, each optional:

| Key             | Purpose                                                                         | Default                                       |
| --------------- | ------------------------------------------------------------------------------- | --------------------------------------------- |
| sink            | Where output goes.                                                              | ConsoleSink on stderr                         |
| max_level       | Records above this level are dropped on the calling thread before any encoding. | `Level::Info`                                 |
| backpressure    | What a logging thread does when its buffer is full.                             | `Backpressure::Drop`                          |
| format          | Log-line pattern with `{field}` placeholders.                                   | `{timestamp} {level} {file}:{line} {message}` |
| timezone_offset | Seconds east of UTC, applied to timestamp formatting only.                      | 0 (UTC)                                       |
| drain_affinity  | Pin the background thread to a set of logical CPUs.                             | none                                          |

Example with every key:

```rust
use ticklog::{ConsoleSink, Level, Backpressure};

let _guard = ticklog::configure! {
    sink: ConsoleSink::stderr(),
    max_level: Level::Trace,
    backpressure: Backpressure::Drop,
    format: "{timestamp} [{level:>5}] {file}:{line:04} {message}",
    timezone_offset: 3600,
    drain_affinity: Some(vec![0]),
}
.unwrap();
```

`Backpressure::Drop` discards the record and returns immediately, never blocking the caller. `Backpressure::Block` spins until space frees up: it never drops records but burns CPU while the buffer stays full.

## Sinks

A `LogSink` is the final destination for formatted lines. The crate ships three:

```rust
use ticklog::{ConsoleSink, ColorMode, FileSink};

// stdout or stderr, colored by level (auto-detected, or forced on/off)
let console = ConsoleSink::stderr();
let plain = ConsoleSink::stdout().with_color(ColorMode::Never);

// a buffered single file, appended to or truncated on open
let appended = FileSink::new("app.log").unwrap();
let fresh = FileSink::truncate("app.log").unwrap();
```

Compose and filter with `FanOut` (dispatch one record to several sinks) and `with_max_level` (limit a sink to a level and below):

```rust
use ticklog::{ConsoleSink, FanOut, Level, LogSinkExt};

let sink = FanOut::new()
    .add(ConsoleSink::stderr().with_max_level(Level::Warn))
    .add(ConsoleSink::stdout().with_max_level(Level::Info));
```

### Custom sinks

For a destination that is not `io::Write`, such as a channel or a metrics counter, implement `LogSink` directly.

```rust
use std::io;
use std::net::UdpSocket;
use ticklog::{Level, LogSink};

struct UdpSink {
    socket: UdpSocket,
}

impl LogSink for UdpSink {
    fn accept(&mut self, line: &[u8], _level: Level) -> io::Result<()> {
        self.socket.send(line).map(|_| ())
    }
}
```

## Threads

Any thread may log, and each allocates its own buffer on first use. To move that one-time allocation off a latency-sensitive path, call `warm_up()` on the thread before its first log call. `pin_thread` pins the calling thread to a set of logical CPUs.

```rust
// A latency-sensitive worker: pin it to a core and pre-allocate its buffer
// up front, so its first log call is as cheap as the rest.
let worker = std::thread::spawn(|| {
    ticklog::pin_thread(&[3]);
    ticklog::warm_up().unwrap();

    // hot loop...
});
```

## License

MIT OR Apache-2.0
