#!/usr/bin/env bash
set -uo pipefail
cd /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench
mkdir -p results
NS=0.313097
declare -a NAMES=(ticklog_policy_fastwm ticklog_policy_watermark ticklog_policy_fast ticklog_policy_quill ticklog_policy_nanolog)
declare -a BINS=(bin/ticklog_policy_fastwm_harness bin/ticklog_policy_watermark_harness bin/ticklog_policy_fast_harness bin/ticklog_policy_quill_harness bin/ticklog_policy_nanolog_harness)
for i in "${!NAMES[@]}"; do
  n="${NAMES[$i]}"; b="${BINS[$i]}"
  echo "=== $n ==="
  if [[ ! -x "$b" ]]; then echo "  SKIP: not found"; continue; fi
  "$b" --ns-per-tick "$NS" --output "results/$n.json" \
       --candidate "$n" --threads 1,2,4,8,16 \
       --sink-file /dev/null 2>&1 | tail -1
  echo "  exit=$? -> results/$n.json"
done
echo "ALL DONE"