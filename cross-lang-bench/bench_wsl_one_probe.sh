#!/usr/bin/env bash
cd /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench
./bin/ticklog_policy_fastwm_harness \
    --ns-per-tick 0.313097 \
    --output results/wsl_probe_fastwm.json \
    --candidate ticklog_policy_fastwm --threads 1,2,4,8,16 \
    >/tmp/wsl_probe.log 2>&1
echo "exit=$?"
python3 - <<'PY'
import json,os
p="/mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench/results/wsl_probe_fastwm.json"
print("exists?", os.path.exists(p))
if os.path.exists(p):
    d=json.load(open(p))
    print("os=",d.get("os"),"cand=",d.get("candidate"),"nspt=",d.get("ns_per_tick"))
    print("nres=",len(d.get("results",[])),"threads=",sorted(set(r.get("threads") for r in d.get("results",[]))))
PY
