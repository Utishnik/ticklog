#!/usr/bin/env bash
# WSL (os=linux) full policy bench for the 5 remaining back-pressure
# strategies. Harness CLI confirmed from --help:
#   harness --ns-per-tick <float> --output <path> [--candidate <n>]
#           [--threads <n,...>] [--ns-per-tick]
set -uo pipefail
cd /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench || exit 1
NS=0.313097
OUTDIR=results
mkdir -p "$OUTDIR"

run_wsl() {
    local name="$1" bin="$2"
    echo "=== $name ==="
    if [[ ! -x "$bin" ]]; then echo "  SKIP: $bin not executable"; return; fi
    "$bin" --ns-per-tick "$NS" \
        --output "$OUTDIR/wsl_${name}.json" \
        --candidate "$name" --threads 1,2,4,8,16
    echo "  exit=$? -> $OUTDIR/wsl_${name}.json"
}

run_wsl "ticklog_policy_fastwm"    "bin/ticklog_policy_fastwm_harness"
run_wsl "ticklog_policy_watermark" "bin/ticklog_policy_watermark_harness"
run_wsl "ticklog_policy_fast"      "bin/ticklog_policy_fast_harness"
run_wsl "ticklog_policy_quill"     "bin/ticklog_policy_quill_harness"
run_wsl "ticklog_policy_nanolog"   "bin/ticklog_policy_nanolog_harness"
echo "ALL DONE"
