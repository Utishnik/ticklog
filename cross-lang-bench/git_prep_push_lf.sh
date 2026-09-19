#!/usr/bin/env bash
set -u
d=/mnt/c/Users/Admin/Desktop/ticklog
echo "=== remove broken python symlink in cross-lang-bench ==="
rm -f "$d/cross-lang-bench/python" 2>/dev/null && echo "  python removed" || echo "  python: nothing/skipped"
echo "=== drop nested .git from quill vendor ==="
if [ -d "$d/cross-lang-bench/cpp/quill/vendor/quill/.git" ]; then
  rm -rf "$d/cross-lang-bench/cpp/quill/vendor/quill/.git"
  echo "  vendor/.git removed"
else
  echo "  no vendor/.git"
fi
echo "=== remove stray compressedLog in repo root ==="
rm -f "$d/compressedLog" 2>/dev/null && echo "  root compressedLog removed" || echo "  none"
cd "$d" || exit 1
echo "=== git add -A ==="
git add -A 2>/dev/null
echo "add_status=$?"
echo "=== staged files count ==="
git diff --cached --name-only | wc -l
