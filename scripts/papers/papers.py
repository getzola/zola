#!/usr/bin/env python3
"""Scaffolding, validation and indexing for `docs/papers/`.

The publication system's job is to stop four copies of a benchmark number
drifting apart. That is nearly all this script does:

* `new`      create a paper skeleton with a unique id and correct metadata
* `validate` the gate: identity, references, figures against their artifacts,
             derivatives against the paper, hygiene
* `index`    rewrite `docs/papers/INDEX.md` from metadata
* `json`     machine-readable dump of every paper's metadata

Source of truth:

* `metadata.toml`            identity and lifecycle of one paper
* `data/measurements.toml`   every figure the paper prints, and where it came from
* `benchmarks/results/**`    the artifacts those figures are checked against
* `paper.md`                 the only authored narrative; derivatives may not
                             contain a number it does not contain

Usage:
    scripts/papers/papers.py validate [--paper <dir>]
    scripts/papers/papers.py index [--check]
    scripts/papers/papers.py new --title "..." [--type performance-study] [--slug ...]
    scripts/papers/papers.py json
"""

from __future__ import annotations

import argparse
import datetime as _dt
import json
import re
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PAPERS = ROOT / "docs" / "papers"
HOTSPOTS = ROOT / "docs" / "performance" / "HOTSPOTS.md"
INDEX = PAPERS / "INDEX.md"

ID_RE = re.compile(r"^ZPERF-(\d{3})$")
DIR_RE = re.compile(r"^zperf-(\d{3})-([a-z0-9][a-z0-9-]*)$")

STATUSES = {"idea", "draft", "review", "published", "superseded"}
TYPES = {
    "performance-study",
    "bug-analysis",
    "architecture",
    "experiment",
    "negative-result",
    "methodology",
    "design-proposal",
    "postmortem",
}
REQUIRED_META = ["id", "title", "status", "type", "series", "date"]

# Anything that would leak the machine a paper was written on, or someone's
# private site, into a public document.
PRIVATE_PATTERNS = [
    (re.compile(r"/Users/[A-Za-z0-9._-]+"), "absolute macOS home path"),
    (re.compile(r"/home/[A-Za-z0-9._-]+"), "absolute Linux home path"),
    (re.compile(r"~/dev/"), "local development path"),
    (re.compile(r"[A-Za-z]:\\\\Users\\\\"), "absolute Windows home path"),
    (re.compile(r"/private/tmp/|/var/folders/"), "temporary directory path"),
]
PLACEHOLDER_RE = re.compile(r"\b(TODO|TBD|FIXME|XXX)\b|<FILL[^>]*>|\.\.\.\.")

# Numbers a derivative may carry without the paper repeating them: dates, the
# paper's own id, list markers, and years.
NUMBER_RE = re.compile(r"\d[\d,]*(?:\.\d+)?")


class Failures:
    def __init__(self) -> None:
        self.items: list[str] = []

    def add(self, where: str, msg: str) -> None:
        self.items.append(f"{where}: {msg}")

    def __bool__(self) -> bool:
        return bool(self.items)


def paper_dirs() -> list[Path]:
    if not PAPERS.is_dir():
        return []
    return sorted(p for p in PAPERS.iterdir() if p.is_dir() and DIR_RE.match(p.name))


def load_meta(d: Path) -> dict:
    with (d / "metadata.toml").open("rb") as fh:
        return tomllib.load(fh)


def known_perf_ids() -> set[str]:
    """Every `PERF-NNN` defined in the hotspot inventory's summary table."""
    if not HOTSPOTS.is_file():
        return set()
    return set(re.findall(r"\|\s*(PERF-\d{3})\s*\|", HOTSPOTS.read_text(encoding="utf-8")))


def dotted_get(obj, path: str):
    """Look up `a.b.c` in nested dicts. Keys may contain dots if unambiguous."""
    cur = obj
    for part in path.split("."):
        if isinstance(cur, list):
            cur = cur[int(part)]
        elif isinstance(cur, dict):
            if part not in cur:
                # Allow a remaining path that is itself a key, e.g. site names.
                rest = path[path.index(part) :]
                if rest in cur:
                    return cur[rest]
                raise KeyError(part)
            cur = cur[part]
        else:
            raise KeyError(part)
    return cur


# --------------------------------------------------------------------------- #
# validate
# --------------------------------------------------------------------------- #


def check_metadata(d: Path, meta: dict, seen: dict, f: Failures) -> None:
    where = f"{d.name}/metadata.toml"
    for key in REQUIRED_META:
        if not meta.get(key):
            f.add(where, f"missing required field `{key}`")
    pid = meta.get("id", "")
    m = ID_RE.match(pid)
    dm = DIR_RE.match(d.name)
    if not m:
        f.add(where, f"id `{pid}` is not of the form ZPERF-NNN")
    elif dm and m.group(1) != dm.group(1):
        f.add(where, f"id `{pid}` does not match directory number `{dm.group(1)}`")
    if pid in seen:
        f.add(where, f"duplicate id `{pid}`, also used by {seen[pid]}")
    else:
        seen[pid] = d.name

    status = meta.get("status")
    if status and status not in STATUSES:
        f.add(where, f"unknown status `{status}` (expected one of {sorted(STATUSES)})")

    types = meta.get("type")
    types = [types] if isinstance(types, str) else (types or [])
    for t in types:
        if t not in TYPES:
            f.add(where, f"unknown type `{t}` (expected one of {sorted(TYPES)})")

    known = known_perf_ids()
    for item in meta.get("perf_items", []) or []:
        if known and item not in known:
            f.add(where, f"references `{item}`, which HOTSPOTS.md does not define")

    date = meta.get("date")
    if date and not re.match(r"^\d{4}-\d{2}-\d{2}$", str(date)):
        f.add(where, f"date `{date}` is not YYYY-MM-DD")


def check_files(d: Path, f: Failures) -> str:
    """Required files exist. Returns paper.md's text (empty if missing)."""
    text = ""
    for rel in ("paper.md", "evidence.md", "metadata.toml"):
        if not (d / rel).is_file():
            f.add(d.name, f"missing `{rel}`")
    paper = d / "paper.md"
    if paper.is_file():
        text = paper.read_text(encoding="utf-8")
        # Intra-tree links and figure references must resolve.
        for target in re.findall(r"]\((?!https?:|mailto:|#)([^)]+)\)", text):
            target = target.split("#")[0].strip()
            if not target:
                continue
            resolved = (d / target) if not target.startswith("/") else (ROOT / target.lstrip("/"))
            if not resolved.exists():
                f.add(f"{d.name}/paper.md", f"link target does not exist: {target}")
    return text


def check_measurements(d: Path, paper_text: str, f: Failures) -> None:
    path = d / "data" / "measurements.toml"
    if not path.is_file():
        f.add(d.name, "missing `data/measurements.toml` — every printed figure is declared there")
        return
    with path.open("rb") as fh:
        doc = tomllib.load(fh)
    entries = doc.get("measurement", [])
    if not entries:
        f.add(f"{d.name}/data/measurements.toml", "declares no measurements")
    ids = set()
    for e in entries:
        mid = e.get("id", "?")
        where = f"{d.name}/measurements[{mid}]"
        if mid in ids:
            f.add(where, "duplicate measurement id")
        ids.add(mid)
        for key in ("id", "claim", "class", "text"):
            if not e.get(key):
                f.add(where, f"missing `{key}`")
        # A figure's *value* is bound by `source`/`json_path`; its *confidence*
        # is not, and that is a separate way a derivative goes wrong. A number
        # can be identical in all five places and still be dishonest in four of
        # them, because the caveat that makes it honest did not travel with it.
        rep = e.get("replication")
        if rep and rep not in {"reproduced", "did-not-reproduce", "single-session"}:
            f.add(where, f"unknown replication `{rep}`")

        cls = e.get("class")
        if cls and cls not in {
            "measured",
            "observed",
            "code-fact",
            "interpretation",
            "hypothesis",
            "proposal",
            "rejected",
        }:
            f.add(where, f"unknown class `{cls}`")

        text = e.get("text")
        if text and paper_text and text not in paper_text:
            f.add(where, f"declared figure `{text}` does not appear in paper.md")

        src = e.get("source")
        jpath = e.get("json_path")
        if src and jpath:
            artifact = ROOT / src
            if not artifact.is_file():
                f.add(where, f"source artifact missing: {src}")
                continue
            try:
                data = json.loads(artifact.read_text(encoding="utf-8"))
                actual = dotted_get(data, jpath)
            except Exception as exc:  # noqa: BLE001 - report, do not crash the gate
                f.add(where, f"cannot read `{jpath}` from {src}: {exc}")
                continue
            expected = e.get("value")
            if expected is None:
                f.add(where, "has a source but no `value` to check against it")
                continue
            # Absolute, in the unit the value is declared in. Relative
            # tolerances scaled by magnitude, which let a claim of -41.0 pass
            # against an artifact holding -33.3.
            tol = float(e.get("tolerance", 0.0))
            try:
                if abs(float(actual) - float(expected)) > tol:
                    f.add(
                        where,
                        f"declared value {expected} but {src}:{jpath} holds {actual}",
                    )
            except (TypeError, ValueError):
                if str(actual) != str(expected):
                    f.add(where, f"declared value {expected!r} but artifact holds {actual!r}")
        elif src and not jpath and not (ROOT / src).exists():
            f.add(where, f"source does not exist: {src}")


def unreproduced_figures(d: Path) -> list[tuple[str, str]]:
    """Figures a repeat measurement failed to confirm, as (text, id)."""
    path = d / "data" / "measurements.toml"
    if not path.is_file():
        return []
    with path.open("rb") as fh:
        doc = tomllib.load(fh)
    return [
        (e["text"], e.get("id", "?"))
        for e in doc.get("measurement", [])
        if e.get("replication") == "did-not-reproduce" and e.get("text")
    ]


def check_derivatives(d: Path, paper_text: str, f: Failures) -> None:
    """No number may appear in a derivative that the paper does not contain.

    And no figure that failed to reproduce may appear in one at all: a social
    post has no room for the caveat that makes such a number honest, so it
    travels stripped of the one thing a reader needs.
    """
    social = d / "social"
    if not social.is_dir():
        return
    unreproduced = unreproduced_figures(d)
    paper_numbers = set(NUMBER_RE.findall(paper_text))
    # Numbers structurally part of a post rather than a claim.
    allowed = paper_numbers | {"1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "0"}
    for post in sorted(social.glob("*.md")):
        head = post.read_text(encoding="utf-8")
        for text, mid in unreproduced:
            if text in head:
                f.add(
                    f"{d.name}/social/{post.name}",
                    f"carries `{text}` ({mid}), which a repeat measurement did not "
                    "reproduce — a derivative cannot carry the caveat that makes that "
                    "figure honest, so it must not carry the figure",
                )
        for num in NUMBER_RE.findall(head):
            if num not in allowed:
                f.add(
                    f"{d.name}/social/{post.name}",
                    f"number `{num}` does not appear in paper.md — derivatives may not "
                    "introduce figures",
                )


def check_hygiene(d: Path, f: Failures) -> None:
    for path in sorted(d.rglob("*")):
        if not path.is_file() or path.suffix not in {".md", ".toml", ".svg"}:
            continue
        rel = path.relative_to(PAPERS)
        text = path.read_text(encoding="utf-8", errors="replace")
        for pattern, label in PRIVATE_PATTERNS:
            hit = pattern.search(text)
            if hit:
                f.add(str(rel), f"{label}: {hit.group(0)}")
        hit = PLACEHOLDER_RE.search(text)
        if hit and path.suffix == ".md":
            f.add(str(rel), f"unfilled marker: {hit.group(0)}")


def cmd_validate(args: argparse.Namespace) -> int:
    f = Failures()
    dirs = paper_dirs()
    if args.paper:
        dirs = [p for p in dirs if p.name == args.paper]
        if not dirs:
            print(f"no such paper: {args.paper}", file=sys.stderr)
            return 2
    if not dirs:
        print("papers OK (no papers yet)")
        return 0

    seen: dict[str, str] = {}
    slugs: dict[str, str] = {}
    for d in dirs:
        meta = {}
        try:
            meta = load_meta(d)
        except FileNotFoundError:
            f.add(d.name, "missing metadata.toml")
        except tomllib.TOMLDecodeError as exc:
            f.add(d.name, f"metadata.toml does not parse: {exc}")
        if meta:
            check_metadata(d, meta, seen, f)
            dm = DIR_RE.match(d.name)
            slug = meta.get("canonical_slug") or (dm.group(2) if dm else d.name)
            if slug in slugs:
                f.add(d.name, f"duplicate slug `{slug}`, also used by {slugs[slug]}")
            slugs[slug] = d.name
        paper_text = check_files(d, f)
        check_measurements(d, paper_text, f)
        check_derivatives(d, paper_text, f)
        check_hygiene(d, f)

    if not args.paper:
        current = render_index()
        if not INDEX.is_file() or INDEX.read_text(encoding="utf-8") != current:
            f.add("INDEX.md", "out of date — run `scripts/dev.sh papers index`")

    if f:
        print(f"papers check FAILED ({len(f.items)} problem(s)):\n")
        for item in f.items:
            print(f"  - {item}")
        return 1
    print(f"papers OK ({len(dirs)} paper(s))")
    return 0


# --------------------------------------------------------------------------- #
# index
# --------------------------------------------------------------------------- #


def render_index() -> str:
    rows = []
    for d in paper_dirs():
        try:
            meta = load_meta(d)
        except Exception:  # noqa: BLE001
            continue
        types = meta.get("type")
        types = [types] if isinstance(types, str) else (types or [])
        perf = ", ".join(meta.get("perf_items", []) or []) or "—"
        rows.append(
            f"| [{meta.get('id', '?')}]({d.name}/paper.md) | {meta.get('title', '?')} | "
            f"{meta.get('status', '?')} | {meta.get('date', '?')} | {', '.join(types) or '—'} | {perf} |"
        )
    body = "\n".join(rows) if rows else "| — | no papers yet | — | — | — | — |"
    return (
        "# Paper index\n\n"
        "Generated by `scripts/dev.sh papers index` from each paper's "
        "`metadata.toml`. Do not edit by hand.\n\n"
        "| ID | Title | Status | Date | Type | PERF items |\n"
        "| -- | ----- | ------ | ---- | ---- | ---------- |\n"
        f"{body}\n\n"
        "Lifecycle, layout and the rules a paper must satisfy: "
        "[README.md](README.md), [METHODOLOGY.md](METHODOLOGY.md), "
        "[CHECKLIST.md](CHECKLIST.md). Prospective papers: [BACKLOG.md](BACKLOG.md).\n"
    )


def cmd_index(args: argparse.Namespace) -> int:
    content = render_index()
    if args.check:
        if not INDEX.is_file() or INDEX.read_text(encoding="utf-8") != content:
            print("INDEX.md is out of date; run `scripts/dev.sh papers index`", file=sys.stderr)
            return 1
        print("INDEX.md is current")
        return 0
    INDEX.write_text(content, encoding="utf-8")
    print(f"wrote {INDEX.relative_to(ROOT)}")
    return 0


# --------------------------------------------------------------------------- #
# new
# --------------------------------------------------------------------------- #

SKELETON_PAPER = """# {title}

> Status: **{status}**. {series}, {pid}. This work was done in a fork of Zola
> and is not affiliated with or endorsed by the upstream project.

## Abstract

## Context

## Problem

## Methodology

## Findings

## Results

## Negative results

## Correctness validation

## Limitations

## Future work

## Reproduction

## Evidence index

Every claim, its class and its source: [evidence.md](evidence.md).
"""

SKELETON_EVIDENCE = """# Evidence — {pid}

Claim classes are defined in [../METHODOLOGY.md](../METHODOLOGY.md).

## E-001

**Claim.**

**Class.**

**Source.**

**Method.**

**Caveat.**
"""

SKELETON_MEASUREMENTS = """# Every figure printed in paper.md is declared here.
#
# A figure taken from a committed benchmark artifact carries `source` and
# `json_path` so `scripts/dev.sh papers validate` can re-extract and compare it.
# A figure read off a terminal carries `source_note` instead and is classed
# `observed`.

[[measurement]]
id = "M-001"
claim = ""
class = "measured"
text = ""
"""


def next_id() -> tuple[str, int]:
    used = []
    for d in paper_dirs():
        m = DIR_RE.match(d.name)
        if m:
            used.append(int(m.group(1)))
    n = max(used) + 1 if used else 1
    return f"ZPERF-{n:03d}", n


def slugify(title: str) -> str:
    s = re.sub(r"[^a-z0-9]+", "-", title.lower()).strip("-")
    return re.sub(r"-{2,}", "-", s)


def cmd_new(args: argparse.Namespace) -> int:
    pid, n = next_id()
    slug = args.slug or slugify(args.title)
    d = PAPERS / f"zperf-{n:03d}-{slug}"
    if d.exists():
        print(f"already exists: {d.relative_to(ROOT)}", file=sys.stderr)
        return 1
    (d / "data").mkdir(parents=True)
    (d / "figures").mkdir()
    (d / "social").mkdir()

    try:
        commit = subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=True,
        ).stdout.strip()
    except Exception:  # noqa: BLE001
        commit = ""

    series = "Zola at Scale"
    date = _dt.date.today().isoformat()
    perf = "".join(f'"{p}", ' for p in (args.perf or []))
    meta = (
        f'id = "{pid}"\n'
        f'title = "{args.title}"\n'
        'subtitle = ""\n'
        f'status = "draft"\n'
        f'type = ["{args.type}"]\n'
        f'series = "{series}"\n'
        f'date = "{date}"\n'
        f'canonical_slug = "{slug}"\n'
        f'repository_commit = "{commit}"\n'
        f"perf_items = [{perf.rstrip(', ')}]\n"
        "topics = []\n"
        "keywords = []\n"
    )
    (d / "metadata.toml").write_text(meta, encoding="utf-8")
    (d / "paper.md").write_text(
        SKELETON_PAPER.format(title=args.title, status="draft", series=series, pid=pid),
        encoding="utf-8",
    )
    (d / "evidence.md").write_text(SKELETON_EVIDENCE.format(pid=pid), encoding="utf-8")
    (d / "data" / "measurements.toml").write_text(SKELETON_MEASUREMENTS, encoding="utf-8")
    for name in ("linkedin.md", "short.md", "thread.md"):
        (d / "social" / name).write_text(f"<!-- derived from ../paper.md; {pid} -->\n", encoding="utf-8")

    print(f"created {d.relative_to(ROOT)} ({pid})")
    print("next: read docs/papers/METHODOLOGY.md, then gather evidence before writing")
    return 0


def cmd_json(_: argparse.Namespace) -> int:
    out = []
    for d in paper_dirs():
        try:
            meta = load_meta(d)
        except Exception:  # noqa: BLE001
            continue
        meta["directory"] = d.name
        out.append(meta)
    print(json.dumps(out, indent=2, default=str))
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)

    v = sub.add_parser("validate", help="the publication gate")
    v.add_argument("--paper", help="validate one paper directory")
    v.set_defaults(func=cmd_validate)

    i = sub.add_parser("index", help="rewrite INDEX.md")
    i.add_argument("--check", action="store_true", help="fail if out of date instead of writing")
    i.set_defaults(func=cmd_index)

    n = sub.add_parser("new", help="scaffold a paper")
    n.add_argument("--title", required=True)
    n.add_argument("--type", default="performance-study", choices=sorted(TYPES))
    n.add_argument("--slug")
    n.add_argument("--perf", nargs="*", help="related PERF-* ids")
    n.set_defaults(func=cmd_new)

    j = sub.add_parser("json", help="machine-readable metadata dump")
    j.set_defaults(func=cmd_json)

    args = ap.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
