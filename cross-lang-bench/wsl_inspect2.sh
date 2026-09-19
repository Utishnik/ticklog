#!/bin/bash
set -euo pipefail
cd /home/utishnik/bench/tkfast
for F in none watermark; do
  FEATURES=""
  if [ "$F" = "watermark" ]; then FEATURES="--features watermark-head"; fi
  cargo run --release --example wm_check $FEATURES -- /tmp/wm_$F.txt 250000 2>&1 | tail -1
  tail -2 /tmp/wm_$F.txt
  echo ---
  python3 - <<PY
import re
nums = [int(x) for x in re.findall(r"record (\d+)", open("/tmp/wm_$F.txt").read())]
seen = set(nums)
missing = [i for i in range(min(nums)+1, max(nums)+1) if i not in seen]
print("$F: count", len(nums), "min", min(nums), "max", max(nums), "missing", len(missing), missing[:10])
PY
done