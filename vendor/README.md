# vendor/

Patched copies of dependencies, carried here only until the fix is released
upstream. Nothing in this directory is a workspace member; it is pulled in by
`[patch.crates-io]` in the root `Cargo.toml`.

## giallo

**Source**: `getzola/giallo@5e19db8` (the 0.5.2 tree), plus one patch:
`docs/performance/giallo-thread-local-regset.patch`.

**Why**: `PatternSet` held a single `Mutex<RegSet>` shared through `Arc` by
every rayon worker, because `onig_regset_search` mutates the regset's internal
region storage. Syntax highlighting therefore did not parallelise at all — the
markdown phase took 1.58 s on one thread and 1.69 s on twelve. The patch keeps
the pattern strings and compiles a `RegSet` per thread on first use.

**What it buys** (measured in `docs/performance/OPTIMIZATIONS.md`):

| | before | after |
| --- | ------ | ----- |
| `markdown-heavy-4000` wall | 4.23 s | 1.67 s (−61%) |
| markdown phase, 12 threads | 1.69 s | 0.27 s |
| peak RSS | 514 MB | 521 MB |

The alternative that needs no vendoring — a per-thread `Registry` clone inside
Zola — is just as fast and costs ~310 MB more, because it duplicates every
grammar instead of only the regsets a thread compiles. That is why this
directory exists.

**Contents**: `src/`, `builtin.zst`, `Cargo.toml`, `LICENSE`, `README.md`. The
manifest is upstream's with the benches, examples, tool binary,
dev-dependencies and profile overrides removed — a path dependency needs none
of them — and an empty `[workspace]` table added so cargo does not expect this
package to be a member of Zola's workspace.

## Removing it

When a giallo release contains the fix:

1. delete `vendor/giallo` and this file;
2. delete the `[patch.crates-io]` section from the root `Cargo.toml`;
3. bump `giallo` in `[workspace.dependencies]` to that release;
4. re-run the highlighting benchmarks
   (`scripts/perf/run.sh` and the `markdown-heavy` scenarios) to confirm the
   released version behaves like this copy.
