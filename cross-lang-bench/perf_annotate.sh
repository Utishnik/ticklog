#!/bin/bash
# Show hot instructions (>=1% local) from perf annotate.
# Usage: annotate.sh <perf.data> [minpct]
set -u
DATA="${1:-/home/utishnik/tickperf/big.data}"
MINPCT="${2:-1.0}"
perf annotate -i "$DATA" --stdio 2>/dev/null | awk -v m="$MINPCT" '
  /^ +[0-9]/ {
    pct = $1 + 0
    if (pct >= m) print
  }'
