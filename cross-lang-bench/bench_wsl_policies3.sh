#!/usr/bin/env bash
# WSL (os=linux) runs of the 5 remaining ticklog back-pressure policy
# strategies. Writes results/wsl_*.json so the native os=windows files
# (results/ticklog_policy_*.json) are preserved.
set -uo pipefail
cd /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench
mkdir -p results
NS=0.313097

run() {
    local name="$1" bin="$2" cand="$3"
    echo "=== $name ($bin) ==="
    if [ ! -x "$bin" ]; then echo "  SKIP: $bin not executable"; return; fi
    "$bin" --ns-per-tick "$NS" \
        --output "results/$name.json" \
        --candidate "$cand" --threads 1,2,4,8,16
    local rc=$?
    echo "  exit=$rc -> results/$name.json"
}

run "wsl_ticklog_policy_fastwm"    "bin/ticklog_policy_fastwm_harness"    "ticklog_policy_fastwm"
run "wsl_ticklog_policy_watermark" "bin/ticklog_policy_watermark_harness" "ticklog_policy_watermark"
run "wsl_ticklog_policy_fast"      "bin/ticklog_policy_fast_harness"      "ticklog_policy_fast"
run "wsl_ticklog_policy_quill"     "bin/ticklog_policy_quill_harness"     "ticklog_policy_quill"
run "wsl_ticklog_policy_nanolog"   "bin/ticklog_policy_nanolog_harness"   "ticklog_policy_nanolog"

echo "ALL_POLICIES_DONE"
