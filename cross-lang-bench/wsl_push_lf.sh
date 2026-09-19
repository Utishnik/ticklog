#!/usr/bin/env bash
set -uo pipefail
d=/mnt/c/Users/Admin/Desktop/ticklog
cd "$d" || exit 1
echo "=== status ==="
git log --oneline -1
git status --short | head -5
echo "=== push ==="
git push -q origin main && echo "PUSHED_OK" || { echo "PUSH_FAILED"; git status -sb; }
git log --oneline -1
