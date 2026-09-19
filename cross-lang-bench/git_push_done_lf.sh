#!/usr/bin/env bash
# One-shot: clean git blockers, stage all, commit, push. LF.
# Run via: wsl -d Ubuntu -u root -e bash -c "bash /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench/git_push_done_lf.sh"
set -uo pipefail
d=/mnt/c/Users/Admin/Desktop/ticklog
cd "$d" || { echo "no cd"; exit 1; }

echo "=== t1) fix dubious ownership ==="
git config --global --add safe.directory "$d" 2>/dev/null || true

echo "=== t2) remove broken cross-lang-bench/python (WSL shim symlink) ==="
if [[ -L cross-lang-bench/python ]]; then
  rm cross-lang-bench/python && echo "  removed symlink"
else
  echo "  (not a symlink)"
fi

echo "=== t3) drop stray compressedLog in repo root ==="
if [[ -e compressedLog ]]; then
  rm -f compressedLog && echo "  removed root compressedLog"
else
  echo "  (none)"
fi

echo "=== t4) drop nested .git in cpp/quill/vendor/quill ==="
g=cross-lang-bench/cpp/quill/vendor/quill/.git
if [[ -d "$g" ]]; then
  rm -rf "$g" && echo "  removed vendor .git"
else
  echo "  (no nested vendor .git)"
fi

echo "=== t5) stage all ==="
git add -A 2>/dev/null
echo "  staged_files=$(git diff --cached --name-only | wc -l)"

echo "=== t6) commit ==="
git commit -q -m "cross-lang-bench: publish partial cross-lang matrix + quill/nanolog/Zerolog/Zap harnesses + policy-policy builds" \
  && echo "  committed" || echo "  (commit nothing/err=$?)"

echo "=== t7) push ==="
git push -q origin main && echo "  PUSHED_OK" || echo "  push_status=$?"

echo "=== t8) head after ==="
git log --oneline -1