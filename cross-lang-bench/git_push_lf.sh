#!/usr/bin/env bash
# One-shot: safe.directory + strip 3 git-blockers + stage + commit + push (LF).
set -uo pipefail
d=/mnt/c/Users/Admin/Desktop/ticklog
b="$d/cross-lang-bench"

echo "=== 0) allow this dir for git (dubious ownership fix) ==="
git config --global --add safe.directory "$d" 2>/dev/null && echo "  safe.directory OK" || echo "  (config skipped)"

echo "=== 1) broken python shim ==="
if [[ -L "$b/python" ]]; then rm "$b/python" && echo "  removed symlink cross-lang-bench/python"; else echo "  (no symlink)"; fi

echo "=== 2) nested .git in quill vendor ==="
qg="$b/cpp/quill/vendor/quill/.git"
if [[ -d "$qg" ]]; then rm -rf "$qg" && echo "  removed vendor/.git"; else echo "  (no nested .git)"; fi

echo "=== 3) stray root compressedLog (bench artifact, 33B) ==="
if [[ -e "$d/compressedLog" ]]; then rm -f "$d/compressedLog" && echo "  removed root compressedLog"; else echo "  (none)"; fi

echo "=== 4) stage all (excluding our LF/scratch churn is fine) ==="
cd "$d" || exit 1
git add -A 2>&1 | grep -v 'LF will be replaced' | tail -3
echo "  add_done; staged=$(git diff --cached --name-only | wc -l)"

echo "=== 5) commit ==="
git commit -q -m "cross-lang-bench: WSL+Windows matrix run6-11 (ticklog policies, quill, nanolog, pinned), partial ticklog backends" && echo "  committed" || echo "  (nothing to commit?)"

echo "=== 6) push ==="
git push -q origin main 2>&1 && echo "  pushed OK" || echo "  PUSH FAILED exit=$?"
echo "=== 7) head ==="
git log --oneline -2