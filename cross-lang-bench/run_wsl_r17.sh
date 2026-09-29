#!/bin/bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT="$SCRIPT_DIR/results_quill"
BIN="$SCRIPT_DIR/bin"
NSPT=0.313148759
RING=8388608    # 8 MiB
THREADS=1,2,4,8,16
SAMPLES=1000
mkdir -p "$OUT"

run_ticklog() {
  local exe="$1" name="$2" out="$3"
  shift 3
  echo "=== run $name $(date +%H:%M:%S)"
  "$exe" \
    --ns-per-tick "$NSPT" \
    --output "$OUT/$out" \
    --candidate "$name" \
    --threads "$THREADS" \
    --samples "$SAMPLES" \
    --ring-capacity "$RING" \
    "$@" \
    2>"$OUT/${out}.log"
  echo "  -> $out"
}

run_quill() {
  local exe="$1" out="$2"
  if [ -x "$exe" ]; then
    echo "=== run $(basename "$exe") 8MiB $(date +%H:%M:%S)"
    "$exe" --ns-per-tick "$NSPT" --output "$OUT/$out" \
      --samples "$SAMPLES" 2>"$OUT/${out}.log"
    echo "  -> $out"
  else
    echo "WARN: missing $exe, skip"
  fi
}

# Run 17: ticklog rtrb + Block backpressure vs C++ quill (v13.0.0 + v12.1.0).
run_ticklog "$BIN/ticklog_rtrb_block" ticklog-rtrb-block wsl_ticklog_rtrb_block_r17.json --chunk-size 64
run_quill "$BIN/quill_harness_8m_v13" wsl_quill_cpp_v13_r17.json
run_quill "$BIN/quill_harness_8m"     wsl_quill_cpp_r17.json

echo "ALL_RUNS_DONE $(date +%H:%M:%S)"
