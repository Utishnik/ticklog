#!/bin/bash
python3 - <<'PY'
import re
for F in ["none", "watermark"]:
    p = "/home/utishnik/wm_%s.txt" % F
    nums = [int(x) for x in re.findall(r"record (\d+)", open(p).read())]
    seen = set(nums)
    missing = [i for i in range(min(nums)+1, max(nums)+1) if i not in seen]
    print(F, "count", len(nums), "min", min(nums), "max", max(nums), "missing", len(missing), missing[:30])
    # contiguous chunks of missing
    chunks = []
    for m in missing:
        if chunks and m == chunks[-1][-1] + 1:
            chunks[-1].append(m)
        else:
            chunks.append([m])
    print(F, "missing chunks", [(c[0], c[-1], len(c)) for c in chunks][:20])
PY