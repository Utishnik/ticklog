#!/usr/bin/env bash
# Full WSL/Linux runs of the 5 remaining ticklog back-pressure policies.
# os=linux will be recorded by the harness. Output -> results/wsl_*.json
# so the os=windows policy results already on disk are never clobbered.
set -uo pipefail
cd /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench
mkdir -p results
NS=0.313097

run_one() {
    local name="$1" bin="$2" cand="$3"
    echo "=== $name ($bin) ==="
    if [[ ! -x "$bin" ]]; then
        echo "  SKIP: $bin not executable"
        return
    fi
    "$bin" --ns-per-tick "$NS" \
        --output "results/wsl_${name}.json" \
        --candidate "$cand" --threads 1,2,4,8,16
    echo "  exit=$? -> results/wsl_${name}.json"
}

run_one "ticklog_policy_fastwm"    "bin/ticklog_policy_fastwm_harness"    "ticklog_policy_fastwm"
run_one "ticklog_policy_watermark" "bin/ticklog_policy_watermark_harness" "ticklog_policy_watermark"
run_one "ticklog_policy_fast"      "bin/ticklog_policy_fast_harness"      "ticklog_policy_fast"
run_one "ticklog_policy_quill"     "bin/ticklog_policy_quill_harness"     "ticklog_policy_quill"
run_one "ticklog_policy_nanolog"   "bin/ticklog_policy_nanolog_harness"   "ticklog_policy_nanolog"
echo "ALL WSL POLICIES DONE"
