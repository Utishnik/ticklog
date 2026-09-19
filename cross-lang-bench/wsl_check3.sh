#!/bin/bash
set -euo pipefail
cd /home/utishnik/bench/tkfast
echo "== baseline (no features) =="
cargo run --release --example wm_check -- /tmp/wm_baseline.txt 250000 2>&1 | tail -2
echo "== watermark-head =="
cargo run --release --example wm_check --features watermark-head -- /tmp/wm_wt.txt 250000 2>&1 | tail -2
echo "== fast-tail-cache+watermark =="
cargo run --release --example wm_check --features "watermark-head fast-tail-cache" -- /tmp/wm_fw.txt 250000 2>&1 | tail -2