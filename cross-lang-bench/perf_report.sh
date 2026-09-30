#!/bin/bash
# Run 18 follow-up: profile ticklog_rtrb_block with perf (WSL).
# Usage: report.sh <perf.data> [minpct]
set -u
DATA="${1:-/tmp/perf_8m.data}"
MINPCT="${2:-0.5}"

echo "=== self-symbols by thread (>=${MINPCT}%) ==="
perf script -i "$DATA" 2>/dev/null | awk '
  /^[^ #]/ && / ticklog/ { comm=$1; next }
  /^ticklog/ { comm=$1; next }
  /^ +[0-9a-f]+ / { sym=$NF; gsub(/[()]/,"",sym); print comm"\t"sym }
' | sort | uniq -c | sort -rn | awk -v m="$MINPCT" 'BEGIN{total=0} {c[NR]=$1; k[NR]=$2; s[NR]=$3; total+=$1} {i=0} END{for(j=1;j<=NR;j++){pct=100*c[j]/total; if(pct>=m) printf "%6.2f%% %7d  %s  %s\n", pct, c[j], k[j], s[j]}}'

echo
echo "=== thread sample totals ==="
perf script -i "$DATA" 2>/dev/null | awk '
  /^[^ #]/ && / ticklog/ { comm=$1; next }
  /^ticklog/ { comm=$1; next }
  /^ +[0-9a-f]+ / { n[comm]++ }
  END { for (c in n) printf "%8d  %s\n", n[c], c }
' | sort -rn
