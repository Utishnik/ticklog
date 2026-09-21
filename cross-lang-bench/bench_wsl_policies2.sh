#!/usr/bin/env bash
# Run the 5 remaining ticklog back-pressure policy ELF harnesses under
# WSL (os=linux). Writes wsl_-prefixed files so the os=windows results
# already in results/ are preserved.
set -uo pipefail
cd /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench || exit 1
mkdir -p results
NS=0.313097   # linux/WSL calibration (matches existing os=linux jsons)

run() {
    local out="$1" bin="$2" cand="$3"
    local f="results/${out}.json"
    echo "=== $out ($bin) ==="
    if [[ ! -x "$bin" ]]; then echo "  SKIP: $bin not executable"; return; fi
    "$bin" --ns-per-tick "$NS" --output "$f" \
        --candidate "$cand" --threads 1,2,4,8,16 || { echo "  FAILED exit=$?"; return; }
    echo "  exit=0 -> $f"
}

run "wsl_ticklog_policy_fastwm"    "bin/ticklog_policy_fastwm_harness"     "ticklog_policy_fastwm"
run "wsl_ticklog_policy_watermark" "bin/ticklog_policy_watermark_harness"  "ticklog_policy_watermark"
run "wsl_ticklog_policy_fast"      "bin/ticklog_policy_fast_harness"       "ticklog_policy_fast"
run "wsl_ticklog_policy_quill"     "bin/ticklog_policy_quill_harness"      "ticklog_policy_quill"
run "wsl_ticklog_policy_nanolog"   "bin/ticklog_policy_nanolog_harness"    "ticklog_policy_nanolog"

echo "ALL DONE"
