#!/usr/bin/env bash
# git-prep + commit + push (LF, из WSL root, для надёжного вывода).
set -uo pipefail

D=/mnt/c/Users/Admin/Desktop/ticklog
B="$D/cross-lang-bench"

cd "$D" || exit 1

echo "=== 1) unstage/cleanup traps ==="
# 1a. Broken python symlink (shim from setup.sh over windows dir)
if [[ -L "$B/python" ]] || [[ -e "$B/python" ]]; then
    rm -f "$B/python" 2>/dev/null
    echo "  removed broken python shim"
else
    echo "  (no python shim)"
fi

# 1b. nested .git inside quill vendor (embedded repo -> gitlink trap)
if [[ -e "$B/cpp/quill/vendor/quill/.git" ]]; then
    rm -rf "$B/cpp/quill/vendor/quill/.git"
    echo "  removed nested quill vendor .git"
fi

# 1c. stray compressedLog artifact
if [[ -e "$D/compressedLog" ]]; then
    rm -f "$D/compressedLog"
    echo "  removed stray compressedLog"
fi

echo "=== 2) git add -A ==="
git add -A 2>&1 | tail -5
echo "  add done (exit=${PIPESTATUS[0]})"

echo "=== 3) staged file count ==="
git diff --cached --name-only | wc -l

echo "=== 4) status short (count) ==="
git status --short | wc -l

echo "=== 5) commit ==="
git commit -q -m "cross-lang-bench: snapshot partial benchmark run + harness/policy fixes

- add 5 ticklog_policy_ harnesses (fastwm/watermark/fast/quill/nanolog)
- quill/nanolog CMake fixes (LF, offline vendor, NANOLOG_RUNTIME_DIR)
- run.sh: policy section for both Linux branches
- results: 7 partial JSON (ticklog, ringbuf, triple-buffer, file, quill, zerolog, zap)
- aux: WSL setup/poll scripts" && echo "  committed" || echo "  (nothing to commit or error)"

echo "=== 6) push ==="
git push -q origin main 2>&1 && echo "  pushed OK" || echo "  push failed (exit=$?)"
git rev-parse --short HEAD
