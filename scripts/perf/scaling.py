#!/usr/bin/env python3
"""Scaling analysis for benchmark results.

Reads one or more result files produced by bench.py and, for each scenario,
reports:

  * T(n) medians,
  * per-page cost,
  * consecutive doubling ratios,
  * a log-log least-squares exponent k in T ∝ n^k,
  * the best fitting model among O(1), O(log n), O(n), O(n log n), O(n²),
    chosen by relative RMSE of a one-parameter fit.

    ./scaling.py benchmarks/results/<sha>/baseline-matrix.json
    ./scaling.py --markdown benchmarks/results/<sha>/*.json

A single doubling ratio is never enough to classify growth, so the exponent and
the model fit are computed over every measured size.
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from pathlib import Path

MODELS = {
    "O(1)": lambda n: 1.0,
    "O(log n)": lambda n: math.log2(max(n, 2)),
    "O(n)": lambda n: float(n),
    "O(n log n)": lambda n: n * math.log2(max(n, 2)),
    "O(n^2)": lambda n: float(n) * n,
}


def load(paths: list[Path]) -> dict[str, list[tuple[int, float, dict]]]:
    by_scenario: dict[str, list[tuple[int, float, dict]]] = {}
    for path in paths:
        payload = json.loads(path.read_text())
        for record in payload["results"]:
            scenario = record.get("scenario") or record.get("label")
            pages = record.get("pages")
            if pages is None:
                manifest = record.get("manifest") or {}
                pages = manifest.get("pages") or record.get("input", {}).get("page_files")
            if pages is None:
                continue
            by_scenario.setdefault(scenario, []).append(
                (int(pages), record["timing_seconds"]["median"], record))
    for values in by_scenario.values():
        values.sort()
    return by_scenario


def fit_exponent(points: list[tuple[int, float]]) -> float | None:
    """Least-squares slope of log T against log n."""
    pts = [(math.log(n), math.log(t)) for n, t in points if n > 0 and t > 0]
    if len(pts) < 2:
        return None
    n = len(pts)
    sx = sum(x for x, _ in pts)
    sy = sum(y for _, y in pts)
    sxx = sum(x * x for x, _ in pts)
    sxy = sum(x * y for x, y in pts)
    denom = n * sxx - sx * sx
    if abs(denom) < 1e-12:
        return None
    return (n * sxy - sx * sy) / denom


def best_model(points: list[tuple[int, float]]) -> tuple[str, float]:
    """Fit T = c * f(n) for each candidate f, return the one with lowest RMSE%."""
    best = ("?", float("inf"))
    for name, f in MODELS.items():
        # least squares for a single scale factor c
        num = sum(f(n) * t for n, t in points)
        den = sum(f(n) ** 2 for n, t in points)
        if den == 0:
            continue
        c = num / den
        err = 0.0
        for n, t in points:
            predicted = c * f(n)
            err += ((predicted - t) / t) ** 2
        rmse = math.sqrt(err / len(points)) * 100
        if rmse < best[1]:
            best = (name, rmse)
    return best


def classify(exponent: float | None) -> str:
    if exponent is None:
        return "unknown"
    if exponent < 0.3:
        return "flat / dominated by fixed cost"
    if exponent < 0.9:
        return "sublinear (fixed cost still significant)"
    if exponent <= 1.15:
        return "linear"
    if exponent <= 1.35:
        return "slightly superlinear (n log n-ish)"
    if exponent <= 1.7:
        return "superlinear — investigate"
    return "strongly superlinear — likely quadratic"


def report(by_scenario: dict, markdown: bool) -> None:
    for scenario, values in sorted(by_scenario.items()):
        points = [(n, t) for n, t, _ in values]
        exponent = fit_exponent(points)
        model, rmse = best_model(points)

        if markdown:
            print(f"\n### {scenario}\n")
            print("| pages | median s | ms/page | ratio vs prev | size ratio | peak RSS MB | out files |")
            print("| ----- | -------- | ------- | ------------- | ---------- | ----------- | --------- |")
        else:
            print(f"\n=== {scenario}")

        prev = None
        for pages, median, record in values:
            rss = (record.get("resources", {}).get("peak_rss_bytes") or 0) / 1e6
            files = record.get("output", {}).get("files", 0)
            ratio = f"{median / prev[1]:.2f}x" if prev else "—"
            size_ratio = f"{pages / prev[0]:.2f}x" if prev else "—"
            if markdown:
                print(f"| {pages} | {median:.3f} | {median / pages * 1000:.3f} | {ratio} | "
                      f"{size_ratio} | {rss:.0f} | {files} |")
            else:
                print(f"  n={pages:6d}  T={median:8.3f}s  {median / pages * 1000:6.3f} ms/page  "
                      f"ratio={ratio:>6}  rss={rss:5.0f}MB")
            prev = (pages, median)

        line = (f"exponent k≈{exponent:.2f} ({classify(exponent)}); "
                f"best fit {model} (RMSE {rmse:.1f}%)") if exponent is not None else "not enough points"
        print(f"\n**Scaling:** {line}" if markdown else f"  → {line}")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("results", nargs="+", type=Path)
    ap.add_argument("--markdown", action="store_true")
    args = ap.parse_args()
    report(load(args.results), args.markdown)
    return 0


if __name__ == "__main__":
    sys.exit(main())
