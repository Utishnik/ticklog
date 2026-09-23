#!/usr/bin/env python3
"""Aggregate Run 14 (ring 8 MiB) results into a markdown report."""
import json
from pathlib import Path

OUT = Path(__file__).resolve().parent / "results_quill"
THREADS = [1, 2, 4, 8, 16]
WORKLOADS = ["single_int", "mixed", "string"]

CANDIDATES = [
    # (label, win_file, wsl_file)
    ("ticklog-custom-drop", "windows_ticklog_custom_drop_r14.json", "wsl_ticklog_custom_drop_r14.json"),
    ("ticklog-rtrb-drop",   "windows_ticklog_rtrb_drop_r14.json",   "wsl_ticklog_rtrb_drop_r14.json"),
    ("ticklog-quill",       "windows_ticklog_quill_r14.json",       "wsl_ticklog_quill_r14.json"),
    ("ticklog-nanolog",     "windows_ticklog_nanolog_r14.json",     "wsl_ticklog_nanolog_r14.json"),
    ("cpp-quill (8MiB q)",  "windows_quill_cpp_r14.json",           "wsl_quill_cpp_r14.json"),
    ("cpp-nanolog",         None,                                   "wsl_nanolog_cpp_r14.json"),
]


def load(path):
    if path is None:
        return None
    p = OUT / path
    if not p.exists():
        return None
    with open(p) as f:
        return json.load(f)


def index(data):
    if data is None:
        return {}
    return {(r["workload"], r["threads"]): r for r in data["results"]}


def fmt(v, nd=2):
    if v is None:
        return "—"
    if v >= 1_000_000:
        return f"{v/1e6:.1f}M"
    if v >= 1000:
        return f"{v/1e3:.1f}k"
    return f"{v:.{nd}f}"


def header_meta():
    lines = []
    for label, wf, wsf in CANDIDATES:
        for plat, f in (("WIN", wf), ("WSL", wsf)):
            d = load(f)
            if d:
                lines.append(
                    f"- **{label} ({plat})**: candidate=`{d['candidate']}` "
                    f"os={d['os']} ns/tick={d['ns_per_tick']} samples={d['samples']} "
                    f"total={d['total_messages']}"
                )
    return "\n".join(lines)


def table(metric, workload, plat):
    rows = []
    for label, wf, wsf in CANDIDATES:
        f = wf if plat == "WIN" else wsf
        d = load(f)
        idx = index(d)
        vals = []
        for t in THREADS:
            r = idx.get((workload, t))
            if r is None:
                vals.append("—")
            else:
                v = r[metric]
                vals.append(fmt(v) if metric == "throughput" else f"{v:.1f}")
        if any(v != "—" for v in vals):
            rows.append((label, vals))
    head = "| candidate | " + " | ".join(f"t{t}" for t in THREADS) + " |"
    sep = "|---" * (len(THREADS) + 1) + "|"
    body = ["| " + label + " | " + " | ".join(vals) + " |" for label, vals in rows]
    return "\n".join([head, sep] + body)


def main():
    lines = [
        "# Run 14: ring 8 MiB (ticklog custom/rtrb/quill/nanolog + C++ quill/nanolog)",
        "",
        "Date: 2026-09-23. Post-split rtrb (commit 98562b9), same harness set as Run 13.",
        "BATCH=1000, SAMPLES=1000 (1M msg/config), threads `1,2,4,8,16`, RDTSC.",
        "Calibration: Windows `0.313132820 ns/tick`, WSL `0.313148759 ns/tick`.",
        "Rings: **8 MiB (8388608) both platforms**. rtrb `--chunk-size 64`.",
        "C++ quill rebuilt with `QUILL_BENCH_QUEUE_CAPACITY=8388608` (per-thread SPSC queue 8 MiB).",
        "C++ NanoLog has **no ring-size parameter** (per-thread staging buffers sized by `NanoLog::preallocate()`); run as-is.",
        "",
        "## Builds & sizing",
        "",
        "| Candidate | Features | Buffer |",
        "|---|---|---|",
        "| ticklog-custom-drop | policy-drop (custom ring) | ring 8 MiB |",
        "| ticklog-rtrb-drop | backend-rtrb,policy-drop | rtrb ring 8 MiB, chunk 64 |",
        "| ticklog-quill | policy-quill (custom ring) | arena regions, ring 8 MiB |",
        "| ticklog-nanolog | policy-nanolog (custom ring) | pool max(8MiB/ring,1) segs |",
        "| cpp-quill (8MiB q) | quill v12.1.0 header-only | per-thread SPSC queue 8 MiB (compile-time) |",
        "| cpp-nanolog | NanoLog staging buffers | no ring parameter (fixed staging buffers) |",
        "",
        "## Result files",
        "",
        "`results_quill/{windows,wsl}_*_r14.json`.",
        "",
        "---",
        "",
    ]

    for plat in ("WSL", "WIN"):
        lines.append(f"## {plat}")
        lines.append("")
        for wl in WORKLOADS:
            lines.append(f"### {wl} — p50 latency (ns)")
            lines.append("")
            lines.append(table("p50", wl, plat))
            lines.append("")
            lines.append(f"### {wl} — throughput (r/s)")
            lines.append("")
            lines.append(table("throughput", wl, plat))
            lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## Meta")
    lines.append("")
    lines.append(header_meta())
    lines.append("")

    dest = OUT.parent / "BACKENDS-QUILL-NANOLOG-Run14.md"
    dest.write_text("\n".join(lines), encoding="utf-8")
    print(f"wrote {dest}")

    print("\n=== p50 single_int ===")
    for plat in ("WSL", "WIN"):
        print(f"-- {plat} --")
        for label, wf, wsf in CANDIDATES:
            f = wf if plat == "WIN" else wsf
            idx = index(load(f))
            vals = []
            for t in THREADS:
                r = idx.get(("single_int", t))
                vals.append(f"{r['p50']:.1f}" if r else "—")
            print(f"  {label:22s} " + "  ".join(f"t{t}:{v}" for t, v in zip(THREADS, vals)))

    print("\n=== throughput single_int (r/s) ===")
    for plat in ("WSL", "WIN"):
        print(f"-- {plat} --")
        for label, wf, wsf in CANDIDATES:
            f = wf if plat == "WIN" else wsf
            idx = index(load(f))
            vals = []
            for t in THREADS:
                r = idx.get(("single_int", t))
                vals.append(fmt(r['throughput']) if r else "—")
            print(f"  {label:22s} " + "  ".join(f"t{t}:{v}" for t, v in zip(THREADS, vals)))


if __name__ == "__main__":
    main()