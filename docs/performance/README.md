# Zola large-site performance program

Goal: make Zola scale substantially better with site size (thousands → tens of
thousands of pages) while preserving exact observable behaviour.

Motivating workload: a ~3.7k-page / ~1.6k-section site that keeps growing.

**Where it stands** — see `STATUS.md` for the live count and `FINAL-REPORT.md`
for the answers with their evidence:

| workload | wall | peak RSS |
| -------- | ---- | -------- |
| `mixed-realistic-4000` | 2.10 s → 1.72 s | 1371 MB → 209 MB |
| `many-taxonomies-4000` | 2.21 s → 1.65 s | 1696 MB → 273 MB |
| `mixed-realistic-16000` | 8.67 s → 6.80 s | 5152 MB → 742 MB |
| `markdown-heavy-4000` | 6.24 s → 2.23 s | 514 MB → 522 MB |
| the reference site (3776 pages) | 28.9 s → 24.6 s | — |

Time is linear in page count in every scenario, before and after; what changed
is the constant, and above all the memory: ~330 KB per page → ~46 KB.

## Documents

| File | Contents |
| ---- | -------- |
| `ARCHITECTURE.md` | Build pipeline, types, collections, phases, shared state (M1) |
| `BASELINE.md` | Measured baseline, environment, methodology (M4) |
| `SCALING.md` | T(n) curves, growth exponents, ranked superlinear offenders (M5) |
| `CPU-PROFILE.md` | Profiler findings, top inclusive/exclusive consumers (M7) |
| `ALLOCATIONS.md` | Allocation behaviour and dominant sources (M8) |
| `REAL-SITE.md` | The reference site: migration to 0.23, 0.22-vs-0.23 numbers, and why its own templates dominate its build |
| `HOTSPOTS.md` | Ranked, evidence-backed `PERF-*` backlog (M10) |
| `OPTIMIZATIONS.md` | Append-only log of completed `PERF-*` items with real numbers |
| `STATUS.md` | Backlog state, **generated** from the two files above |
| `INCREMENTAL-BUILD-DESIGN.md` | Dependency-graph design for incremental builds |
| `FINAL-REPORT.md` | Summary against the program's success criteria |
| `giallo-thread-local-regset.patch` | The highlighting fix, applied to the vendored copy in `vendor/giallo` and ready to send upstream |

## Harness

```bash
scripts/perf/run.sh build                 # build the benchmark binary (pinned profile)
scripts/perf/run.sh quick                 # ~1 min smoke run
scripts/perf/run.sh baseline              # full scenario × size matrix
scripts/perf/run.sh scaling benchmarks/results/<hardware>/<commit-utc>-<sha>/baseline-matrix.json --markdown
scripts/perf/run.sh threads --pages 4000  # parallel efficiency sweep
```

Components:

* `scripts/perf/build.sh` — builds `zola` with a pinned release profile and a
  neutralised environment (see BASELINE.md for why this is necessary).
* `scripts/perf/gen_site.py` — deterministic synthetic site generator. Same
  `(scenario, pages, seed)` always produces a byte-identical tree.
* `scripts/perf/make_proxy_site.py` — builds a *content-faithful proxy* of an
  external site (real content, substitute templates) for sites that cannot be
  built by the version under test.
* `scripts/perf/bench.py` — hyperfine-driven runner; writes
  `benchmarks/results/<hardware>/<commit-utc>-<sha>[-dirty]/<label>.json`.

  The path is a pure function of (machine, commit, label):

  * **grouped by hardware** — `m4-pro-12c-24gb-mac16-8` — because numbers from
    different machines must not be compared. Override with `ZOLA_PERF_HW=…`.
  * **sorted by commit date** — the directory starts with the commit's UTC
    timestamp, so a plain `ls` inside a machine is chronological.
  * **named by commit** — the short sha is in the directory, and `-dirty` is
    appended when the working tree was modified, because then the sha does not
    identify what was measured.
  * **idempotent** — re-running the same benchmark overwrites its own file. A
    run spoiled by other programs competing for the machine is corrected by
    closing them and running again, not by pruning stale files.
* `scripts/perf/scaling.py` — growth-model fitting over result files.
* `scripts/perf/ab.py` — interleaved A/B of two binaries. The two alternate
  inside every round and swap order between rounds, and the verdict is the
  **paired** per-round delta plus whether every round agrees on its sign — not
  the difference of two medians, which on a laptop measures thermal state and
  background load as much as it measures code. It reports CPU time alongside
  wall, because a build that writes gigabytes stalls on the filesystem in ways
  that move wall time several seconds in both directions without saying anything
  about the change under test.
* `scripts/perf/compare_output.py` — byte-for-byte output equivalence gate.

Generated sites and proxies live under `benchmarks/` and are gitignored; only
result JSON is committed.

The backlog itself is validated: `scripts/dev.sh perf-index check` fails if a
`PERF-*` id is referenced but never defined, or if a completed item does not
cite the commit that delivered it. `scripts/dev.sh generate` rewrites
`STATUS.md`. Both also run in CI.

The rules this program follows are stated below; the working procedure — what a
finished item owes, and when to stop — is in `.claude/workflows/performance.md`.

## Scenarios

| Scenario | Stresses |
| -------- | -------- |
| `simple-pages` | per-page floor cost |
| `dense-internal-links` | `@/` resolution, backlinks, anchor checking |
| `many-taxonomies` | taxonomy construction, term page lists |
| `deep-sections` | ancestors, subsections, per-section sorting |
| `template-heavy` | Tera render cost, `get_section`, breadcrumbs |
| `markdown-heavy` | pulldown-cmark, highlighting, headings, TOC |
| `data-heavy` | per-page `load_data()` of JSON view models |
| `mixed-realistic` | combination calibrated against the reference site |

## Reading the backlog

`OPTIMIZATIONS.md` is append-only and contains the **rejected** experiments as
well as the accepted ones — caching created directories (two variants), and both
ways of making the output clean cheaper. They are there because each looked
obvious, was measured, and did not pay; re-proposing one costs a day.

## Rules this program follows

1. No production change without a reproducible measurement before and after.
2. Output equivalence is a hard gate: `compare_output.py` must report IDENTICAL.
3. `cargo fmt --check`, `cargo clippy --all-targets --all-features` and
   `cargo test --workspace` must pass for every optimization commit.
4. Never benchmark a debug build; never count cargo compilation as build time.
5. Scaling ratios (8k/4k, 4k/2k) are the primary regression signal, not
   absolute milliseconds.
6. Comparisons are interleaved and judged on the paired per-round delta. A
   sequential "N runs of A, then N runs of B" comparison produced a −69% result
   early in this program that was entirely thermal drift.
7. When an effect is smaller than the noise floor of a whole-build measurement,
   measure the phase or the profile symbol instead — and say which was measured.
   A change that cannot be resolved is reported as unresolved, not as a win.
