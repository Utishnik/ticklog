#!/usr/bin/env bash
cd /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench/results
for n in ticklog_policy_fastwm ticklog_policy_watermark ticklog_policy_fast ticklog_policy_quill ticklog_policy_nanolog; do
  f="$n.json"
  if [[ -f "$f" ]]; then
    python3 - "$f" <<'PY'
import json,sys
f=sys.argv[1]
d=json.load(open(f))
osv=d.get("os"); nt=d.get("ns_per_tick", d.get("nspt")); r=d.get("results",[])
thr=sorted(set(x.get("threads") for x in r))
print(f, "os=",osv, "nspt=",nt, "n=",len(r), "thr=",thr)
PY
  else
    echo "$f MISSING"
  fi
done