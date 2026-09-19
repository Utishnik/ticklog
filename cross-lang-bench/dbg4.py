import json, re, io, sys, importlib.util

dir = "/mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench"
sys.path.insert(0, dir)
C = importlib.import_module("compare_block_vs_drop")

with io.open(dir + "/BENCHMARKS.md", "r", encoding="utf-8-sig") as f:
    md = f.read()

old_lat, old_thr, old_two = C.parse_run4(md, "ticklog")
print("old_lat keys:", len(old_lat), "sample:", list(old_lat.items())[:1])
print("old_thr keys:", sorted(old_thr)[:1], "sample:", list(old_thr.items())[:1])
print("old_two:", old_two)

lat, thr, two = C.load_new("/home/utishnik/bench/ticklog-cross-lang-bench/results_1/ticklog.json")
print("new_lat keys:", len(lat), "sample:", list(lat.items())[:1])
print("new_thr keys:", sorted(thr)[:1])
print("new_two present:", two is not None and bool(two))
