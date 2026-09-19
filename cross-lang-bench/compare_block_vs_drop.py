#!/usr/bin/env python3
"""
Side-by-side comparison: published Run 4 (WSL2, same host) numbers from
BENCHMARKS.md vs a fresh run built with ticklog `Backpressure::Block`.

The old (Drop) cells live in the "Run 4" section of BENCHMARKS.md; the new
(Block) cells are the usual per-candidate harness JSON (`results/*.json`
schema, same as results.py consumes).

Usage:
    python3 compare_block_vs_drop.py results_1/ticklog.json \
        [--benches BENCHMARKS.md] [--candidate ticklog] \
        [--two-core results_2/ticklog.json]
"""

import argparse
import json
import re

WORKLOADS = ["single_int", "mixed", "string"]
THREADS = [1, 2, 4, 8, 16]
METRICS = ["p50", "p95", "p99", "p999", "max"]


def render(header, rows):
    """Render a left-aligned markdown table, like results.py."""
    widths = [max(len(row[i]) for row in [header] + rows) for i in range(len(header))]
    fmt = "| " + " | ".join("{" + ":<{w}".format(w=w) + "}" for w in widths) + " |"
    sep = "|-" + "-|-".join("-" * w for w in widths) + "-|"
    lines = [fmt.format(*header), sep] + [fmt.format(*row) for row in rows]
    return "\n".join(lines)


def pct(new, old):
    return (new - old) / old * 100 if old else 0.0


def parse_run4(md, candidate):
    """Extract `candidate` rows from the Run 4 section of BENCHMARKS.md.

    Returns (latency, throughput, two_core):
      latency[(workload, threads)] -> [p50, p95, p99, p999, max]
      throughput[threads]          -> {workload: r/s}
      two_core                     -> [p50, p95, p99, p999, max, r/s] or None
    """
    idx = md.find("# Run 4")
    if idx == -1:
        raise SystemExit("error: '# Run 4' section not found in BENCHMARKS.md")
    lines = md[idx:].splitlines()

    lat, thr, two_core = {}, {}, None
    i = 0
    while i < len(lines):
        line = lines[i]

        m = re.match(r"^### (single_int|mixed|string): (\d+) thread\(s\)$", line)
        if m:
            wl, n = m.group(1), int(m.group(2))
            i += 1
            while i < len(lines) and not lines[i].startswith("|"):
                i += 1
            while i < len(lines) and lines[i].startswith("|"):
                cells = [c.strip() for c in lines[i].strip().strip("|").split("|")]
                if cells and cells[0] == candidate and len(cells) == 6:
                    lat[(wl, n)] = [float(x) for x in cells[1:6]]
                    break
                i += 1
            i += 1
            continue

        m = re.match(r"^### (\d+) thread\(s\)$", line)
        if m:
            n = int(m.group(1))
            i += 1
            while i < len(lines) and not lines[i].startswith("|"):
                i += 1
            while i < len(lines) and lines[i].startswith("|"):
                cells = [c.strip() for c in lines[i].strip().strip("|").split("|")]
                if cells and cells[0] == candidate and len(cells) == 4:
                    thr[n] = {
                        WORKLOADS[k]: int(cells[1 + k].replace(",", ""))
                        for k in range(3)
                    }
                    break
                i += 1
            i += 1
            continue

        if line.startswith("## Two-Core SPSC (Run 4)"):
            i += 1
            while i < len(lines) and not lines[i].startswith("|"):
                i += 1
            while i < len(lines) and lines[i].startswith("|"):
                cells = [c.strip() for c in lines[i].strip().strip("|").split("|")]
                if cells and cells[0] == candidate and len(cells) == 7:
                    vals = [float(x) for x in cells[1:6]] + [int(cells[6].replace(",", ""))]
                    two_core = vals
                    break
                i += 1
            i += 1
            continue

        i += 1

    return lat, thr, two_core


def load_new(path):
    """Load a harness JSON file; returns (results, two_core_trimmed)."""
    with open(path) as f:
        d = json.load(f)
    lat = {(r["workload"], r["threads"]): [r[m] for m in METRICS] for r in d["results"]}
    thr = {}
    for r in d["results"]:
        thr.setdefault(r["threads"], {})[r["workload"]] = r["throughput"]
    two_core = None
    if len(d["results"]) == 3 and all(
        r["workload"] in WORKLOADS and r["threads"] == 1 for r in d["results"]
    ):
        two_core = {
            r["workload"]: [r[m] for m in METRICS] + [r["throughput"]]
            for r in d["results"]
        }
    return lat, thr, two_core


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("new", help="new Block-policy results JSON (ticklog.json)")
    ap.add_argument("--benches", default="BENCHMARKS.md", help="markdown with Run 4 tables")
    ap.add_argument("--candidate", default="ticklog", help="candidate to compare")
    ap.add_argument(
        "--two-core", help="optional two-core pinned Block results JSON (results_2/ticklog.json)"
    )
    args = ap.parse_args()

    with open(args.benches, encoding="utf-8") as f:
        old_lat, old_thr, old_two = parse_run4(f.read(), args.candidate)

    new_lat, new_thr, _ = load_new(args.new)
    new_two = None
    if args.two_core:
        with open(args.two_core) as f:
            d = json.load(f)
        new_two = {}
        for r in d["results"]:
            if r["workload"] in WORKLOADS and r["threads"] == 1:
                new_two[r["workload"]] = [r[m] for m in METRICS] + [r["throughput"]]

    cand = args.candidate
    print(f"# {cand}: Backpressure::Block vs published Run 4 (Drop)\n")
    print(
        "Old cells are the published Run 4 numbers parsed from `BENCHMARKS.md` "
        f"(candidate `{cand}`, Drop policy). New cells are a rerun of the same "
        "harness on the same WSL2 host built with `Backpressure::Block`. Same "
        "protocol: BATCH=1000, 10M messages per config, null sink.\n"
    )

    print("## Latency (ns) — Drop → Block, Δ%\n")
    for wl in WORKLOADS:
        print(f"### {wl}\n")
        header = ["threads", "p50", "p95", "p99", "p999", "max"]
        rows = []
        for n in THREADS:
            o = old_lat.get((wl, n))
            v = new_lat.get((wl, n))
            if not (o and v):
                continue
            cells = [str(n)]
            for i, name in enumerate(METRICS):
                cells.append(f"{o[i]:.1f} -> {v[i]:.1f} ({pct(v[i], o[i]):+.1f}%)")
            rows.append(cells)
        if rows:
            print(render(header, rows))
            print()

    print("## Throughput (r/s) — Drop → Block, Δ%\n")
    for n in THREADS:
        print(f"### {n} thread(s)\n")
        header = ["workload", "Drop", "Block", "Δ%"]
        rows = []
        for wl in WORKLOADS:
            o = (old_thr.get(n) or {}).get(wl)
            v = (new_thr.get(n) or {}).get(wl)
            if not (o and v):
                continue
            rows.append([
                wl,
                f"{o:,}",
                f"{v:,}",
                f"{pct(v, o):+.1f}%",
            ])
        if rows:
            print(render(header, rows))
            print()

    if old_two and new_two:
        print("## Two-Core SPSC (producer core 0 / drain core 1)\n")
        header = ["workload", "p50", "p95", "p99", "p999", "max", "r/s"]
        rows = []
        for wl in WORKLOADS:
            o, v = old_two, new_two.get(wl)
            if not (o and v):
                continue
            cells = [wl]
            for i in range(5):
                cells.append(f"{o[i]:.1f} -> {v[i]:.1f} ({pct(v[i], o[i]):+.1f}%)")
            cells.append(f"{o[5]:,} -> {v[5]:,} ({pct(v[5], o[5]):+.1f}%)")
            rows.append(cells)
        if rows:
            print(render(header, rows))
            print()

    print(
        "Note: best-effort numbers on an unisolated WSL2 box; Block drains the "
        "ring with a busy wait, so its throughput ceiling is the drain speed and "
        "p95-p999 reflect queueing while producers spin. Not comparable to "
        "Run 2 (different hardware)."
    )


if __name__ == "__main__":
    main()