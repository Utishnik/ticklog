import json, io, re, importlib.util

def load_new(path):
    with open(path) as f:
        d = json.load(f)
    lat = {}
    thr = {}
    two_core = None
    for r in d["results"]:
        wl = r["workload"]
        n = r["threads"]
        lat[(wl, n)] = [r[m] for m in ("p50","p95","p99","p999","max")]
        if n == 1:
            thr.setdefault(n, {})[wl] = r["throughput"]
            if len(d["results"]) == 3:
                two_core = (lat[(wl,1)], r["throughput"])
        # also collect throughput by threads col
        thr.setdefault(n, {})[wl] = r["throughput"]
    return lat, thr, two_core

md = io.open('/mnt/c/Users/Admin/Desktop/ticklog/BENCHMARKS.md','r',encoding='utf-8').read()
idx = md.find('# Run 4')
sec = md[idx:]
print('Run4 slice len:', len(sec))
# check header regex
m = re.search(r'^### single_int: 1 thread\(s\)$', sec, re.M)
print('lat header match:', bool(m))
full = re.search(r'^### single_int: 1 thread\(s\)\n\| Candidate.*?\n\| ticklog.*$', sec, re.M)
print('full block:', full.group(0)[:120] if full else None)
