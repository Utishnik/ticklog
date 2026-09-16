# Windows benchmark results (Run 3)

Raw JSON outputs from the **native Windows** benchmark run (BENCHMARKS.md "Run 3").

- `sweep/` — full 1/2/4/8/16-thread sweep, all candidates that build on Windows
  (ticklog, ticklog_file, ticklog_ringbuf, ticklog_triple-buffer, zerolog, zap, quill).
  quill built with clang 23.1.1 (LLVM), zerolog/zap with Go 1.27.0, ticklog with
  Rust 1.98.1 (MSVC, lto=fat, codegen-units=1).
- `two-core-pinned/` — ticklog variants pinned producer core 0 / drain core 1,
  inline loggers (zerolog/zap/quill) pinned to core 0 via process affinity.

Each file carries `"os": "windows"` internally; same for the `clock`/`ns_per_tick`
fields. ns_per_tick = 0.313097 was calibrated in WSL2 on the same physical CPU
(invariant TSC) because the Windows `calibrate` cannot use `clock_gettime`.

Host: AMD Ryzen 7 7735HS (8C/16T), Windows 11 Pro build 26200, 13.3 GB RAM.