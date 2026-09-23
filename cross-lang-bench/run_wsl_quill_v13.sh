#!/bin/bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT="$SCRIPT_DIR/results_quill"
BIN="$SCRIPT_DIR/bin"
mkdir -p "$OUT"

JSON="$OUT/wsl_quill_cpp_v13_r16.json"
LOG="$OUT/wsl_quill_cpp_v13_r16.log"
rm -f "$JSON" "$LOG" "$LOG.err"
echo "=== run quill-cpp v13 8MiB $(date +%H:%M:%S)"
cd "$OUT"
set +e
"$BIN/quill_harness_8m_v13" --ns-per-tick 0.313148759 --output "$JSON" --samples 1000 > "$LOG" 2> "$LOG.err"
code=$?
set -e
if [ $code -ne 0 ]; then
  echo "harness failed code=$code"
  cat "$LOG" "$LOG.err"
  exit $code
fi
[ -f "$JSON" ] || { echo "no json"; cat "$LOG" "$LOG.err"; exit 1; }
grep -i 'blocking' "$LOG" || true
echo "  -> wsl_quill_cpp_v13_r16.json"
echo "WSL_V13_DONE $(date +%H:%M:%S)"