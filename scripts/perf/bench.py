#!/usr/bin/env python3
"""Zola build benchmark runner.

Measures `zola build` (never cargo compilation) on synthetic scenario sites and
on external real-world sites, with statistical repetition via hyperfine.

    # generate + benchmark a scenario matrix
    ./bench.py matrix --scenarios simple-pages,mixed-realistic --sizes 100,1000

    # benchmark one external site (never modified, output goes to a temp dir)
    ./bench.py site --path ~/dev/example.com --label example
    ZOLA_PERF_SITE=~/dev/example.com ./bench.py site

Results are written to benchmarks/results/<git-sha>/<label>.json.

Benchmark hygiene notes
-----------------------
* The binary must be built with `scripts/perf/build.sh`, which pins the release
  profile (see docs/performance/BASELINE.md) and neutralises the developer's
  global `~/.cargo/config.toml`.
* Every measured run passes `--force`, so each run cleans a *populated* output
  directory — the same work a real rebuild into an existing `public/` does. A
  warmup run guarantees the directory is populated before the first measured run.
* The output directory always lives outside the site so external sites are never
  written to.
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
ZOLA = REPO / "target" / "release" / "zola"
SITES = REPO / "benchmarks" / "sites"
RESULTS = REPO / "benchmarks" / "results"
GEN = REPO / "scripts" / "perf" / "gen_site.py"

DEFAULT_SIZES = [100, 250, 500, 1000, 2000, 4000, 8000]
DEFAULT_SCENARIOS = [
    "simple-pages", "dense-internal-links", "many-taxonomies", "deep-sections",
    "template-heavy", "markdown-heavy", "data-heavy", "mixed-realistic",
]


def sh(cmd: list[str], cwd: Path | None = None, check: bool = True) -> str:
    res = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    if check and res.returncode != 0:
        raise RuntimeError(f"command failed: {' '.join(cmd)}\n{res.stdout}\n{res.stderr}")
    return res.stdout.strip()


def git_sha() -> str:
    sha = sh(["git", "rev-parse", "--short", "HEAD"], cwd=REPO)
    dirty = sh(["git", "status", "--porcelain"], cwd=REPO)
    # Untracked/modified files under scripts/ and docs/ do not change the binary,
    # but we still record dirtiness so results are never silently mislabelled.
    return f"{sha}-dirty" if dirty else sha


def machine_metadata() -> dict:
    def sysctl(key: str) -> str:
        try:
            return sh(["sysctl", "-n", key])
        except Exception:
            return ""

    return {
        "os": platform.platform(),
        "arch": platform.machine(),
        "cpu": sysctl("machdep.cpu.brand_string") or platform.processor(),
        "physical_cores": sysctl("hw.physicalcpu"),
        "logical_cores": sysctl("hw.logicalcpu") or str(os.cpu_count()),
        "memory_bytes": sysctl("hw.memsize"),
        "rustc": sh(["rustc", "--version"], check=False),
        "cargo": sh(["cargo", "--version"], check=False),
        "zola_version": sh([str(ZOLA), "--version"], check=False) if ZOLA.exists() else "",
        "git_commit": git_sha(),
        "git_branch": sh(["git", "rev-parse", "--abbrev-ref", "HEAD"], cwd=REPO, check=False),
        "build_profile": "release (pinned, see scripts/perf/build.sh)",
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
    }


def tree_stats(path: Path) -> dict:
    files = 0
    total = 0
    for root, _dirs, names in os.walk(path):
        for n in names:
            p = Path(root) / n
            try:
                total += p.stat().st_size
            except OSError:
                continue
            files += 1
    return {"files": files, "bytes": total}


def site_stats(path: Path) -> dict:
    """Cheap structural description of an input site."""
    content = path / "content"
    md = list(content.rglob("*.md")) if content.exists() else []
    index_md = [p for p in md if p.name.startswith("_index.")]
    internal_links = 0
    for p in md:
        try:
            internal_links += p.read_text(encoding="utf-8", errors="replace").count("](@/")
        except OSError:
            pass
    static = path / "static"
    return {
        "md_files": len(md),
        "section_files": len(index_md),
        "page_files": len(md) - len(index_md),
        "internal_links": internal_links,
        "static_files": sum(1 for _ in static.rglob("*") if _.is_file()) if static.exists() else 0,
        "templates": sum(1 for _ in (path / "templates").rglob("*") if _.is_file())
        if (path / "templates").exists() else 0,
        "input_bytes": tree_stats(content)["bytes"] if content.exists() else 0,
    }


def peak_rss_run(site: Path, out: Path) -> dict:
    """One extra run under /usr/bin/time -l to capture peak RSS and CPU time."""
    cmd = ["/usr/bin/time", "-l", str(ZOLA), "build", "--force", "-o", str(out)]
    res = subprocess.run(cmd, cwd=site, capture_output=True, text=True)
    err = res.stderr
    out_stats = {"peak_rss_bytes": None, "user_seconds": None, "sys_seconds": None,
                 "wall_seconds": None}
    m = re.search(r"^\s*(\d+)\s+maximum resident set size", err, re.M)
    if m:
        out_stats["peak_rss_bytes"] = int(m.group(1))
    m = re.search(r"([\d.]+)\s+real\s+([\d.]+)\s+user\s+([\d.]+)\s+sys", err)
    if m:
        out_stats["wall_seconds"] = float(m.group(1))
        out_stats["user_seconds"] = float(m.group(2))
        out_stats["sys_seconds"] = float(m.group(3))
    if out_stats["wall_seconds"] and out_stats["wall_seconds"] > 0:
        cpu = (out_stats["user_seconds"] or 0) + (out_stats["sys_seconds"] or 0)
        out_stats["cpu_utilization"] = round(cpu / out_stats["wall_seconds"], 2)
    return out_stats


def hyperfine(site: Path, out: Path, runs: int, warmup: int, threads: int | None) -> dict:
    if not shutil.which("hyperfine"):
        raise RuntimeError("hyperfine not found; install it or use --runs 1 --no-hyperfine")
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as tmp:
        export = Path(tmp.name)
    cmd = [
        "hyperfine", "--warmup", str(warmup), "--runs", str(runs),
        "--export-json", str(export),
        "--command-name", site.name,
        f"{ZOLA} build --force -o {out}",
    ]
    env = dict(os.environ)
    if threads:
        env["RAYON_NUM_THREADS"] = str(threads)
    res = subprocess.run(cmd, cwd=site, capture_output=True, text=True, env=env)
    if res.returncode != 0:
        raise RuntimeError(f"hyperfine failed:\n{res.stdout}\n{res.stderr}")
    data = json.loads(export.read_text())["results"][0]
    export.unlink(missing_ok=True)
    times = sorted(data["times"])
    return {
        "runs": len(times),
        "mean": data["mean"],
        "stddev": data.get("stddev"),
        "median": data.get("median", times[len(times) // 2]),
        "min": data["min"],
        "max": data["max"],
        "user": data.get("user"),
        "system": data.get("system"),
        "times": times,
    }


def ensure_site(scenario: str, pages: int, seed: int) -> Path:
    site = SITES / f"{scenario}-{pages}"
    manifest = site / "manifest.json"
    if manifest.exists():
        existing = json.loads(manifest.read_text())
        if existing.get("seed") == seed and existing.get("requested_pages") == pages:
            return site
    SITES.mkdir(parents=True, exist_ok=True)
    sh([sys.executable, str(GEN), "--scenario", scenario, "--pages", str(pages),
        "--seed", str(seed), "--out", str(site)])
    return site


def benchmark(site: Path, label: str, runs: int, warmup: int,
              threads: int | None = None) -> dict:
    out = Path(tempfile.mkdtemp(prefix=f"zolaperf-{label}-"))
    try:
        # Warmup also guarantees the output dir is populated for every measured run.
        subprocess.run([str(ZOLA), "build", "--force", "-o", str(out)], cwd=site,
                       capture_output=True, text=True, check=True)
        timing = hyperfine(site, out, runs, warmup, threads)
        resources = peak_rss_run(site, out)
        record = {
            "label": label,
            "site": str(site),
            "threads": threads or "all",
            "timing_seconds": timing,
            "resources": resources,
            "output": tree_stats(out),
            "input": site_stats(site),
        }
        manifest = site / "manifest.json"
        if manifest.exists():
            record["manifest"] = json.loads(manifest.read_text())
        return record
    finally:
        shutil.rmtree(out, ignore_errors=True)


def write_results(name: str, payload: dict) -> Path:
    target = RESULTS / git_sha()
    target.mkdir(parents=True, exist_ok=True)
    path = target / f"{name}.json"
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return path


def cmd_matrix(args) -> int:
    scenarios = args.scenarios.split(",") if args.scenarios else DEFAULT_SCENARIOS
    sizes = [int(s) for s in args.sizes.split(",")] if args.sizes else DEFAULT_SIZES
    results = []
    for scenario in scenarios:
        for pages in sizes:
            site = ensure_site(scenario, pages, args.seed)
            label = f"{scenario}-{pages}"
            print(f"[bench] {label} …", flush=True)
            record = benchmark(site, label, args.runs, args.warmup, args.threads)
            record["scenario"] = scenario
            record["pages"] = pages
            results.append(record)
            print(f"[bench] {label}: median {record['timing_seconds']['median']:.3f}s "
                  f"rss {(record['resources']['peak_rss_bytes'] or 0) / 1e6:.0f}MB", flush=True)
    payload = {"machine": machine_metadata(), "results": results}
    path = write_results(args.name, payload)
    print(f"[bench] wrote {path}")
    return 0


def cmd_site(args) -> int:
    raw = args.path or os.environ.get("ZOLA_PERF_SITE")
    if not raw:
        print("error: pass --path or set ZOLA_PERF_SITE", file=sys.stderr)
        return 2
    site = Path(raw).expanduser().resolve()
    if not (site / "config.toml").exists() and not (site / "zola.toml").exists():
        print(f"error: {site} has no config.toml/zola.toml", file=sys.stderr)
        return 2
    label = args.label or site.name
    print(f"[bench] {label} ({site}) …", flush=True)
    record = benchmark(site, label, args.runs, args.warmup, args.threads)
    payload = {"machine": machine_metadata(), "results": [record]}
    path = write_results(args.name or f"site-{label}", payload)
    print(json.dumps(record, indent=2, sort_keys=True))
    print(f"[bench] wrote {path}")
    return 0


def cmd_threads(args) -> int:
    """Parallel-efficiency sweep on a single site."""
    site = (Path(args.path).expanduser().resolve() if args.path
            else ensure_site(args.scenario, args.pages, args.seed))
    results = []
    for threads in [int(t) for t in args.thread_counts.split(",")]:
        label = f"{site.name}-t{threads}"
        print(f"[bench] {label} …", flush=True)
        record = benchmark(site, label, args.runs, args.warmup, threads)
        record["thread_count"] = threads
        results.append(record)
        print(f"[bench] {label}: median {record['timing_seconds']['median']:.3f}s", flush=True)
    base = next((r for r in results if r["thread_count"] == 1), None)
    if base:
        for r in results:
            speedup = base["timing_seconds"]["median"] / r["timing_seconds"]["median"]
            r["speedup_vs_1_thread"] = round(speedup, 3)
            r["parallel_efficiency"] = round(speedup / r["thread_count"], 3)
    payload = {"machine": machine_metadata(), "results": results}
    path = write_results(args.name or f"threads-{site.name}", payload)
    print(f"[bench] wrote {path}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="cmd", required=True)

    common = argparse.ArgumentParser(add_help=False)
    common.add_argument("--runs", type=int, default=5)
    common.add_argument("--warmup", type=int, default=1)
    common.add_argument("--seed", type=int, default=20260811)
    common.add_argument("--threads", type=int, default=None,
                        help="RAYON_NUM_THREADS for the measured runs")
    common.add_argument("--name", default=None, help="result file name (without .json)")

    m = sub.add_parser("matrix", parents=[common], help="scenario × size matrix")
    m.add_argument("--scenarios", default=None)
    m.add_argument("--sizes", default=None)
    m.set_defaults(func=cmd_matrix, name="matrix")

    s = sub.add_parser("site", parents=[common], help="benchmark an external site")
    s.add_argument("--path", default=None)
    s.add_argument("--label", default=None)
    s.set_defaults(func=cmd_site)

    t = sub.add_parser("threads", parents=[common], help="parallel efficiency sweep")
    t.add_argument("--path", default=None)
    t.add_argument("--scenario", default="mixed-realistic")
    t.add_argument("--pages", type=int, default=4000)
    t.add_argument("--thread-counts", default="1,2,4,8,12")
    t.set_defaults(func=cmd_threads)

    args = parser.parse_args()
    if not ZOLA.exists():
        print(f"error: {ZOLA} missing — run scripts/perf/build.sh first", file=sys.stderr)
        return 2
    if getattr(args, "name", None) is None:
        args.name = args.cmd
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
