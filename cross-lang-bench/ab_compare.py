import json, sys
a = json.load(open(sys.argv[1]))
b = json.load(open(sys.argv[2]))
ra = {(r["workload"], r["threads"]): r for r in a["results"]}
rb = {(r["workload"], r["threads"]): r for r in b["results"]}
hdr = "%-12s %2s %11s %9s %6s %11s %11s %6s" % (
    "wl", "t", "p50 A", "p50 B", "d%", "thr A", "thr B", "d%")
print(hdr)
for k in sorted(ra):
    x, y = ra[k], rb[k]
    dp = 100 * (y["p50"] - x["p50"]) / x["p50"]
    dt = 100 * (y["throughput"] - x["throughput"]) / x["throughput"]
    print("%-12s %2d %11.2f %9.2f %+6.1f %11d %11d %+6.1f" % (
        k[0], k[1], x["p50"], y["p50"], dp,
        x["throughput"], y["throughput"], dt))
