#!/usr/bin/env python3
"""Aggregate Run 12 (all backends + quill + nanolog) results into a markdown report."""
import json
from pathlib import Path

OUT = Path(__file__).resolve().parent / "results_quill"
THREADS = [1, 2, 4, 8, 16]
WORKLOADS = ["single_int", "mixed", "string"]

CANDIDATES = [
    # (label, win_file, wsl_file)
    ("ticklog-custom",     "windows_ticklog_custom_r12.json",     "wsl_ticklog_custom_r12.json"),
    ("ticklog-rtrb",       "windows_ticklog_rtrb_r12.json",       "wsl_ticklog_rtrb_r12.json"),
    ("ticklog-ringbuf",    "windows_ticklog_ringbuf_r12.json",    "wsl_ticklog_ringbuf_r12.json"),
    ("ticklog-ringbuffer", "windows_ticklog_ringbuffer_r12.json", "wsl_ticklog_ringbuffer_r12.json"),
    ("ticklog-triple",     "windows_ticklog_triple_r12.json",     "wsl_ticklog_triple_r12.json"),
    ("ticklog-quill",      "windows_ticklog_quill_r12.json",      "wsl_ticklog_quill_r12.json"),
    ("ticklog-nanolog",    "windows_ticklog_nanolog_r12.json",    "wsl_ticklog_nanolog_r12.json"),
    ("C++ quill",          "windows_quill_cpp_r12.json",          "wsl_quill_cpp_r12.json"),
    ("C++ NanoLog",        None,                                  "wsl_nanolog_cpp_r12.json"),
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
    """{(workload, threads): row}"""
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
    """One table: rows=candidates, cols=threads, values=metric for one platform+workload."""
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
    # build markdown
    head = "| candidate | " + " | ".join(f"t{t}" for t in THREADS) + " |"
    sep = "|---" * (len(THREADS) + 1) + "|"
    body = ["| " + label + " | " + " | ".join(vals) + " |" for label, vals in rows]
    return "\n".join([head, sep] + body)


def main():
    lines = [
        "# Run 12: all ticklog backends + quill/nanolog policies (WSL + Windows)",
        "",
        "Date: 2026-09-23. Post-split rtrb (commit 98562b9).",
        "BATCH=1000, SAMPLES=1000 (1M msg/config), threads `1,2,4,8,16`, RDTSC.",
        "Calibration: Windows `0.313132820 ns/tick`, WSL `0.313148759 ns/tick`.",
        "Rings: Win 64 MiB / WSL 32 MiB; rtrb `--chunk-size 512`.",
        "",
        "## Builds & sizing",
        "",
        "| Candidate | Features | Buffer |",
        "|---|---|---|",
        "| ticklog-custom | (none — custom ring, Block) | ring 64/32 MiB |",
        "| ticklog-rtrb | backend-rtrb (split P/C) | rtrb ring 64/32 MiB, chunk 512 |",
        "| ticklog-ringbuf | backend-ringbuf | ringbuf 64/32 MiB |",
        "| ticklog-ringbuffer | backend-ringbuffer | ringbuffer 64/32 MiB |",
        "| ticklog-triple | backend-triple-buffer | triple buffer 64/32 MiB |",
        "| ticklog-quill | policy-quill (custom ring) | arena regions, ring 64/32 MiB |",
        "| ticklog-nanolog | policy-nanolog (custom ring) | pool max(8MiB/ring,1) segs |",
        "| C++ quill v12.1.0 | clang, QUILL_NO_EXCEPTIONS | per-thread SPSC 64/32 MiB |",
        "| C++ NanoLog | clang++, Linux-only | default staging |",
        "",
        "## Result files",
        "",
        "`results_quill/{windows,wsl}_*_r12.json` and `results_quill/wsl_nanolog_cpp_r12.json`.",
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

    dest = OUT.parent / "BACKENDS-QUILL-NANOLOG-Run12.md"
    dest.write_text("\n".join(lines), encoding="utf-8")
    print(f"wrote {dest}")

    # also print a compact p50 single_int summary to stdout
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


if __name__ == "__main__":
    main()
