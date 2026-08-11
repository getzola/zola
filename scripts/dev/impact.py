#!/usr/bin/env python3
"""Classify a change: components touched, risk, required gates, documentation impact.

This does not judge whether a change is correct. It answers three questions that
are cheap to get wrong when you are deep in a diff:

  1. which parts of the workspace does this touch?
  2. how much validation does a change of this shape need?
  3. which documents describe the behaviour I just changed?

Documentation impact is a *reminder*, derived from paths only. It never edits a
document and it cannot tell whether the behaviour actually changed — a refactor
under `components/config/` may need no documentation at all. Say so in the
commit message rather than silently ignoring the reminder.

Usage:
    scripts/dev/impact.py                 # working tree vs HEAD
    scripts/dev/impact.py --base master   # branch vs a base ref
    scripts/dev/impact.py --json
    scripts/dev/impact.py --strict        # exit 1 if a HIGH/CRITICAL change ships
                                          # with no test and no documentation edit
"""

from __future__ import annotations

import argparse
import fnmatch
import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(os.environ.get("ZOLA_DEV_ROOT", Path(__file__).resolve().parents[2]))

RISK_ORDER = ["LOW", "MEDIUM", "HIGH", "CRITICAL"]

# First matching pattern wins. Everything unmatched under components/ or src/ is
# MEDIUM; everything else is LOW.
RISK_RULES: list[tuple[str, str, str]] = [
    ("components/site/src/queue.rs", "CRITICAL", "decides what is written where"),
    ("components/site/src/lib.rs", "CRITICAL", "build pipeline order and content discovery"),
    ("components/render/src/cache.rs", "CRITICAL", "everything templates can see"),
    ("components/utils/src/fs.rs", "CRITICAL", "output cleaning and static copy"),
    ("src/cli.rs", "HIGH", "CLI surface; also feeds generated man pages and completions"),
    ("components/config/**", "HIGH", "configuration surface"),
    ("components/content/**", "HIGH", "page/section semantics, permalinks, taxonomies"),
    ("components/markdown/**", "HIGH", "rendered output"),
    ("components/render/**", "HIGH", "rendered output"),
    ("components/templates/src/functions/**", "HIGH", "template API"),
    ("components/templates/src/filters/**", "HIGH", "template API"),
    ("components/templates/src/builtins/**", "HIGH", "default output for feeds, sitemap, 404"),
    ("components/site/src/link_checking.rs", "HIGH", "build failure conditions"),
    ("components/site/src/feeds.rs", "HIGH", "published output"),
    ("components/site/src/sitemap.rs", "HIGH", "published output"),
    ("components/site/src/md_render.rs", "HIGH", "content templating"),
    ("components/search/**", "HIGH", "published search index"),
    ("components/imageproc/**", "HIGH", "published assets and their URLs"),
    ("components/link_checker/**", "HIGH", "build failure conditions"),
    ("src/cmd/serve.rs", "HIGH", "serve-mode rebuild selection"),
    ("src/fs_utils.rs", "HIGH", "serve-mode change classification"),
    ("Cargo.toml", "MEDIUM", "dependency or profile change"),
    ("components/*/Cargo.toml", "MEDIUM", "dependency change"),
]

# Path pattern -> documents that describe the behaviour behind it.
DOC_RULES: list[tuple[str, list[str]]] = [
    ("src/cli.rs", ["docs/content/documentation/getting-started/cli-usage.md"]),
    ("src/cmd/**", ["docs/content/documentation/getting-started/cli-usage.md"]),
    ("components/config/**", ["docs/content/documentation/getting-started/configuration.md"]),
    (
        "components/templates/src/functions/**",
        ["docs/content/documentation/templates/overview.md"],
    ),
    (
        "components/templates/src/filters/**",
        ["docs/content/documentation/templates/overview.md"],
    ),
    (
        "components/templates/src/builtins/**",
        [
            "docs/content/documentation/templates/404.md",
            "docs/content/documentation/templates/feeds/index.md",
            "docs/content/documentation/templates/sitemap.md",
            "docs/content/documentation/templates/robots.md",
        ],
    ),
    (
        "components/markdown/**",
        [
            "docs/content/documentation/content/linking.md",
            "docs/content/documentation/content/syntax-highlighting.md",
            "docs/content/documentation/content/table-of-contents.md",
        ],
    ),
    (
        "components/content/**",
        [
            "docs/content/documentation/content/page.md",
            "docs/content/documentation/content/section.md",
            "docs/content/documentation/content/taxonomies.md",
            "docs/content/documentation/content/multilingual.md",
        ],
    ),
    ("components/search/**", ["docs/content/documentation/content/search.md"]),
    (
        "components/imageproc/**",
        ["docs/content/documentation/content/image-processing/index.md"],
    ),
    ("components/site/src/sass.rs", ["docs/content/documentation/content/sass.md"]),
    ("components/site/src/feeds.rs", ["docs/content/documentation/templates/feeds/index.md"]),
    ("components/site/src/sitemap.rs", ["docs/content/documentation/templates/sitemap.md"]),
    ("components/render/src/pagination.rs", ["docs/content/documentation/templates/pagination.md"]),
    # Self-maintenance: the tooling documents itself.
    ("scripts/dev.sh", [".claude/README.md", "CLAUDE.md"]),
    ("scripts/dev/**", [".claude/README.md"]),
    ("scripts/perf/**", ["docs/performance/README.md"]),
]

# Touching one of these means the change is performance work and owes evidence.
PERF_PATHS = ["components/**", "src/**"]


def git(*args: str) -> str:
    return subprocess.run(
        ["git", *args], cwd=ROOT, capture_output=True, text=True, check=False
    ).stdout


def changed_files(base: str | None) -> list[str]:
    if base:
        out = git("diff", "--name-only", f"{base}...HEAD")
        out += git("diff", "--name-only", "HEAD")
    else:
        out = git("diff", "--name-only", "HEAD")
        out += git("ls-files", "--others", "--exclude-standard")
    return sorted({line for line in out.splitlines() if line.strip()})


def matches(path: str, pattern: str) -> bool:
    if pattern.endswith("/**"):
        return path.startswith(pattern[:-2])
    return fnmatch.fnmatch(path, pattern)


def classify(files: list[str]) -> tuple[str, list[tuple[str, str, str]]]:
    reasons: list[tuple[str, str, str]] = []
    worst = "LOW"
    for path in files:
        risk, why = "LOW", ""
        for pattern, level, explanation in RISK_RULES:
            if matches(path, pattern):
                risk, why = level, explanation
                break
        else:
            if (path.startswith(("components/", "src/"))) and path.endswith(".rs"):
                risk, why = "MEDIUM", "internal implementation"
        if risk != "LOW":
            reasons.append((path, risk, why))
        if RISK_ORDER.index(risk) > RISK_ORDER.index(worst):
            worst = risk
    return worst, reasons


def components_of(files: list[str]) -> list[str]:
    found = set()
    for path in files:
        parts = path.split("/")
        if len(parts) > 2 and parts[0] == "components":
            found.add(parts[1])
        elif parts[0] == "src":
            found.add("zola (binary)")
    return sorted(found)


def doc_impact(files: list[str]) -> dict[str, list[str]]:
    """Document -> the changed paths that suggest it may need an update."""
    impact: dict[str, list[str]] = {}
    for path in files:
        for pattern, docs in DOC_RULES:
            if matches(path, pattern):
                for doc in docs:
                    impact.setdefault(doc, []).append(path)
    return {k: sorted(set(v)) for k, v in sorted(impact.items())}


def gates_for(risk: str, files: list[str]) -> list[str]:
    gates = ["scripts/dev.sh quality"]
    if risk in ("HIGH", "CRITICAL"):
        gates.append("a test that fails without the change")
    if risk == "CRITICAL":
        gates.append(
            "output equivalence: scripts/perf/run.sh equivalence <baseline-bin> <candidate-bin> <site>"
        )
    if any(f.startswith("scripts/") or f.startswith(".githooks/") for f in files):
        gates.append("scripts/dev.sh test-tooling")
    if any(f == "src/cli.rs" for f in files):
        gates.append("cargo build (regenerates man pages and shell completions)")
    return gates


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--base", help="compare against this ref instead of the working tree")
    ap.add_argument(
        "--files",
        nargs="+",
        help="classify these paths instead of asking git (used by the tooling tests)",
    )
    ap.add_argument("--json", action="store_true", help="machine-readable output")
    ap.add_argument(
        "--strict",
        action="store_true",
        help="exit 1 when a HIGH/CRITICAL change has neither a test nor a documentation edit",
    )
    args = ap.parse_args(argv[1:])

    files = sorted(set(args.files)) if args.files else changed_files(args.base)
    if not files:
        print("no changes.")
        return 0

    risk, reasons = classify(files)
    comps = components_of(files)
    docs = doc_impact(files)
    gates = gates_for(risk, files)

    touched_tests = [f for f in files if "/tests/" in f or f.endswith("_test.rs")]
    touched_docs = [f for f in files if f.startswith("docs/") or f == "CHANGELOG.md"]
    perf_files = [
        f
        for f in files
        if any(matches(f, p) for p in PERF_PATHS) and f.endswith(".rs")
    ]

    report = {
        "risk": risk,
        "files": files,
        "components": comps,
        "risk_reasons": [{"path": p, "risk": r, "why": w} for p, r, w in reasons],
        "documentation_impact": docs,
        "required_gates": gates,
        "tests_touched": touched_tests,
        "docs_touched": touched_docs,
        "performance_relevant": perf_files,
    }

    if args.json:
        json.dump(report, sys.stdout, indent=2)
        print()
    else:
        print(f"# Change impact ({len(files)} files)\n")
        print(f"Risk: **{risk}**")
        print(f"Components: {', '.join(comps) or 'none (tooling/docs only)'}\n")
        if reasons:
            print("## Why")
            for path, level, why in reasons:
                print(f"* `{path}` — {level}: {why}")
            print()
        print("## Gates this change needs")
        for gate in gates:
            print(f"* {gate}")
        print()
        if docs:
            print("## Documentation that may need updating")
            for doc, causes in docs.items():
                mark = "edited" if doc in touched_docs or doc in files else "NOT edited"
                print(f"* `{doc}` ({mark}) — because of {', '.join(f'`{c}`' for c in causes)}")
            print()
        if perf_files:
            print(
                "## Performance\n\n"
                "Production code changed. If this is a `PERF-*` item, it owes a "
                "before/after measurement in `docs/performance/OPTIMIZATIONS.md` "
                "and a result file under `benchmarks/results/`.\n"
            )

    if args.strict and risk in ("HIGH", "CRITICAL"):
        problems = []
        if not touched_tests:
            problems.append(
                f"a {risk} change with no test edit under any `tests/` directory"
            )
        if docs and not touched_docs:
            problems.append(
                f"a {risk} change touching documented behaviour with no edit under `docs/` "
                "and no CHANGELOG entry"
            )
        if problems:
            print("impact check FAILED:\n")
            for p in problems:
                print(f"  - {p}")
            print(
                "\n    If the change genuinely needs neither, say so explicitly in the "
                "commit message and re-run without --strict."
            )
            return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
