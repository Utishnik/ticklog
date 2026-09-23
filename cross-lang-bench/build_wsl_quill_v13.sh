#!/bin/bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$SCRIPT_DIR"
BIN="$ROOT/bin"
VENDOR13="$ROOT/cpp/quill/vendor13/quill"

mkdir -p "$BIN"

echo "=== configure quill v13 (8 MiB) $(date +%H:%M:%S)"
cmake -S "$ROOT/cpp/quill" -B "$ROOT/cpp/quill/build13-wsl" \
  -DCMAKE_BUILD_TYPE=Release \
  -DQUILL_BENCH_QUEUE_CAPACITY=8388608 \
  -DQUILL_VENDOR_DIR="$VENDOR13"

echo "=== build quill v13 $(date +%H:%M:%S)"
cmake --build "$ROOT/cpp/quill/build13-wsl" -j "$(nproc)"

cp "$ROOT/cpp/quill/build13-wsl/quill_harness" "$BIN/quill_harness_8m_v13"
echo "  -> $BIN/quill_harness_8m_v13"
echo "ALL_BUILDS_DONE $(date +%H:%M:%S)"