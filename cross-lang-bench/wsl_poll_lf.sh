#!/usr/bin/env bash
cd /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench || exit 1
echo "=== latest timestamps ==="
date +%Y-%m-%dT%H:%M:%S
echo "=== run.sh alive? (pid count) ==="
pgrep -fc run.sh
echo "=== newest 10 result files (name + mtime) ==="
ls -lt --time-style=+%H:%M results/*.json | head -10
echo "=== ticklog_policy_*.json present? ==="
ls -1 results/ticklog_policy_*.json 2>/dev/null | wc -l
echo "=== policy harness binaries in bin/ ==="
ls -1 bin/ticklog_policy_*_harness* 2>/dev/null
echo "=== tail of run log (last 8 lines) ==="
tail -8 /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench/run.log 2>/dev/null || echo "(no run.log)"
echo "=== WSL-side run log ==="
ls -lt --time-style=+%H:%M /tmp/run_full.log /tmp/run.log 2>/dev/null
echo "=== run.log tail ==="
tail -5 /tmp/run_full.log /tmp/run.log 2>/dev/null
