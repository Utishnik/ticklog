#!/bin/bash
set -e
cd /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench/rust/ticklog
BIN=/mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench/bin
EXE=target/release/ticklog-cross-lang-harness

build_one() {
  local name="$1"; shift
  echo "=== build $name [$*]"
  cargo build --release --features "$*" 2>&1 | tail -2
  cp "$EXE" "$BIN/${name}"
}

build_one ticklog_custom_drop_prof policy-drop,hotpath-profiler
build_one ticklog_rtrb_drop_prof backend-rtrb,policy-drop,hotpath-profiler
build_one ticklog_quill_prof policy-quill,hotpath-profiler
build_one ticklog_nanolog_prof policy-nanolog,hotpath-profiler
echo ALL_PROF_BUILDS_DONE