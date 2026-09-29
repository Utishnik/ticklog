#!/bin/bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/rust/ticklog"
EXE=target/release/ticklog-cross-lang-harness
BIN=/mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench/bin
mkdir -p "$BIN"

# Run 17: rtrb backend under the build-time Block backpressure policy.
# No hotpath/hotpath feature in this build (cargo tree verified): the
# #[hotpath::measure]/#[hotpath::main] macros expand to no-ops, so the bench
# runs with zero instrumentation overhead.
build_one() {
  local name="$1"; shift
  echo "=== build $name features=[$*] $(date +%H:%M:%S)"
  cargo build --release --features "$*"
  cp "$EXE" "$BIN/${name}"
  echo "  -> $BIN/$name"
}

build_one ticklog_rtrb_block backend-rtrb,policy-block

echo "ALL_BUILDS_DONE $(date +%H:%M:%S)"
