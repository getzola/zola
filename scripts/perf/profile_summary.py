#!/usr/bin/env python3
"""Per-subsystem inclusive CPU breakdown across one or more samply profiles.

`analyze_profile.py` answers "which function burns CPU". This answers "which
subsystem is the build inside", which is what the hotspot ranking needs.

    ./profile_summary.py benchmarks/profiles/*.json --markdown

Method: every sample whose leaf frame is an idle primitive (a worker thread
parked in the kernel) is excluded, because rayon keeps 12 threads parked for
most of a short build and they would otherwise drown out the signal. Each
remaining sample is attributed to every subsystem whose marker appears anywhere
in its stack, so columns are inclusive and deliberately overlap (highlighting
sits inside markdown rendering, mutex waits sit inside highlighting).
"""

from __future__ import annotations

import argparse
import collections
import importlib.util
import sys
from pathlib import Path

_spec = importlib.util.spec_from_file_location("ap", Path(__file__).with_name("analyze_profile.py"))
ap = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(ap)

# Leaf frames that mean "this thread had nothing to do".
IDLE_LEAVES = {
    "__psynch_cvwait", "__workq_kernreturn", "kevent", "swtch_pri", "__semwait_signal",
}

SUBSYSTEMS: list[tuple[str, str]] = [
    ("load: site load (total)", "Site::load"),
    ("load: page parse", "Page::from_file"),
    ("load: markdown render", "markdown::render_content"),
    ("load:   syntax highlighting", "giallo"),
    ("load:   oniguruma regex", "onig"),
    ("load: render cache build", "render::cache"),
    ("load: tera value serialize", "Serialize>::serialize"),
    ("build: total", "Site::build"),
    ("build: clean output dir", "clean_site_output_folder"),
    ("build: write output", "Queue::write_output"),
    ("build:   create_dir_all", "create_dir_all"),
    ("build:   minify html", "minify"),
    ("build: tera render", "tera::"),
    ("build: load_data", "load_data"),
    ("build: copy assets", "copy_assets"),
    ("build: copy static dir", "copy_directory"),
    ("blocked: mutex wait", "psynch_mutexwait"),
    ("blocked: malloc lock", "_malloc_lock"),
]


def summarize(path: Path) -> tuple[dict[str, int], int, int]:
    profile = ap.load_profile(path)
    symbols = ap.load_symbols(path)
    counts: collections.Counter = collections.Counter()
    total = idle = 0

    for thread in profile.get("threads", []):
        samples = thread.get("samples", {})
        n = samples.get("length", 0)
        if not n:
            continue
        names = ap.func_names(profile, thread, symbols)
        frame_func = thread["frameTable"]["func"]
        prefix = thread["stackTable"]["prefix"]
        frame = thread["stackTable"]["frame"]
        joined_cache: dict[int, str] = {}
        leaf_cache: dict[int, str] = {}

        for i in range(n):
            node = samples["stack"][i]
            if node is None:
                continue
            total += 1
            if node not in joined_cache:
                chain = []
                cursor = node
                while cursor is not None:
                    chain.append(names[frame_func[frame[cursor]]])
                    cursor = prefix[cursor]
                joined_cache[node] = "\n".join(chain)
                leaf_cache[node] = chain[0].split(" [")[0]
            if leaf_cache[node] in IDLE_LEAVES:
                idle += 1
                continue
            stack = joined_cache[node]
            for label, marker in SUBSYSTEMS:
                if marker in stack:
                    counts[label] += 1
    return counts, total, idle


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("profiles", nargs="+", type=Path)
    parser.add_argument("--markdown", action="store_true")
    args = parser.parse_args()

    for path in args.profiles:
        counts, total, idle = summarize(path)
        busy = total - idle
        if busy <= 0:
            print(f"{path.name}: no busy samples", file=sys.stderr)
            continue
        print(f"\n### {path.stem}\n")
        print(f"samples: {total} total, {idle} idle-parked, {busy} busy\n")
        if args.markdown:
            print("| subsystem | busy samples | % of busy CPU |")
            print("| --------- | ------------ | ------------- |")
        for label, _marker in SUBSYSTEMS:
            value = counts.get(label, 0)
            if not value:
                continue
            if args.markdown:
                print(f"| {label} | {value} | {value / busy * 100:.1f}% |")
            else:
                print(f"{value:8d}  {value / busy * 100:6.1f}%  {label}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
