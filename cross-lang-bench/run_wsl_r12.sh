#!/bin/bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT="$SCRIPT_DIR/results_quill"
BIN="$SCRIPT_DIR/bin"
NSPT=0.313148759
RING=33554432   # 32 MiB
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

run_ticklog "$BIN/ticklog_custom"     ticklog-custom     wsl_ticklog_custom_r12.json
run_ticklog "$BIN/ticklog_rtrb"       ticklog-rtrb       wsl_ticklog_rtrb_r12.json       --chunk-size 512
run_ticklog "$BIN/ticklog_ringbuf"    ticklog-ringbuf    wsl_ticklog_ringbuf_r12.json
run_ticklog "$BIN/ticklog_ringbuffer" ticklog-ringbuffer wsl_ticklog_ringbuffer_r12.json
run_ticklog "$BIN/ticklog_triple"     ticklog-triple     wsl_ticklog_triple_r12.json
run_ticklog "$BIN/ticklog_quill"      ticklog-quill      wsl_ticklog_quill_r12.json
run_ticklog "$BIN/ticklog_nanolog"    ticklog-nanolog    wsl_ticklog_nanolog_r12.json

# C++ references: fixed thread/workload matrix; only ns-per-tick/output/samples
if [ -x "$BIN/quill_harness" ]; then
  echo "=== run quill-cpp $(date +%H:%M:%S)"
  "$BIN/quill_harness" --ns-per-tick "$NSPT" --output "$OUT/wsl_quill_cpp_r12.json" \
    --samples "$SAMPLES" 2>"$OUT/wsl_quill_cpp_r12.log"
  echo "  -> wsl_quill_cpp_r12.json"
fi

NANO_EXE=""
if [ -x "$SCRIPT_DIR/cpp/nanolog/build/nanolog_harness" ]; then
  NANO_EXE="$SCRIPT_DIR/cpp/nanolog/build/nanolog_harness"
elif [ -x "$BIN/nanolog_harness" ]; then
  NANO_EXE="$BIN/nanolog_harness"
fi
if [ -n "$NANO_EXE" ]; then
  echo "=== run nanolog-cpp $(date +%H:%M:%S)"
  "$NANO_EXE" --ns-per-tick "$NSPT" --output "$OUT/wsl_nanolog_cpp_r12.json" \
    --samples "$SAMPLES" 2>"$OUT/wsl_nanolog_cpp_r12.log"
  echo "  -> wsl_nanolog_cpp_r12.json"
else
  echo "WARN: nanolog_harness not found, skip"
fi

echo "ALL_RUNS_DONE $(date +%H:%M:%S)"
ls -la "$OUT"/*r12* 2>/dev/null || true
