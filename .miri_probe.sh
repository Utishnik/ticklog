#!/bin/bash
cd /mnt/c/Users/Admin/Desktop/ticklog
log=/tmp/full_suite.log
timeout 480 cargo +nightly miri test --all-features > "$log" 2>&1
rc=$?
echo "RC=$rc"
echo "=== sequence of test binaries STARTED, in order (the LAST printed = candidate hang) ==="
grep -nE "Running unittests src|Running tests/|Running Docs-tests|running [0-9]+ tests" "$log" | tail -30
echo "=== the very last 6 lines: where execution was when budget ran out ==="
tail -6 "$log"
echo "=== if the last binary printed 'running N tests' => which test names did it reach? ==="
awk '/Running tests\//{bin=$0} /^test [^ ]+ \.\.\./{print bin" -> "$0}' "$log" | tail -6