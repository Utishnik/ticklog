#!/usr/bin/env bash
set -uo pipefail
cd /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench
mkdir -p results
NS=0.313097

run_one() {
    local name="$1" bin="$2" cand="$3" out="results/${name}.json"
    echo "=== $name ($bin) os=linux nspt=$NS ==="
    if [[ ! -x "$bin" ]]; then
        echo "  SKIP: $bin not present"
        return
    fi
    "$bin" --ns-per-tick "$NS" \
        --output "$out" \
        --candidate "$cand" --threads 1,2,4,8,16
    echo "  exit=$? -> $out"
}

run_one "wsl_ticklog_policy_fastwm"    "bin/ticklog_policy_fastwm_harness"    "ticklog_policy_fastwm"
run_one "wsl_ticklog_policy_watermark" "bin/ticklog_policy_watermark_harness" "ticklog_policy_watermark"
run_one "wsl_ticklog_policy_fast"      "bin/ticklog_policy_fast_harness"      "ticklog_policy_fast"
run_one "wsl_ticklog_policy_quill"     "bin/ticklog_policy_quill_harness"     "ticklog_policy_quill"
run_one "wsl_ticklog_policy_nanolog"   "bin/ticklog_policy_nanolog_harness"   "ticklog_policy_nanolog"

echo "ALL DONE"
