#!/bin/bash
# Single focused Miri probe of the Block-policy ring-overflow test.
# Budget 240s. RC=124 => still running at 240s (hang/slow).
cd /mnt/c/Users/Admin/Desktop/ticklog
log=/tmp/pb_final.log
timeout 240 cargo +nightly miri test --all-features --test backpressure_block \
  -- block_policy_never_loses_a_record --exact >"$log" 2>&1
rc=$?
echo "=== RC=$rc (0=pass,101=miri UB-abort,124=still running 240s=>hang/slow, 1=compile-err) ==="
echo "=== tail 10 ==="; tail -10 "$log"
echo "=== UB / abort markers ==="
grep -nE "Undefined Behavior|integer-to-pointer|error:|abort" "$log" | head -8
echo "=== drain progress: consecutive seq decoded? (how far did drain get) ==="
grep -nE "decoded seq|seq=" "$log" | tail -5
printf 'done probe\n'
