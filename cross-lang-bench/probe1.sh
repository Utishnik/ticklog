#!/usr/bin/env bash
set -uo pipefail
cd /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench 2>/dev/null || cd /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench

echo "=== HELP (authoritative) ==="
./bin/ticklog_policy_fastwm_harness 2>&1 | sed -n '2p'

echo ""
echo "=== ONE harness run (fastwm) -> /tmp/sc_t.json ==="
./bin/ticklog_policy_fastwm_harness \
    --ns-per-tick 0.313123 \
    --output /tmp/sc_t.json \
    --candidate ticklog_policy_fastwm --threads 1,2,4,8,16 2>&1 | tail -3
echo "  exit=$?"
echo "--- result head ---"
if [ -f /tmp/sc_t.json ]; then head -c 420 /tmp/sc_t.json; echo; else echo "NO FILE"; fi
