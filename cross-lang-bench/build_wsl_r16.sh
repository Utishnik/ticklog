#!/bin/bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/rust/ticklog"
EXE=target/release/ticklog-cross-lang-harness
BIN=/mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench/bin
mkdir -p "$BIN"

build_one() {
  local name="$1"; shift
  echo "=== build $name features=[$*] $(date +%H:%M:%S)"
  cargo build --release --features "$*"
  cp "$EXE" "$BIN/${name}"
  echo "  -> $BIN/$name"
}

build_one ticklog_custom_drop policy-drop
build_one ticklog_rtrb_drop backend-rtrb,policy-drop
build_one ticklog_quill policy-quill
build_one ticklog_nanolog policy-nanolog

echo "ALL_BUILDS_DONE $(date +%H:%M:%S)"