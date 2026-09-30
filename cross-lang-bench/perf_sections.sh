#!/bin/bash
# Summarize symbol sections in a perf annotate dump: symbol header lines + sample totals.
# Usage: sections.sh <annot.txt>
set -u
F="$1"
# Section headers in perf annotate stdio output look like: "Absolute symbol: 0x..." or
# "<symbol>:" lines after a blank line; safest is to list lines that contain "samples" %
# Actually perf annotate --stdio prints blocks like:
#   HITS(%)   LINE                    CODE
# We instead find lines with pattern "  [<addr>] <symbol>:" or the "Files" list at top.
grep -n -E '^[0-9a-f]+ <[^>]+>:' "$F" | head -80
echo '--- fs tls variants ---'
grep -o '%fs:-0x[0-9a-f]*' "$F" | sort | uniq -c | sort -rn
