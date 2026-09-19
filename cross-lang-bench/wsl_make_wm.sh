#!/bin/bash
set -euo pipefail
SRC=/mnt/c/Users/Admin/Desktop/ticklog
DST=/home/utishnik/bench/tkfast
cp "$SRC/Cargo.toml" "$DST/Cargo.toml"
cp "$SRC/src/ring.rs" "$DST/src/ring.rs"
cp "$SRC/src/thread_buf.rs" "$DST/src/thread_buf.rs"
cp "$SRC/cross-lang-bench/rust/ticklog/Cargo.toml" "$DST/cross-lang-bench/rust/ticklog/Cargo.toml"
cp "$SRC/cross-lang-bench/rust/ticklog/src/main.rs" "$DST/cross-lang-bench/rust/ticklog/src/main.rs"
cd "$DST/cross-lang-bench/rust/ticklog"
cargo build --release --features policy-watermark 2>&1 | tail -8
cp target/release/ticklog-cross-lang-harness /home/utishnik/tkbin/ticklog_wm
cargo build --release --features policy-fastwm 2>&1 | tail -8
cp target/release/ticklog-cross-lang-harness /home/utishnik/tkbin/ticklog_fastwm
ls -la /home/utishnik/tkbin/ticklog_wm /home/utishnik/tkbin/ticklog_fastwm