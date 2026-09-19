#!/usr/bin/env bash
# Minimal, quoting-safe poll of the running cross-lang benchmark.
# Lives at /tmp/poll.sh inside WSL and is invoked as `bash /tmp/poll.sh`
# (no $-signs or braces in the wsl wrapper => safe from PowerShell).
cd /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench || exit 1

echo "=== time: $(date +%H:%M) ==="
echo "=== run.sh alive? (pgrep count) ==="
pgrep -fc run.sh
echo "=== newest 6 result files (HH:MM  name) ==="
ls -lt --time-style=+%H:%M results/*.json 2>/dev/null | awk '{print $6, $7}' | head -6
echo "=== policy results present? ==="
ls -1 results/ticklog_policy_*.json 2>/dev/null | wc -l
echo "=== total results JSONs ==="
ls -1 results/*.json | wc -l
echo "=== last 5 lines of the shared progress log ==="
tail -5 /tmp/run_full.log 2>/dev/null || echo "(no /tmp/run_full.log yet)"
