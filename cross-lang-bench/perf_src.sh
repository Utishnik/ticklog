#!/bin/bash
# Show hottest source lines (>=MINPCT% local) for one symbol.
# Usage: src.sh <perf.data> <symbol-substring> [minpct]
set -u
DATA="$1"
SYM="$2"
MINPCT="${3:-0.5}"
perf annotate -i "$DATA" --stdio "$SYM" 2>/dev/null | awk -v m="$MINPCT" '
  /^ +[0-9]/ {
    pct = $1 + 0
    if (pct >= m) {
      # keep source-line field if present (after last "|" or ".c" col)
      print
    }
  }'
