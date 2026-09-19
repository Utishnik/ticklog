#!/usr/bin/env bash
# Just push. Everything is already staged+committed; only push remains.
set -uo pipefail
d=/mnt/c/Users/Admin/Desktop/ticklog
cd "$d" || { echo "no cd"; exit 1; }
echo "=== head before push ==="
git log --oneline -1
echo "=== pushing (origin main) ==="
git push origin main 2>&1 | tail -4
echo "push_exit=${PIPESTATUS[0]}"
echo "=== head after ==="
git log --oneline -1
git status -sb | head -2