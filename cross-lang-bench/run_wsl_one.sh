#!/usr/bin/env bash
set -uo pipefail
cd /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench
mkdir -p results
NS=0.313097
echo "cwd=$(pwd)"
echo "ns_per_tick=$NS"
cmd=(bin/ticklog_policy_fastwm_harness
     --ns-per-tick "$NS"
     --output results/ticklog_policy_fastwm.json
     --candidate ticklog_policy_fastwm --threads 1,2,4,8,16)
echo "running: ${cmd[*]}"
"${cmd[@]}"
echo "exit=$?"
echo "--- octet head of json ---"
head -c 400 results/ticklog_policy_fastwm.json
echo
