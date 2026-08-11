# Zola large-site performance program

Goal: make Zola scale substantially better with site size (thousands → tens of
thousands of pages) while preserving exact observable behaviour.

Motivating workload: a ~3.7k-page / ~1.6k-section site that keeps growing.

## Documents

| File | Contents |
| ---- | -------- |
| `ARCHITECTURE.md` | Build pipeline, types, collections, phases, shared state (M1) |
| `BASELINE.md` | Measured baseline, environment, methodology (M4) |
| `SCALING.md` | T(n) curves, growth exponents, ranked superlinear offenders (M5) |
| `CPU-PROFILE.md` | Profiler findings, top inclusive/exclusive consumers (M7) |
| `ALLOCATIONS.md` | Allocation behaviour and dominant sources (M8) |
| `HOTSPOTS.md` | Ranked, evidence-backed `PERF-*` backlog (M10) |
| `OPTIMIZATIONS.md` | Append-only log of completed `PERF-*` items with real numbers |
| `INCREMENTAL-BUILD-DESIGN.md` | Dependency-graph design for incremental builds |
| `FINAL-REPORT.md` | Summary against the program's success criteria |

## Harness

```bash
scripts/perf/run.sh build                 # build the benchmark binary (pinned profile)
scripts/perf/run.sh quick                 # ~1 min smoke run
scripts/perf/run.sh baseline              # full scenario × size matrix
scripts/perf/run.sh scaling benchmarks/results/<sha>/baseline-matrix.json --markdown
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
  `benchmarks/results/<git-sha>/*.json`.
* `scripts/perf/scaling.py` — growth-model fitting over result files.
* `scripts/perf/compare_output.py` — byte-for-byte output equivalence gate.

Generated sites and proxies live under `benchmarks/` and are gitignored; only
result JSON is committed.

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

## Rules this program follows

1. No production change without a reproducible measurement before and after.
2. Output equivalence is a hard gate: `compare_output.py` must report IDENTICAL.
3. `cargo fmt --check`, `cargo clippy --all-targets --all-features` and
   `cargo test --workspace` must pass for every optimization commit.
4. Never benchmark a debug build; never count cargo compilation as build time.
5. Scaling ratios (8k/4k, 4k/2k) are the primary regression signal, not
   absolute milliseconds.
