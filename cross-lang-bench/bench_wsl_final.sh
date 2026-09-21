#!/usr/bin/env bash
# WSL/Linux policy harness full bench (5 strategies).
# Preserves existing os=windows results by writing wsl_ prefixed files.
set -uo pipefail
cd /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench
mkdir -p results

echo "=== calibrating ns_per_tick in WSL ==="
NS="$(./calibrate)"
echo "  ns_per_tick = $NS"

run_one() {
    local name="$1" bin="$2" cand="$3" out="results/wsl_${name}.json"
    echo "=== $name ($bin) ==="
    if [[ ! -x "$bin" ]]; then echo "  SKIP: $bin not found"; return; fi
    "$bin" --ns-per-tick "$NS" --output "$out" \
        --candidate "$cand" --threads 1,2,4,8,16
    echo "  exit=$? -> $out"
}

run_one "ticklog_policy_fastwm"    "bin/ticklog_policy_fastwm_harness"    "ticklog_policy_fastwm"
run_one "ticklog_policy_watermark" "bin/ticklog_policy_watermark_harness" "ticklog_policy_watermark"
run_one "ticklog_policy_fast"      "bin/ticklog_policy_fast_harness"      "ticklog_policy_fast"
run_one "ticklog_policy_quill"     "bin/ticklog_policy_quill_harness"     "ticklog_policy_quill"
run_one "ticklog_policy_nanolog"   "bin/ticklog_policy_nanolog_harness"   "ticklog_policy_nanolog"

echo ""
echo "ALL DONE"
