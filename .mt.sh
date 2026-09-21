#!/bin/bash
cd /mnt/c/Users/Admin/Desktop/ticklog
echo "=== multithread.rs constants ==="
grep -nE "const [A-Z_]+: usize|RECORDS_PER_THREAD|THREADS" tests/multithread.rs | head
echo "=== run multithread binary: budget 150s ==="
timeout 150 cargo +nightly miri test --all-features --test multithread > /tmp/mt.log 2>&1
echo "RC=$? (124=150s not enough => volume too big for a single-file run; needs cfg(miri) shrink)"
tail -8 /tmp/mt.log