import json, os

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
    ("WSL", "wsl_ticklog_nanolog_r10.json", "NanoLog policy r10"),
    ("WSL", "wsl_ticklog_fastwm_r10.json",  "quill-ring fast+wm r10"),
    ("WSL", "wsl_quill_cpp_r7.json",        "C++ quill r7"),
    ("WIN", "windows_ticklog_nanolog_r10.json", "NanoLog policy r10"),
    ("WIN", "windows_ticklog_fastwm_r10.json",  "quill-ring fast+wm r10"),
    ("WIN", "windows_quill_cpp_r7.json",        "C++ quill r7"),
]

for workload in ["single_int", "mixed", "string"]:
    print(f"== {workload} p50 ns ==")
    print("threads: " + " ".join(f"{t:>8}" for t in [1,2,4,8,16]))
    for platform, f, label in rows:
        vals = []
        for t in [1,2,4,8,16]:
            try:
                vals.append(f"{p50(f, workload, t):8.2f}")
            except KeyError:
                vals.append("      --")
        print(f"{platform + ' ' + label:>42} " + " ".join(vals))
    print()