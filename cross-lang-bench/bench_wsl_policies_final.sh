#!/usr/bin/env bash
# WSL/linux full-bench of the 5 remaining back-pressure policies.
# os=linux is recorded by the harness automatically (std::env::consts::OS).
# Output: results/wsl_<name>.json (wsl_ prefix keeps os=windows files intact).
# A python3 audit then writes results/wsl_policies_report.txt.
set -uo pipefail
cd /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench
mkdir -p results
NS=0.313097
LOG=results/wsl_policies_run.log
: > "$LOG"

run_one () {
    local name="$1" bin="$2" cand="$3"
    {
        echo "=== $name ($bin) ==="
        if [[ ! -x "$bin" ]]; then
            echo "  SKIP: $bin not found or not executable"
            return
        fi
        "$bin" --ns-per-tick "$NS" \
            --output "results/wsl_${name}.json" \
            --candidate "$cand" --threads 1,2,4,8,16
        echo "  exit=$? -> results/wsl_${name}.json"
    } >> "$LOG" 2>&1
}

run_one ticklog_policy_fastwm    bin/ticklog_policy_fastwm_harness    ticklog_policy_fastwm
run_one ticklog_policy_watermark bin/ticklog_policy_watermark_harness ticklog_policy_watermark
run_one ticklog_policy_fast      bin/ticklog_policy_fast_harness      ticklog_policy_fast
run_one ticklog_policy_quill     bin/ticklog_policy_quill_harness     ticklog_policy_quill
run_one ticklog_policy_nanolog   bin/ticklog_policy_nanolog_harness   ticklog_policy_nanolog

python3 - results dirname "$LOG" <<'PY'
import json, os, sys, glob
d = os.path.dirname(sys.argv[2])
if d:
    os.chdir(d)
lines = []
for name in ["fastwm","watermark","fast","quill","nanolog"]:
    fp = os.path.join("results", f"wsl_ticklog_policy_{name}.json")
    if not os.path.exists(fp):
        lines.append(f"{name}: MISSING {fp}")
        continue
    try:
        o = json.load(open(fp))
    except Exception as e:
        lines.append(f"{name}: JSONERROR {e}")
        continue
    lines.append(f"{name}: os={o.get('os')} candidate={o.get('candidate')} "
                 f"ns_per_tick={o.get('ns_per_tick')} nres={len(o.get('results',[]))}")
open("results/wsl_policies_report.txt","w").write("\n".join(lines)+"\nALLWSLFINALDONE\n")
print("audit written")
PY
echo "exit=$? ALLWSLFINALDONE"
