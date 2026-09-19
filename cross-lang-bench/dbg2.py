import sys
sys.path.insert(0, "/mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench")
import compare_block_vs_drop as C

md = open("/mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench/BENCHMARKS.md", encoding="utf-8").read()
print("Run4 idx:", md.find("# Run 4"))

resp = C.parse_run4(md, "ticklog")
print("parse_run4 type:", type(resp))
if isinstance(resp, tuple):
    print("parts:", [type(x) for x in resp])
    for x in resp:
        if isinstance(x, dict):
            print("  dict len", len(x), "first:", list(x.items())[:1])
        else:
            print("  other:", x)
