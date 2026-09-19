#!/usr/bin/env bash
set -e
cd /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench/cpp/quill
cmake --build build -j1 2>&1 | grep -E "main.cpp:[0-9]+:[0-9]+: error:|error: [A-Za-z].*:" | head -8
