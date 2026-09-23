#!/bin/bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT="$SCRIPT_DIR/results_quill"
BIN="$SCRIPT_DIR/bin"
NSPT=0.313148759
RING=67108864   # 64 MiB (user asked 60 MB; capacity must be power-of-two)
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

run_ticklog "$BIN/ticklog_custom_drop" ticklog-custom-drop wsl_ticklog_custom_drop_r13.json
run_ticklog "$BIN/ticklog_rtrb_drop"   ticklog-rtrb-drop   wsl_ticklog_rtrb_drop_r13.json   --chunk-size 64
run_ticklog "$BIN/ticklog_quill"       ticklog-quill       wsl_ticklog_quill_r13.json
run_ticklog "$BIN/ticklog_nanolog"     ticklog-nanolog     wsl_ticklog_nanolog_r13.json

echo "ALL_RUNS_DONE $(date +%H:%M:%S)"
ls -la "$OUT"/*r13* 2>/dev/null || true
