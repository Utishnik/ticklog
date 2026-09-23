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
  if [ "$#" -eq 0 ]; then
    cargo build --release
  else
    cargo build --release --features "$*"
  fi
  cp "$EXE" "$BIN/${name}"
  echo "  -> $BIN/$name"
}

build_one ticklog_custom
build_one ticklog_rtrb backend-rtrb
build_one ticklog_ringbuf backend-ringbuf
build_one ticklog_ringbuffer backend-ringbuffer
build_one ticklog_triple backend-triple-buffer
build_one ticklog_quill policy-quill
build_one ticklog_nanolog policy-nanolog

echo "ALL_BUILDS_DONE $(date +%H:%M:%S)"
