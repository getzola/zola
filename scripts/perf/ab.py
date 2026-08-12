#!/usr/bin/env python3
"""Interleaved A/B comparison of two zola binaries.

Why interleaved: a sequential "N runs of A, then N runs of B" comparison on a
laptop measures thermal state and background load as much as it measures the
code. Early in this program that produced a −69% result that was entirely an
artefact (docs/performance/OPTIMIZATIONS.md records it). Here the two binaries
alternate within every round, and the order flips each round, so any drift over
the session hits both sides equally.

Usage:
    scripts/perf/ab.py --a /tmp/zola-A --b /tmp/zola-B \
        --site benchmarks/sites/mixed-realistic-4000 \
        --site benchmarks/proxies/vomaste-live \
        --rounds 3 [--warmup] [--json out.json]

Reports the median of each side and the delta. Deltas smaller than the spread
between a side's own runs are reported as noise, because they are.
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path

RSS_RE = re.compile(r"^\s*(\d+)\s+maximum resident set size", re.M)
TIME_RE = re.compile(r"^\s*([\d.]+) real\s+([\d.]+) user\s+([\d.]+) sys", re.M)


def run_once(binary: Path, site: Path, out_dir: Path) -> dict:
    """One full build into a fresh output directory.

    Returns wall, CPU time and peak RSS. CPU time matters as much as wall here:
    a build that writes gigabytes stalls on the filesystem in ways that move
    wall time around without saying anything about the code, while user+sys is
    insensitive to those stalls.
    """
    if out_dir.exists():
        shutil.rmtree(out_dir)
    cmd = ["/usr/bin/time", "-l", str(binary), "build", "--force", "-o", str(out_dir)]
    start = time.monotonic()
    proc = subprocess.run(cmd, cwd=site, capture_output=True, text=True)
    wall = time.monotonic() - start
    if proc.returncode != 0:
        sys.exit(f"build failed ({binary} in {site}):\n{proc.stderr[-2000:]}")
    rss = RSS_RE.search(proc.stderr)
    cpu = TIME_RE.search(proc.stderr)
    return {
        "wall_s": wall,
        "peak_rss_mb": int(rss.group(1)) / 1024 / 1024 if rss else None,
        "user_s": float(cpu.group(2)) if cpu else None,
        "sys_s": float(cpu.group(3)) if cpu else None,
        "cpu_s": float(cpu.group(2)) + float(cpu.group(3)) if cpu else None,
    }


def summarise(samples: list[dict], key: str) -> tuple[float, float]:
    values = [s[key] for s in samples if s[key] is not None]
    if not values:
        return (float("nan"), float("nan"))
    return (statistics.median(values), max(values) - min(values))


def paired(samples: dict, key: str) -> dict:
    """Per-round B-vs-A deltas.

    Interleaving exists so the two sides can be compared *within* a round; the
    spread of the absolute numbers across rounds is drift that the pairing
    already cancels. A result is called only when every round agrees on the
    sign, which is a 1-in-2^n coincidence under the null.
    """
    pairs = [
        (a[key] - b[key]) / a[key] * 100
        for a, b in zip(samples["a"], samples["b"])
        if a[key] is not None and b[key] is not None
    ]
    if not pairs:
        return {"deltas_pct": [], "median_pct": float("nan"), "unanimous": False}
    return {
        "deltas_pct": [-p for p in pairs],  # negative = B faster
        "median_pct": -statistics.median(pairs),
        "unanimous": all(p > 0 for p in pairs) or all(p < 0 for p in pairs),
    }


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--a", required=True, type=Path, help="baseline binary")
    ap.add_argument("--b", required=True, type=Path, help="candidate binary")
    ap.add_argument("--site", required=True, type=Path, action="append", help="site to build (repeatable)")
    ap.add_argument("--rounds", type=int, default=3)
    ap.add_argument("--warmup", action="store_true", help="one discarded build per side first")
    ap.add_argument("--json", type=Path, help="write raw samples here")
    args = ap.parse_args()

    for binary in (args.a, args.b):
        if not binary.is_file():
            sys.exit(f"not a binary: {binary}")

    results = {}
    with tempfile.TemporaryDirectory(prefix="zola-ab-") as tmp:
        out_dir = Path(tmp) / "public"
        for site in args.site:
            site = site.resolve()
            if not (site / "config.toml").is_file():
                sys.exit(f"not a zola site: {site}")
            print(f"\n=== {site.name} ===", flush=True)
            samples = {"a": [], "b": []}

            if args.warmup:
                for side, binary in (("a", args.a), ("b", args.b)):
                    run_once(binary, site, out_dir)
                    print(f"  warmup {side}: done", flush=True)

            for r in range(args.rounds):
                # Flip the order every round so neither side always runs on a
                # colder machine or a warmer page cache.
                order = [("a", args.a), ("b", args.b)]
                if r % 2:
                    order.reverse()
                for side, binary in order:
                    sample = run_once(binary, site, out_dir)
                    samples[side].append(sample)
                    print(
                        f"  round {r + 1} {side}: {sample['wall_s']:6.2f} s wall"
                        f"  {sample['cpu_s']:7.1f} s cpu"
                        f"  {sample['peak_rss_mb']:7.1f} MB rss",
                        flush=True,
                    )

            entry = {"a": samples["a"], "b": samples["b"]}
            for key, unit, fmt in (
                ("wall_s", "s wall", "6.2f"),
                ("cpu_s", "s cpu", "7.1f"),
                ("peak_rss_mb", "MB rss", "7.1f"),
            ):
                a_med, a_spread = summarise(samples["a"], key)
                b_med, b_spread = summarise(samples["b"], key)
                pair = paired(samples, key)
                verdict = "" if pair["unanimous"] else "  [rounds disagree on sign]"
                print(
                    f"  {unit:>7}: A {a_med:{fmt}} (spread {a_spread:.2f})"
                    f"   B {b_med:{fmt}} (spread {b_spread:.2f})"
                    f"   paired {pair['median_pct']:+.1f}%{verdict}"
                )
                entry[key] = {
                    "a_median": a_med,
                    "a_spread": a_spread,
                    "b_median": b_med,
                    "b_spread": b_spread,
                    **pair,
                }
            results[site.name] = entry

    if args.json:
        args.json.write_text(
            json.dumps(
                {"a": str(args.a), "b": str(args.b), "rounds": args.rounds, "sites": results},
                indent=2,
            )
        )
        print(f"\nwrote {args.json}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
