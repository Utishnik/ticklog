#!/bin/bash
set -euo pipefail
NSPT=0.313148759
for B in quill; do
  /home/utishnik/tkbin/ticklog_$B --ns-per-tick $NSPT --threads 1 --ring-capacity 1048576 --samples 300 --candidate check-$B --sink-file /tmp/chk_$B.txt --output /tmp/chk_$B.json
  LINES=$(wc -l < /tmp/chk_$B.txt)
  echo "quill(base 1MiB): sink lines=$LINES (expected 900000)"
done
for B in wm; do
  /home/utishnik/tkbin/ticklog_$B --ns-per-tick $NSPT --threads 1 --ring-capacity 16777216 --samples 200 --candidate check-$B --sink-file /tmp/chk_$B.txt --output /tmp/chk_$B.json
  LINES=$(wc -l < /tmp/chk_$B.txt)
  echo "$B(16MiB): sink lines=$LINES (expected 600000)"
done