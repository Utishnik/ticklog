#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

# NanoLog's code-generating preprocessor calls `python`; on Ubuntu 24.04
# only `python3` exists. Provide a build-local shim (mirrors setup.sh).
mkdir -p .pyshim
ln -sf "$(command -v python3)" .pyshim/python
if [ -d /root/NanoLog ]; then
    NANOLOG_DIR=/root/NanoLog
else
    NANOLOG_DIR="$SCRIPT_DIR/cpp/NanoLog"
fi

echo "=== build NanoLog runtime ($NANOLOG_DIR) ==="
make -C "$NANOLOG_DIR/runtime" -j6 2>&1 | tail -3
ls -l "$NANOLOG_DIR/runtime/libNanoLog.a"

echo "=== build nanolog harness ==="
cd cpp/nanolog
rm -rf build
PATH="$SCRIPT_DIR/.pyshim:$PATH" cmake -B build -DCMAKE_BUILD_TYPE=Release \
    -DNANOLOG_RUNTIME_DIR="$NANOLOG_DIR/runtime" 2>&1 | tail -2
PATH="$SCRIPT_DIR/.pyshim:$PATH" cmake --build build -j6 2>&1 | tail -3
cp build/nanolog_harness "$SCRIPT_DIR/bin/nanolog_harness"
echo "  done -> bin/nanolog_harness"
