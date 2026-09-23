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

run_ticklog "$BIN/ticklog_custom_drop" ticklog-custom-drop wsl_ticklog_custom_drop_r14.json
run_ticklog "$BIN/ticklog_rtrb_drop"   ticklog-rtrb-drop   wsl_ticklog_rtrb_drop_r14.json   --chunk-size 64
run_ticklog "$BIN/ticklog_quill"       ticklog-quill       wsl_ticklog_quill_r14.json
run_ticklog "$BIN/ticklog_nanolog"     ticklog-nanolog     wsl_ticklog_nanolog_r14.json

# C++ quill harness rebuilt with QUILL_BENCH_QUEUE_CAPACITY=8388608 (8 MiB)
if [ -x "$BIN/quill_harness_8m" ]; then
  echo "=== run quill-cpp 8MiB $(date +%H:%M:%S)"
  "$BIN/quill_harness_8m" --ns-per-tick "$NSPT" --output "$OUT/wsl_quill_cpp_r14.json" \
    --samples "$SAMPLES" 2>"$OUT/wsl_quill_cpp_r14.log"
  echo "  -> wsl_quill_cpp_r14.json"
fi

# C++ nanolog: no ring-size parameter (per-thread staging buffers, libNanoLog
# fixed by NanoLog::preallocate()); run as-is for reference.
NANO_EXE=""
if [ -x "$SCRIPT_DIR/cpp/nanolog/build/nanolog_harness" ]; then
  NANO_EXE="$SCRIPT_DIR/cpp/nanolog/build/nanolog_harness"
elif [ -x "$BIN/nanolog_harness" ]; then
  NANO_EXE="$BIN/nanolog_harness"
fi
if [ -n "$NANO_EXE" ]; then
  echo "=== run nanolog-cpp $(date +%H:%M:%S)"
  "$NANO_EXE" --ns-per-tick "$NSPT" --output "$OUT/wsl_nanolog_cpp_r14.json" \
    --samples "$SAMPLES" 2>"$OUT/wsl_nanolog_cpp_r14.log"
  echo "  -> wsl_nanolog_cpp_r14.json"
else
  echo "WARN: nanolog_harness not found, skip"
fi

echo "ALL_RUNS_DONE $(date +%H:%M:%S)"
ls -la "$OUT"/*r14* 2>/dev/null || true