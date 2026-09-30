#!/bin/bash
# Dump instruction lines with pct >= threshold from an annotate dump, with line numbers.
set -u
F="$1"; FROM="${2:-1}"; TO="${3:-999999}"; MIN="${4:-0.5}"
sed -n "${FROM},${TO}p" "$F" | awk -v m="$MIN" -v off="$FROM" '
  /^ +[0-9]/ && $1+0 >= m { printf "%d: %s\n", NR+off-1, $0 }'
