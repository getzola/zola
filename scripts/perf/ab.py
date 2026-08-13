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

The `--json` artifact
---------------------
Measurements live under `sites.<site name>` exactly as before: the raw per-round
samples in `a`/`b`, and a `wall_s`/`cpu_s`/`peak_rss_mb` block each holding
`a_median`, `a_spread`, `b_median`, `b_spread`, `deltas_pct`, `median_pct` and
`unanimous`. Alongside them the artifact describes where the numbers came from,
so a committed result does not need its provenance written down by hand
somewhere else:

* `binaries.a` / `binaries.b` — path, resolved path, size, mtime, SHA-256 and
  `--version` of each measured binary. The hash is the identifying fact: a
  binary in /tmp may have been built from a commit that is no longer checked
  out, so `machine.git_commit` can disagree with what actually ran, and a reader
  can only notice that if both are recorded.
* `machine` — the block bench.py writes, with the same field names, plus an
  explicit `git_dirty`. `build_profile` and `zola_version` are absent: ab.py is
  handed two arbitrary binaries and cannot know how either was built.
* `timestamp_utc` — when the comparison ran.

Everything here is collected before the first build, and anything that cannot be
determined (no git, a binary that will not report its version) is recorded as
null rather than guessed.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
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

REPO = Path(__file__).resolve().parents[2]


def sh(cmd: list[str], cwd: Path | None = None) -> str | None:
    """Stripped stdout, or None if the command is missing or fails.

    None means "not determined" everywhere in the provenance block; an empty
    string means the command ran and said nothing.
    """
    try:
        res = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    except OSError:
        return None
    return res.stdout.strip() if res.returncode == 0 else None


def git(args: list[str]) -> str | None:
    return sh(["git", *args], cwd=REPO)


def sysctl(key: str) -> str | None:
    return sh(["sysctl", "-n", key]) or None


def hardware_slug() -> str | None:
    """bench.py's slug, so an A/B result files under the same machine name."""
    here = str(Path(__file__).resolve().parent)
    if here not in sys.path:
        sys.path.insert(0, here)
    try:
        import bench  # noqa: PLC0415  (optional, imported only for its slug)

        return bench.hardware_slug()
    except Exception:
        return None


def git_dirty() -> bool | None:
    status = git(["status", "--porcelain"])
    return None if status is None else bool(status)


def commit_slug() -> str | None:
    """bench.py's `<commit time UTC>-<short sha>[-dirty]`."""
    sha = git(["rev-parse", "--short", "HEAD"])
    if not sha:
        return None
    epoch = git(["show", "-s", "--format=%ct", "HEAD"])
    stamp = time.strftime("%Y%m%dT%H%M%SZ", time.gmtime(int(epoch))) if epoch and epoch.isdigit() \
        else "unknown"
    return f"{stamp}-{sha}-dirty" if git_dirty() else f"{stamp}-{sha}"


def git_commit() -> str | None:
    sha = git(["rev-parse", "--short", "HEAD"])
    if not sha:
        return None
    return f"{sha}-dirty" if git_dirty() else sha


def machine_metadata() -> dict:
    """Machine facts under bench.py's field names — one vocabulary, not two.

    bench.py's `build_profile` and `zola_version` are deliberately missing:
    those describe *the* binary it built, and ab.py measures two binaries it did
    not build. Their versions are in the per-binary block instead.
    """
    return {
        "hardware_slug": hardware_slug(),
        "hw_model": sysctl("hw.model"),
        "commit_slug": commit_slug(),
        "commit_date": git(["show", "-s", "--format=%cI", "HEAD"]),
        "os": platform.platform(),
        "arch": platform.machine(),
        "cpu": sysctl("machdep.cpu.brand_string") or platform.processor() or None,
        "physical_cores": sysctl("hw.physicalcpu"),
        "logical_cores": sysctl("hw.logicalcpu") or str(os.cpu_count() or ""),
        "memory_bytes": sysctl("hw.memsize"),
        "rustc": sh(["rustc", "--version"]),
        "cargo": sh(["cargo", "--version"]),
        "git_commit": git_commit(),
        "git_branch": git(["rev-parse", "--abbrev-ref", "HEAD"]),
        # bench.py encodes this in the "-dirty" suffix; spelled out here because
        # a reader of an A/B artifact is asking exactly this question.
        "git_dirty": git_dirty(),
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
    }


def sha256(path: Path) -> str | None:
    h = hashlib.sha256()
    try:
        with path.open("rb") as fh:
            for chunk in iter(lambda: fh.read(1 << 20), b""):
                h.update(chunk)
    except OSError:
        return None
    return h.hexdigest()


def binary_metadata(binary: Path) -> dict:
    """What was actually run, identified by content rather than by path.

    Paths are reused (/tmp/zola-BASE is rebuilt for every comparison) and the
    repo's commit says nothing about a binary built somewhere else, so the hash
    is the only durable identity here.
    """
    resolved = binary.resolve()
    try:
        st = resolved.stat()
    except OSError:
        st = None
    return {
        "path": str(binary),
        "resolved_path": str(resolved),
        "size_bytes": st.st_size if st else None,
        "mtime_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(st.st_mtime)) if st else None,
        "sha256": sha256(resolved),
        "zola_version": sh([str(resolved), "--version"]) or None,
    }


def fmt_sample(sample: dict) -> str:
    """One progress line, tolerant of fields the platform did not supply.

    `run_once` parses BSD `/usr/bin/time -l`. On GNU coreutils `-l` is not a
    valid flag, so cpu and rss come back None — and formatting None with
    `{:7.1f}` used to raise TypeError and kill the comparison mid-run. The
    statistics already handle None correctly; only this line did not.
    """
    parts = [f"{sample['wall_s']:6.2f} s wall"]
    parts.append(f"{sample['cpu_s']:7.1f} s cpu" if sample["cpu_s"] is not None else "  cpu n/a")
    parts.append(
        f"{sample['peak_rss_mb']:7.1f} MB rss" if sample["peak_rss_mb"] is not None else "  rss n/a"
    )
    return "  ".join(parts)


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
    if cpu is None and not getattr(run_once, "_warned", False):
        run_once._warned = True  # type: ignore[attr-defined]
        print(
            "  note: could not parse `/usr/bin/time -l` output — that flag is BSD-only, so on\n"
            "        GNU coreutils only wall time is available. CPU and peak RSS will be n/a,\n"
            "        and wall time on a machine doing other work is the weakest of the three.",
            file=sys.stderr,
            flush=True,
        )
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
    # Builds run with cwd set to the site, so a relative --a/--b would resolve
    # against the wrong directory and fail inside the timed command, where the
    # error reads "No such file or directory" buried in `time -l` output.
    args.a = args.a.resolve()
    args.b = args.b.resolve()

    # Before the first build: the binaries can be rebuilt and the tree can be
    # edited while a long comparison runs, and then the recorded provenance
    # would describe neither what ran nor when.
    provenance = {
        "binaries": {"a": binary_metadata(args.a), "b": binary_metadata(args.b)},
        "machine": machine_metadata(),
        "timestamp_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    }

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
                    print(f"  round {r + 1} {side}: {fmt_sample(sample)}", flush=True)

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

    for side in ("a", "b"):
        info = provenance["binaries"][side]
        digest = info["sha256"] or "sha256 unavailable"
        print(f"\n  {side.upper()} {info['resolved_path']}\n      {digest}"
              f"  {info['zola_version'] or 'version unknown'}")
    machine = provenance["machine"]
    dirty = machine["git_dirty"]
    state = "tree unknown" if dirty is None else ("dirty" if dirty else "clean")
    print(f"  repo {machine['git_commit'] or 'unknown commit'} ({state})"
          f" at {provenance['timestamp_utc']}")

    if args.json:
        args.json.write_text(
            json.dumps(
                {
                    "a": str(args.a),
                    "b": str(args.b),
                    "rounds": args.rounds,
                    **provenance,
                    "sites": results,
                },
                indent=2,
            )
        )
        print(f"\nwrote {args.json}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
