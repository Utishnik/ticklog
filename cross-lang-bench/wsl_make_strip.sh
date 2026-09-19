#!/bin/bash
set -euo pipefail
SRC=/mnt/c/Users/Admin/Desktop/ticklog
DST=/home/utishnik/bench/tkfast
for F in src/macros.rs src/lib.rs src/record.rs Cargo.toml; do
  cp "$SRC/$F" "$DST/$F"
done
cd "$DST/cross-lang-bench/rust/ticklog"
cargo build --release --features policy-fastwm-strip 2>&1 | tail -2
cp target/release/ticklog-cross-lang-harness /home/utishnik/tkbin/ticklog_fastwm_strip
ls -la /home/utishnik/tkbin/ticklog_fastwm_strip