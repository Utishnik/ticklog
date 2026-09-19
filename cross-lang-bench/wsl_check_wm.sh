#!/bin/bash
set -euo pipefail
NSPT=0.313148759
for B in wm fastwm; do
  /home/utishnik/tkbin/ticklog_$B --ns-per-tick $NSPT --threads 1 --ring-capacity 1048576 --samples 300 --candidate check-$B --sink-file /tmp/wm_$B.txt --output /tmp/wm_$B.json
  LINES=$(wc -l < /tmp/wm_$B.txt)
  echo "$B: sink lines=$LINES (expected 300000)"
done