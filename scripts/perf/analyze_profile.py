#!/usr/bin/env python3
"""Summarise a samply/Firefox-profiler JSON profile without opening the UI.

Computes, per function, self (exclusive) and total (inclusive) sample counts
across all threads, plus a caller breakdown for the top entries.

    samply record --save-only -o profile.json -- ./target/profiling/zola build --force -o /tmp/out
    ./analyze_profile.py profile.json --top 40
    ./analyze_profile.py profile.json --filter zola --callers markdown

Sample counts are converted to milliseconds using each thread's sampling
interval, so numbers are comparable between runs of different length. Because
Zola renders in parallel, summed CPU time across threads exceeds wall time.
"""

from __future__ import annotations

import argparse
import bisect
import collections
import gzip
import json
import sys
from pathlib import Path


def load_profile(path: Path) -> dict:
    raw = path.read_bytes()
    if raw[:2] == b"\x1f\x8b":
        raw = gzip.decompress(raw)
    return json.loads(raw)


def string_array(profile: dict, thread: dict) -> list[str]:
    for holder, key in ((thread, "stringArray"), (thread, "stringTable"),
                        (profile.get("shared", {}), "stringArray")):
        value = holder.get(key)
        if isinstance(value, list):
            return value
        if isinstance(value, dict) and isinstance(value.get("_array"), list):
            return value["_array"]
    raise SystemExit("could not locate the profile's string table")


def load_symbols(profile_path: Path) -> dict[str, list[tuple[int, int, str]]]:
    """Load samply's `--unstable-presymbolicate` sidecar, if present.

    Returns {debug_name: [(rva, size, symbol), ...]} sorted by rva. Only
    `symbol_table` is used: the sidecar's `known_addresses` list carries
    placeholder names ("UNKNOWN", "start") that would otherwise shadow real
    symbols. Without the sidecar, frames stay as hex addresses, because samply
    normally symbolicates inside its own UI, which we deliberately do not use.
    """
    sidecar = profile_path.with_name(profile_path.stem + ".syms.json")
    if not sidecar.exists():
        return {}
    payload = json.loads(sidecar.read_text())
    strings = payload["string_table"]
    out: dict[str, list[tuple[int, int, str]]] = {}
    for lib in payload["data"]:
        entries = [
            (e["rva"], e.get("size") or 0, strings[e["symbol"]])
            for e in (lib.get("symbol_table") or [])
        ]
        entries.sort()
        out[lib["debug_name"]] = entries
    return out


def resolve(table: list[tuple[int, int, str]], rva: int) -> str | None:
    """Find the symbol whose [rva, rva+size) range contains `rva`."""
    if not table:
        return None
    index = bisect.bisect_right(table, (rva, float("inf"), "")) - 1
    if index < 0:
        return None
    start, size, name = table[index]
    if size and rva >= start + size:
        return None
    return name


def func_names(profile: dict, thread: dict, symbols: dict) -> list[str]:
    strings = string_array(profile, thread)
    func_table = thread["funcTable"]
    names = func_table["name"]
    resources = func_table.get("resource") or [-1] * len(names)
    resource_lib = thread.get("resourceTable", {}).get("lib") or []
    libs = profile.get("libs") or []

    resolved: list[str] = []
    for i, name_index in enumerate(names):
        raw = (strings[name_index] if isinstance(name_index, int) and name_index < len(strings)
               else "<unknown>")
        if symbols and raw.startswith("0x"):
            res = resources[i] if i < len(resources) else -1
            lib_name = None
            if isinstance(res, int) and 0 <= res < len(resource_lib):
                lib_index = resource_lib[res]
                if isinstance(lib_index, int) and 0 <= lib_index < len(libs):
                    lib_name = libs[lib_index].get("debugName")
            if lib_name:
                try:
                    rva = int(raw, 16)
                except ValueError:
                    rva = None
                symbol = resolve(symbols.get(lib_name, []), rva) if rva is not None else None
                if symbol:
                    raw = symbol if lib_name == "zola" else f"{symbol} [{lib_name}]"
                else:
                    raw = f"{raw} [{lib_name}]"
        resolved.append(raw)
    return resolved


def analyze(profile: dict, symbols: dict) -> tuple[dict, dict, dict, float]:
    self_ms: dict[str, float] = collections.defaultdict(float)
    total_ms: dict[str, float] = collections.defaultdict(float)
    callers: dict[str, collections.Counter] = collections.defaultdict(collections.Counter)
    wall_ms = 0.0

    for thread in profile.get("threads", []):
        samples = thread.get("samples", {})
        count = samples.get("length", len(samples.get("stack", []) or []))
        if not count:
            continue
        interval = thread.get("interval") or profile.get("meta", {}).get("interval") or 1.0

        names = func_names(profile, thread, symbols)
        frame_func = thread["frameTable"]["func"]
        stack_prefix = thread["stackTable"]["prefix"]
        stack_frame = thread["stackTable"]["frame"]
        stacks = samples["stack"]
        weights = samples.get("weight") or [1] * count

        # Cache the resolved function chain per stack node.
        chain_cache: dict[int, tuple[str, ...]] = {}

        def chain(node: int) -> tuple[str, ...]:
            cached = chain_cache.get(node)
            if cached is not None:
                return cached
            parts: list[str] = []
            cursor: int | None = node
            while cursor is not None:
                parts.append(names[frame_func[stack_frame[cursor]]])
                cursor = stack_prefix[cursor]
            result = tuple(reversed(parts))
            chain_cache[node] = result
            return result

        for i in range(count):
            node = stacks[i]
            if node is None:
                continue
            weight = weights[i] if isinstance(weights, list) else 1
            ms = interval * (weight or 1)
            wall_ms += ms
            frames = chain(node)
            if not frames:
                continue
            self_ms[frames[-1]] += ms
            for name in set(frames):
                total_ms[name] += ms
            for parent, child in zip(frames, frames[1:]):
                callers[child][parent] += 1

    return self_ms, total_ms, callers, wall_ms


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("profile", type=Path)
    ap.add_argument("--top", type=int, default=30)
    ap.add_argument("--filter", default=None, help="only show functions containing this substring")
    ap.add_argument("--callers", default=None, help="show callers of functions matching this")
    ap.add_argument("--markdown", action="store_true")
    args = ap.parse_args()

    profile = load_profile(args.profile)
    symbols = load_symbols(args.profile)
    if not symbols:
        print('warning: no .syms.json sidecar next to the profile; frames will be raw addresses.\n'
              '         re-record with: samply record --save-only --unstable-presymbolicate', file=sys.stderr)
    self_ms, total_ms, callers, wall_ms = analyze(profile, symbols)
    if not self_ms:
        print("no samples found in profile", file=sys.stderr)
        return 1

    def keep(name: str) -> bool:
        return args.filter is None or args.filter in name

    print(f"total CPU samples: {wall_ms:.0f} ms (summed over threads)\n")

    rows = sorted(((v, k) for k, v in self_ms.items() if keep(k)), reverse=True)[: args.top]
    header = f"{'self ms':>10} {'self %':>7} {'total ms':>10} {'total %':>8}  function"
    print("SELF (exclusive) TIME")
    print(header)
    for ms, name in rows:
        print(f"{ms:10.0f} {ms / wall_ms * 100:6.1f}% {total_ms[name]:10.0f} "
              f"{total_ms[name] / wall_ms * 100:7.1f}%  {name}")

    rows = sorted(((v, k) for k, v in total_ms.items() if keep(k)), reverse=True)[: args.top]
    print("\nTOTAL (inclusive) TIME")
    print(header)
    for ms, name in rows:
        print(f"{self_ms[name]:10.0f} {self_ms[name] / wall_ms * 100:6.1f}% {ms:10.0f} "
              f"{ms / wall_ms * 100:7.1f}%  {name}")

    if args.callers:
        print(f"\nCALLERS of functions matching '{args.callers}'")
        for name in sorted(k for k in callers if args.callers in k):
            top = callers[name].most_common(5)
            if not top:
                continue
            print(f"\n  {name}")
            for parent, n in top:
                print(f"      {n:8d} samples  <- {parent}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
