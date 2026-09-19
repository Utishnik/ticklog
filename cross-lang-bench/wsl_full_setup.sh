#!/usr/bin/env bash
set -euo pipefail
cd /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench
bash setup.sh 2>&1 | grep -E "^=== |^  done|error:|Error |FAILED" | tail -40