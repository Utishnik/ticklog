#!/bin/bash
set -e
cd /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench
NSPT=0.313148759
RING=8388608
THREADS=1,2,4,8,16
SAMP=1000
OUT=/mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench/results_quill

for c in custom_drop rtrb_drop quill nanolog; do
  exe=bin/ticklog_${c}_prof
  extra=""
  if [ "$c" = rtrb_drop ]; then extra="--chunk-size 64"; fi
  echo "=== $c $(date +%H:%M:%S)"
  $exe --ns-per-tick $NSPT --output $OUT/wsl_ticklog_${c}_prof.json \
    --candidate ticklog-$c --threads $THREADS --samples $SAMP \
    --ring-capacity $RING $extra \
    > $OUT/wsl_ticklog_${c}_prof.out 2> $OUT/wsl_ticklog_${c}_prof.log
done
echo ALL_PROF_RUNS_DONE