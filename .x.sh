#!/bin/bash
cd /mnt/c/Users/Admin/Desktop/ticklog
f=/tmp/backpressure_block_miri.log
echo "=== log head (first 12) ==="
sed -n '1,12p' $f
echo "=== integer-to-pointer line ===  (source drain.rs:930) ==="
grep -nB3 -A10 'integer-to-pointer' $f | head -30
echo "=== abort/UB markers ==="
grep -nE 'Undefined Behavior|error: Undefined|Aborting|aborting execution|integer-to-pointer cast is' $f | head
echo "=== progress markers: how far did tests get? ==="
grep -nE '^test .* \.\.\.|running [0-9]* test|test result' $f | tail -12