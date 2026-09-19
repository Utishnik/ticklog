#!/bin/bash
set -euo pipefail
SRC=/mnt/c/Users/Admin/Desktop/ticklog
DST=/home/utishnik/bench/tkfast
rm -rf "$DST"
mkdir -p "$DST"
cd "$SRC"
tar cf - \
  --exclude='./.git' \
  --exclude='./target' \
  --exclude='./cross-lang-bench/rust/ticklog/target' \
  --exclude='./results_quill' \
  . | (cd "$DST" && tar xf -)
cd "$DST/cross-lang-bench/rust/ticklog"
cargo build --release --features policy-fast 2>&1 | tail -15
cp target/release/ticklog-cross-lang-harness /home/utishnik/tkbin/ticklog_fast
ls -la /home/utishnik/tkbin/ticklog_fast