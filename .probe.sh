#!/bin/bash
cd /mnt/c/Users/Admin/Desktop/ticklog
echo "=== run block_policy_never_loses_a_record under miri (budget 140s) ==="
timeout 140 cargo +nightly miri test --all-features --test backpressure_block block_policy_never_loses_a_record > /tmp/miri_ub.log 2>&1
echo "RC=$? (124=timeout/hang; 101=build err; 1=miri assert/UB abort)"
echo "=== integer-to-pointer / UB diagnostics ==="
grep -nE "integer-to-pointer|Undefined Behavior|aborting|Evaluation|warning:|error:" /tmp/miri_ub.log | head -20
echo "=== source anchor for the flagged site_ptr construction (record macro side) ==="
sed -n "903,936p" src/drain.rs