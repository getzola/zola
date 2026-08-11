#!/usr/bin/env python3
"""Byte-for-byte output equivalence gate for performance changes.

Builds the same site with two zola binaries (or compares two existing output
trees) and reports missing files, extra files and changed files, including the
first differing byte offset.

    # build with two binaries and compare
    ./compare_output.py --site benchmarks/sites/mixed-realistic-1000 \
        --baseline /tmp/zola-baseline --candidate target/release/zola

    # compare two trees that already exist
    ./compare_output.py --tree-a /tmp/out-a --tree-b /tmp/out-b

Exit code 0 means the trees are identical.

Nondeterministic fields
-----------------------
Zola output is deterministic for a fixed input, with one documented exception:
templates may call `now()`, which changes between runs. `--normalize-dates`
masks ISO-8601 timestamps and 4-digit years inside otherwise-differing files
before comparing, and reports how many files needed it.
"""

from __future__ import annotations

import argparse
import hashlib
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ISO = re.compile(rb"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:?\d{2})?")
YEAR = re.compile(rb"(?<![\d-])(19|20)\d{2}(?![\d-])")


def digest(path: Path) -> str:
    h = hashlib.blake2b(digest_size=16)
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def relative_files(root: Path) -> dict[str, Path]:
    out: dict[str, Path] = {}
    for dirpath, _dirs, names in os.walk(root):
        for name in names:
            p = Path(dirpath) / name
            out[str(p.relative_to(root))] = p
    return out


def first_difference(a: Path, b: Path) -> int | None:
    with a.open("rb") as fa, b.open("rb") as fb:
        offset = 0
        while True:
            ca, cb = fa.read(1 << 16), fb.read(1 << 16)
            if not ca and not cb:
                return None
            if ca != cb:
                for i, (x, y) in enumerate(zip(ca, cb)):
                    if x != y:
                        return offset + i
                return offset + min(len(ca), len(cb))
            offset += len(ca)


def normalized(path: Path) -> bytes:
    data = path.read_bytes()
    data = ISO.sub(b"<TIMESTAMP>", data)
    return YEAR.sub(b"<YEAR>", data)


def build(binary: Path, site: Path, out: Path) -> None:
    out.mkdir(parents=True, exist_ok=True)
    res = subprocess.run([str(binary), "build", "--force", "-o", str(out)],
                         cwd=site, capture_output=True, text=True)
    if res.returncode != 0:
        raise SystemExit(f"build failed with {binary}:\n{res.stdout}\n{res.stderr}")


def compare(a: Path, b: Path, normalize_dates: bool) -> int:
    files_a, files_b = relative_files(a), relative_files(b)
    keys_a, keys_b = set(files_a), set(files_b)

    missing = sorted(keys_a - keys_b)
    extra = sorted(keys_b - keys_a)
    changed: list[tuple[str, int | None]] = []
    normalized_away = 0

    for key in sorted(keys_a & keys_b):
        pa, pb = files_a[key], files_b[key]
        if pa.stat().st_size == pb.stat().st_size and digest(pa) == digest(pb):
            continue
        if normalize_dates and normalized(pa) == normalized(pb):
            normalized_away += 1
            continue
        changed.append((key, first_difference(pa, pb)))

    print(f"baseline : {a}  ({len(files_a)} files)")
    print(f"candidate: {b}  ({len(files_b)} files)")
    if normalized_away:
        print(f"note     : {normalized_away} file(s) matched only after date normalisation")

    if not missing and not extra and not changed:
        print("RESULT: IDENTICAL")
        return 0

    if missing:
        print(f"\nMISSING in candidate ({len(missing)}):")
        for k in missing[:50]:
            print(f"  - {k}")
        if len(missing) > 50:
            print(f"  … {len(missing) - 50} more")
    if extra:
        print(f"\nEXTRA in candidate ({len(extra)}):")
        for k in extra[:50]:
            print(f"  + {k}")
        if len(extra) > 50:
            print(f"  … {len(extra) - 50} more")
    if changed:
        print(f"\nCHANGED ({len(changed)}):")
        for k, off in changed[:50]:
            print(f"  ~ {k} (first differing byte: {off})")
        if len(changed) > 50:
            print(f"  … {len(changed) - 50} more")
    print("\nRESULT: DIFFERENT")
    return 1


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--site", type=Path)
    ap.add_argument("--baseline", type=Path, help="baseline zola binary")
    ap.add_argument("--candidate", type=Path, help="candidate zola binary")
    ap.add_argument("--tree-a", type=Path)
    ap.add_argument("--tree-b", type=Path)
    ap.add_argument("--normalize-dates", action="store_true")
    ap.add_argument("--keep", action="store_true", help="keep the built trees")
    args = ap.parse_args()

    if args.tree_a and args.tree_b:
        return compare(args.tree_a, args.tree_b, args.normalize_dates)

    if not (args.site and args.baseline and args.candidate):
        ap.error("either --tree-a/--tree-b, or --site with --baseline and --candidate")

    tmp = Path(tempfile.mkdtemp(prefix="zolaperf-equiv-"))
    try:
        a, b = tmp / "baseline", tmp / "candidate"
        build(args.baseline.resolve(), args.site.resolve(), a)
        build(args.candidate.resolve(), args.site.resolve(), b)
        rc = compare(a, b, args.normalize_dates)
        if args.keep:
            print(f"\ntrees kept in {tmp}")
        return rc
    finally:
        if not args.keep:
            shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
