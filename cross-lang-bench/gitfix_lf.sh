#!/usr/bin/env bash
# Fix the three objects that block `git add`, then stage/commit/push.
# Invoke: wsl -d Ubuntu -u root -e bash -c "bash /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench/gitfix_lf.sh"
set -uo pipefail
d=/mnt/c/Users/Admin/Desktop/ticklog
b="$d/cross-lang-bench"

echo "=== 0) current status ==="
cd "$d" || exit 1
git status --short | head -20
echo "  (untracked count: $(git status --porcelain | wc -l))"

echo "=== 1) remove broken WSL python shim (symlink -> fails 'Function not implemented') ==="
if [[ -L "$b/python" ]]; then
    rm -f "$b/python" && echo "  removed symlink cross-lang-bench/python"
else
    echo "  (python not a symlink)"
fi

echo "=== 2) drop nested .git inside quill vendor (embedded repo blocks add) ==="
vg="$b/cpp/quill/vendor/quill/.git"
if [[ -d "$vg" ]]; then
    rm -rf "$vg" && echo "  removed vendor/quill/.git"
else
    echo "  (no nested vendor .git)"
fi

echo "=== 3) remove stray root 'compressedLog' file ==="
if [[ -e "$d/compressedLog" ]]; then
    rm -f "$d/compressedLog" && echo "  removed root compressedLog"
else
    echo "  (no root compressedLog)"
fi

echo "=== 4) stage everything ==="
git add -A 2>&1 | grep -v "^warning:" | tail -5
echo "  add exit=${PIPESTATUS[0]}"
echo "  staged files: $(git diff --cached --name-only | wc -l)"

echo "=== 5) commit ==="
git commit -q -m "bench: snapshot current partial results + policy harnesses" \
  && echo "  committed" || echo "  commit failed but continuing"

echo "=== 6) push ==="
git push origin main 2>&1 | tail -4
echo "  push exit=${PIPESTATUS[0]}"

echo "=== 7) final ==="
git log --oneline -2
