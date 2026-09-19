#!/usr/bin/env bash
# Remove the three objects that break `git add`; then git add -A.
# LF-safe, no $ trouble when invoked via: bash <(tr -d '\r' < <file>)
set -u
d=/mnt/c/Users/Admin/Desktop/ticklog
b="$d/cross-lang-bench"

echo "=== remove broken python symlink ==="
if [ -L "$b/python" ]; then rm "$b/python" && echo "  removed broken symlink"; else echo "  (no symlink)"; fi

echo "=== drop nested .git from quill vendor ==="
q="$b/cpp/quill/vendor/quill/.git"
if [ -d "$q" ]; then rm -rf "$q" && echo "  removed vendor/.git"; else echo "  (no vendor/.git)"; fi

echo "=== remove stray root compressedLog ==="
if [ -f "$d/compressedLog" ]; then rm "$d/compressedLog" && echo "  removed"; else echo "  (none)"; fi

echo "=== git add -A ==="
cd "$d" && git add -A 2>&1 | tail -6
echo "add_exit=${PIPESTATUS[0]}"
echo "=== staged file count ==="
git diff --cached --name-only | wc -l