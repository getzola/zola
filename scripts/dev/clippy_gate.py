#!/usr/bin/env python3
"""Clippy as a ratchet: existing warnings are tolerated, new ones are not.

`-D warnings` is the right gate for a clean workspace. This one is not clean —
there are pre-existing warnings, most of them in tests — and failing every
contributor's build on debt they did not create is how a lint gate gets
disabled. So instead of a pass/fail on zero, this compares the count per lint
against a checked-in baseline:

* a lint that appears for the first time      -> fail
* a lint whose count went up                  -> fail
* a lint whose count went down                -> fail, asking you to bank the win
* anything else                               -> pass

Banking a reduction keeps the number honest and stops the debt from silently
growing back.

Usage:
    scripts/dev/clippy_gate.py            # check against the baseline
    scripts/dev/clippy_gate.py --update   # rewrite the baseline from reality
    scripts/dev/clippy_gate.py --list     # show the current counts

A toolchain upgrade that introduces new lints will fail here. That is
deliberate: look at what it found, then `--update` and commit the new baseline
in its own commit.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(os.environ.get("ZOLA_DEV_ROOT", Path(__file__).resolve().parents[2]))
BASELINE = ROOT / "scripts" / "dev" / "clippy-baseline.json"


def run_clippy() -> tuple[Counter, list[str]]:
    """Return (counts by lint code, hard errors)."""
    env = dict(os.environ)
    # See scripts/perf/build.sh: a global cargo config can inject rustflags that
    # make the workspace fail to compile under clippy.
    env.setdefault("RUSTFLAGS", "")
    env["RUSTFLAGS"] = env.get("ZOLA_DEV_RUSTFLAGS", "")

    proc = subprocess.run(
        [
            "cargo",
            "clippy",
            "--workspace",
            "--all-targets",
            "--message-format=json",
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
        env=env,
    )

    counts: Counter = Counter()
    errors: list[str] = []
    for line in proc.stdout.splitlines():
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue
        if msg.get("reason") != "compiler-message":
            continue
        diag = msg.get("message", {})
        level = diag.get("level")
        if level not in ("warning", "error"):
            continue
        code = (diag.get("code") or {}).get("code") or f"<un-coded {level}>"
        if level == "error":
            errors.append(diag.get("rendered", code).strip())
        else:
            counts[code] += 1

    if proc.returncode != 0 and not errors:
        errors.append(
            "cargo clippy exited with "
            f"{proc.returncode} and no diagnostic:\n{proc.stderr.strip()[-2000:]}"
        )
    return counts, errors


def load_baseline() -> dict[str, int]:
    if not BASELINE.is_file():
        return {}
    return json.loads(BASELINE.read_text(encoding="utf-8"))["lints"]


def save(counts: Counter) -> None:
    BASELINE.write_text(
        json.dumps(
            {
                "comment": (
                    "Pre-existing clippy warnings, per lint. Checked by "
                    "scripts/dev/clippy_gate.py; regenerate with --update. "
                    "The only acceptable direction for these numbers is down."
                ),
                "total": sum(counts.values()),
                "lints": dict(sorted(counts.items())),
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--update", action="store_true", help="rewrite the baseline")
    ap.add_argument("--list", action="store_true", help="print current counts")
    args = ap.parse_args(argv[1:])

    counts, errors = run_clippy()

    if errors:
        print("clippy reported errors, not just warnings:\n")
        for e in errors[:5]:
            print(e)
        return 1

    if args.list:
        for code, n in sorted(counts.items(), key=lambda kv: (-kv[1], kv[0])):
            print(f"{n:4d}  {code}")
        print(f"{sum(counts.values()):4d}  total")
        return 0

    if args.update:
        save(counts)
        print(
            f"baseline written: {sum(counts.values())} warnings across "
            f"{len(counts)} lints ({BASELINE.relative_to(ROOT)})"
        )
        return 0

    baseline = load_baseline()
    if not baseline and not BASELINE.is_file():
        print(
            f"no clippy baseline at {BASELINE.relative_to(ROOT)}.\n"
            "    Create one:\n"
            "        scripts/dev/clippy_gate.py --update"
        )
        return 1

    grew, shrank, new = [], [], []
    for code, n in sorted(counts.items()):
        was = baseline.get(code)
        if was is None:
            new.append((code, n))
        elif n > was:
            grew.append((code, was, n))
        elif n < was:
            shrank.append((code, was, n))
    gone = sorted(set(baseline) - set(counts))

    if not (grew or new or shrank or gone):
        print(f"clippy OK ({sum(counts.values())} known warnings, none new).")
        return 0

    print("clippy check FAILED:\n")
    for code, n in new:
        print(f"  - new lint `{code}` ({n} occurrence(s)).")
        print(f"        cargo clippy --workspace --all-targets 2>&1 | grep -A5 '{code}'")
    for code, was, n in grew:
        print(f"  - `{code}` went from {was} to {n}. Fix the ones you added.")
    for code, was, n in shrank:
        print(f"  - `{code}` went from {was} down to {n}. Bank it:")
        print("        scripts/dev/clippy_gate.py --update")
    for code in gone:
        print(f"  - `{code}` is gone entirely. Bank it:")
        print("        scripts/dev/clippy_gate.py --update")
    return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
