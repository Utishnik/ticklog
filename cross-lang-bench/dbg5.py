import sys, io, re

REPO = "/mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench"
sys.path.insert(0, REPO)
import compare_block_vs_drop as C

md = io.open(REPO + "/BENCHMARKS.md", encoding="utf-8").read()
idx = md.find("# Run 4")
print("find('# Run 4'):", idx, "-> file slice found" if idx >= 0 else "NOT FOUND")
if idx >= 0:
    b4 = md[idx:md.find("# Run 3") if md.find("# Run 3") > idx else len(md)]
    print("Run4 byte len:", len(b4))
    heads = re.findall(r"^### [^\n]+$", b4, re.M)[:14]
    print("first ### headers in Run4:")
    for h in heads:
        print("   ", repr(h))
    # which regex does the module use?
    print("module raw regex:", repr(C.RAW_LAT_RE if hasattr(C, "RAW_LAT_RE") else "n/a"))
