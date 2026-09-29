#!/usr/bin/env python3
"""Aggregate Run 18 (rtrb-block in-place assembly vs C++ quill, 8 MiB) results."""
import json
from pathlib import Path

OUT = Path(__file__).resolve().parent / "results_quill"
THREADS = [1, 2, 4, 8, 16]
WORKLOADS = ["single_int", "mixed", "string"]

CANDIDATES = [
    # (label, win_file, wsl_file)
    ("ticklog-rtrb-block (r18)", "windows_ticklog_rtrb_block_r18.json", "wsl_ticklog_rtrb_block_r18.json"),
    ("ticklog-rtrb-block (r17)", "windows_ticklog_rtrb_block_r17.json", "wsl_ticklog_rtrb_block_r17.json"),
    ("cpp-quill v13 (r18)",      "windows_quill_cpp_v13_r18.json",       "wsl_quill_cpp_v13_r18.json"),
    ("cpp-quill v12 (r18)",      "windows_quill_cpp_r18.json",           "wsl_quill_cpp_r18.json"),
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
        "# Run 18: ticklog rtrb + Block (in-place assembly) vs C++ quill (8 MiB queues)",
        "",
        "Date: 2026-09-30. Same harness set and parameters as Run 17: BATCH=1000,",
        "SAMPLES=1000 (1M msg/config), threads `1,2,4,8,16`, RDTSC. Calibration:",
        "Windows `0.313132820 ns/tick`, WSL `0.313148759 ns/tick`. Rings: **8 MiB",
        "(8388608) both platforms**. rtrb `--chunk-size 64`. C++ quill harnesses",
        "reused from Run 17 (`QUILL_BENCH_QUEUE_CAPACITY=8388608`, v13.0.0 =",
        "`quill_harness_8m_v13`, v12.1.0 baseline = `quill_harness_8m`).",
        "",
        "**Difference from Run 17**: ticklog commit `226deb9` — the producer",
        "assembles each record in place (no staging zero-fill, no staging→ring",
        "memcpy on the fifo path); Run 17's rtrb-block row is kept as the",
        "reference. **hotpath: OFF** (`backend-rtrb,policy-block` only, no",
        "`hotpath-profiler`): `#[hotpath::measure]` expands to a no-op.",
        "",
        "## Builds & sizing",
        "",
        "| Candidate | Features | Buffer |",
        "|---|---|---|",
        "| ticklog-rtrb-block (r18) | backend-rtrb,policy-block | rtrb ring 8 MiB, chunk 64, Backpressure::Block |",
        "| cpp-quill v13 (8MiB q) | quill v13.0.0 header-only | per-thread SPSC queue 8 MiB (compile-time) |",
        "| cpp-quill v12 (8MiB q) | quill v12.1.0 header-only | per-thread SPSC queue 8 MiB (compile-time) |",
        "",
        "## Result files",
        "",
        "`results_quill/{windows,wsl}_ticklog_rtrb_block_r18.json`,",
        "`results_quill/{windows,wsl}_quill_cpp_v13_r18.json`,",
        "`results_quill/{windows,wsl}_quill_cpp_r18.json` (+ `.log` / `.log.err`).",
        "Scripts: `build_{win,wsl}_r18.{ps1,sh}`, `run_{win,wsl}_r18.{ps1,sh}`,",
        "`summarize_r18.py`.",
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
    lines.append("## Tails: single_int (ns)")
    lines.append("")
    for metric, title in (("p99", "### p99"), ("p999", "### p999")):
        lines.append(title)
        lines.append("")
        lines.append("| OS | candidate | " + " | ".join(f"t{t}" for t in THREADS) + " |")
        lines.append("|---|---|" + "---|" * len(THREADS))
        for plat in ("WSL", "WIN"):
            for label, wf, wsf in CANDIDATES:
                f = wf if plat == "WIN" else wsf
                idx = index(load(f))
                vals = []
                for t in THREADS:
                    r = idx.get(("single_int", t))
                    vals.append(f"{r[metric]:.1f}" if r else "—")
                if any(v != "—" for v in vals):
                    lines.append(f"| {plat} | {label} | " + " | ".join(vals) + " |")
        lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## Meta")
    lines.append("")
    lines.append(header_meta())
    lines.append("")

    dest = OUT.parent / "RTRB-BLOCK-vs-QUILL-Run18.md"
    dest.write_text("\n".join(lines), encoding="utf-8")
    print(f"wrote {dest}")

    print("\n=== p50 r17 -> r18 delta (ticklog-rtrb-block) ===")
    for plat in ("WSL", "WIN"):
        print(f"-- {plat} --")
        i17 = index(load("windows_ticklog_rtrb_block_r17.json" if plat == "WIN" else "wsl_ticklog_rtrb_block_r17.json"))
        i18 = index(load("windows_ticklog_rtrb_block_r18.json" if plat == "WIN" else "wsl_ticklog_rtrb_block_r18.json"))
        for wl in WORKLOADS:
            vals = []
            for t in THREADS:
                a, b = i17.get((wl, t)), i18.get((wl, t))
                if a and b:
                    vals.append(f"t{t}: {a['p50']:.1f}->{b['p50']:.1f} ({100*(b['p50']-a['p50'])/a['p50']:+.0f}%)")
            print(f"  {wl:11s} " + "  ".join(vals))

    print("\n=== p50 r18 quill-vs-ticklog ===")
    for plat in ("WSL", "WIN"):
        print(f"-- {plat} --")
        tb = index(load("windows_ticklog_rtrb_block_r18.json" if plat == "WIN" else "wsl_ticklog_rtrb_block_r18.json"))
        q13 = index(load("windows_quill_cpp_v13_r18.json" if plat == "WIN" else "wsl_quill_cpp_v13_r18.json"))
        for wl in WORKLOADS:
            vals = []
            for t in THREADS:
                a, b = tb.get((wl, t)), q13.get((wl, t))
                if a and b:
                    vals.append(f"t{t}: {a['p50']:.1f}/{b['p50']:.1f}")
            print(f"  {wl:11s} (tick/quill13) " + "  ".join(vals))


if __name__ == "__main__":
    main()
