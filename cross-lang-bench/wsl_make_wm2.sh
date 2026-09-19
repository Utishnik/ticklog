#!/bin/bash
set -euo pipefail
SRC=/mnt/c/Users/Admin/Desktop/ticklog
DST=/home/utishnik/bench/tkfast
cp "$SRC/src/drain.rs" "$DST/src/drain.rs"
cd "$DST/cross-lang-bench/rust/ticklog"
cargo build --release --features policy-watermark 2>&1 | tail -4
cp target/release/ticklog-cross-lang-harness /home/utishnik/tkbin/ticklog_wm
cargo build --release --features policy-fastwm 2>&1 | tail -4
cp target/release/ticklog-cross-lang-harness /home/utishnik/tkbin/ticklog_fastwm
ls -la /home/utishnik/tkbin/ticklog_wm /home/utishnik/tkbin/ticklog_fastwm