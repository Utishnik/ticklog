#!/bin/bash
set -euo pipefail
tail -3 /tmp/wm_wt.txt
echo ---
python3 - <<'PY'
import re
nums = [int(x) for x in re.findall(r"record (\d+)", open("/tmp/wm_wt.txt").read())]
print("count", len(nums), "min", min(nums), "max", max(nums))
seen = set(nums)
missing = [i for i in range(min(nums)+1, max(nums)+1) if i not in seen]
print("missing", len(missing), "first", missing[:20], "last", missing[-20:])
PY