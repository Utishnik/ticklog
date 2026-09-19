import json, glob, collections, os

DIR = r"/mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench/results_quill"

def load(f):
    with open(os.path.join(DIR, f), encoding="utf-8") as fh:
        return json.load(fh)

def p50(fname, workload, threads):
    d = load(fname)
    for r in d["results"]:
        if r["workload"] == workload and r["threads"] == threads:
            return r["p50"]
    raise KeyError((fname, workload, threads))

rows = [
    ("WSL", "wsl_ticklog_quill_r9.json",   "quill(baseline) r9"),
    ("WSL", "wsl_ticklog_fast_r8.json",    "fast-tail-cache r8"),
    ("WSL", "wsl_ticklog_wm_r9.json",      "watermark-head r9"),
    ("WSL", "wsl_ticklog_fastwm_r9.json",  "fast+wm r9"),
    ("WSL", "wsl_quill_cpp_r7.json",       "C++ quill r7"),
    ("WIN", "windows_ticklog_quill_r9.json", "quill(baseline) r9"),
    ("WIN", "windows_ticklog_fast_r8.json",  "fast-tail-cache r8"),
    ("WIN", "windows_ticklog_wm_r9.json",    "watermark-head r9"),
    ("WIN", "windows_ticklog_fastwm_r9.json", "fast+wm r9"),
    ("WIN", "windows_quill_cpp_r7.json",     "C++ quill r7"),
]

for workload in ["single_int", "mixed", "string"]:
    print(f"== {workload} p50 ns ==")
    hdr = "threads: " + " ".join(f"{t:>8}" for t in [1,2,4,8,16])
    print(hdr)
    for platform, f, label in rows:
        vals = []
        for t in [1,2,4,8,16]:
            try:
                vals.append(f"{p50(f, workload, t):8.2f}")
            except KeyError:
                vals.append("      --")
        print(f"{platform + ' ' + label:>42} " + " ".join(vals))
    print()