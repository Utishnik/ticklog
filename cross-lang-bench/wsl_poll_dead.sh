#!/usr/bin/env bash
cd /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench || exit 1
echo "=== processes matching ticklog/python/perf (pid, cmd) ==="
ps -eo pid,comm,args | grep -Ei 'run|ticklog|perf|python|nanolog' | grep -v grep
echo "=== run_launch logs in /tmp ==="
ls -l --time-style=+%H:%M /tmp/run*.log 2>/dev/null
echo "=== tail of /tmp/run_full.log ==="
tail -6 /tmp/run_full.log 2>/dev/null
echo "=== run.sh launcher file present in dir? ==="
ls -1 run.sh 2>/dev/null
