#!/usr/bin/env bash
set -e
cd /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench/cpp/quill
rm -rf build
echo '=== configure quill (vendored, offline) ==='
cmake -B build -DCMAKE_BUILD_TYPE=Release 2>&1 | tail -2
echo '=== build quill_harness ==='
cmake --build build -j6 2>&1 | grep -E "error:|Built target quill_harness|Linking" | tail -3
if [ -f build/quill_harness ]; then
    cp build/quill_harness ../../bin/quill_harness
    echo "  done -> bin/quill_harness"
else
    echo "  BUILD_FAILED: no build/quill_harness"
    exit 1
fi