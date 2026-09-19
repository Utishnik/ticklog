#!/bin/bash
set -euo pipefail
NSPT=0.313148759
OUT=/mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench/results_quill
RING=33554432
THREADS=1,2,4,8,16
for W in nanolog fastwm; do
  BIN=/home/utishnik/tkbin/ticklog_$W
  "$BIN" --ns-per-tick $NSPT --threads $THREADS --ring-capacity $RING --samples 1000 --candidate ticklog-$W --output "$OUT/wsl_ticklog_${W}_r10.json"
done
ls -la "$OUT" | grep r10