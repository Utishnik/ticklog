#!/usr/bin/env bash
# Find & print the ticklog policy harness SOURCE arg-parsing + the panic line.
d=/mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench
echo "=== locate harness main.rs (policy) ==="
find "$d" -name 'main.rs' -path '*policy*' 2>/dev/null
find "$d" -name '*.rs' 2>/dev/null | grep -iv 'target\|\.git' | head -40
echo ""
echo "=== any file that defines --ns-per-tick / ns_per_tick / --candidate / --threads ==="
grep -rln -- '--ns-per-tick\|ns_per_tick\|ns-per-tick\|--candidate\|--threads\|--output' "$d" --include=*.rs 2>/dev/null | grep -v '@/cache' | head