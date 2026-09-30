#!/bin/bash
# Extract hottest source lines from a perf annotate dump for a line range.
# Usage: hotlines.sh <annot.txt> <from> <to> [minpct]
set -u
F="$1"; FROM="$2"; TO="$3"; MIN="${4:-1.0}"
sed -n "${FROM},${TO}p" "$F" | awk -v m="$MIN" '
  /^ +[0-9]/ {
    pct = $1 + 0
    if (pct >= m) print
  }'
