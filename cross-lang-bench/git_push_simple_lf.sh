#!/usr/bin/env bash
# Simple push: stage all + commit + push. Name kept short to avoid quoting noise.
set -uo pipefail
d=/mnt/c/Users/Admin/Desktop/ticklog
cd "$d" || { echo "no cd"; exit 1; }
# Make git trust the dir (after toggling from root) to avoid dubious-owner failures.
git config --global --add safe.directory "$d" 2>/dev/null
echo "=== add ==="
git add -A 2>&1 | tail -1
echo "add_exit=${PIPESTATUS[0]}"
echo "staged=$(git diff --cached --name-only | wc -l)"
echo "=== commit ==="
git commit -q -m "cross-lang-bench: publish raw result matrices (ticklog, ringbuf, triple-buffer, quill, zerolog, zap, file)" && echo "committed" || echo "commit_status=$?"
echo "=== push ==="
git push -q origin main && echo "PUSHED_OK" || echo "push_status=$?"
git log --oneline -1