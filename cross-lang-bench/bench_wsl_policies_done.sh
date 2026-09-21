#!/usr/bin/env bash
# Final: run the 5 back-pressure policy harnesses (ticklog_policy_*)
# in WSL/linux with os=linux. Writes results/wsl_ticklog_policy_*.json
# then python3 audit -> results/wsl_policies_report.txt
set -uo pipefail
cd /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench
mkdir -p results
NS=0.313097
LOG=results/wsl_policies_run.log
: > "$LOG"
{
run_one () {
    local name="$1" bin="$2"
    echo "=== $name ($bin) ==="
    if [[ ! -x "$bin" ]]; then echo "  SKIP: $bin not executable"; return; fi
    "$bin" --ns-per-tick "$NS" \
        --output "results/wsl_${name}.json" \
        --candidate "$name" --threads 1,2,4,8,16
    echo "  exit=$? -> wsl_${name}.json"
}
run_one ticklog_policy_fastwm    bin/ticklog_policy_fastwm_harness
run_one ticklog_policy_watermark bin/ticklog_policy_watermark_harness
run_one ticklog_policy_fast      bin/ticklog_policy_fast_harness
run_one ticklog_policy_quill     bin/ticklog_policy_quill_harness
run_one ticklog_policy_nanolog   bin/ticklog_policy_nanolog_harness
} >> "$LOG" 2>&1

python3 - results ../ "$LOG" <<'PY'
import json, os, sys
base = os.path.realpath(os.path.dirname(sys.argv[2]))
os.chdir(base)
lines=[]
for name in ["fastwm","watermark","fast","quill","nanolog"]:
    fp=os.path.join("results", f"wsl_ticklog_policy_{name}.json")
    if not os.path.exists(fp):
        lines.append(f"{name}: MISSING")
        continue
    try: o=json.load(open(fp))
    except Exception as e: lines.append(f"{name}: JSONERR {e}"); continue
    lines.append(f"{name}: os={o.get('os')} cand={o.get('candidate')} "
                 f"nspt={o.get('ns_per_tick')} n={len(o.get('results',[]))}")
open(os.path.join("results","wsl_policies_report.txt"),"w").write("\n".join(lines)+"\nALLWSLPOLICIESOK\n")
PY
echo "ALLWSLPOLICIESDONE"
