#!/bin/bash
# Aggregate perf report self% by thread and symbol.
# Usage: report2.sh <perf.data> [minpct]
set -u
DATA="${1:-/home/utishnik/tickperf/big.data}"
MINPCT="${2:-0.3}"

perf report -i "$DATA" --stdio --percent-limit 0 -g none 2>/dev/null | awk -v m="$MINPCT" '
NF >= 5 && $1 ~ /^[0-9]+\.[0-9]+%$/ {
  pct = $2 + 0
  comm = $3
  # symbol starts at field 5
  sym = ""
  for (i = 5; i <= NF; i++) sym = sym (i > 5 ? " " : "") $i
  if (pct >= m && (comm ~ /ticklog/)) printf "%6.2f%%  %-15s %s\n", pct, comm, sym
}' | sort -rn
