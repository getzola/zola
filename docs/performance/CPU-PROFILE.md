# CPU profile (M7)

## Tooling

macOS on Apple Silicon: no `perf`, no `heaptrack`. `xctrace` and `dtrace` exist;
`samply` (0.13.1) was installed because it produces a portable Firefox-profiler
JSON that can be analysed offline.

```bash
scripts/perf/build.sh profiling      # release codegen + full debug symbols, no strip
samply record --save-only --unstable-presymbolicate \
  -o benchmarks/profiles/<name>.json -r 999 -- \
  target/profiling/zola build --force -o /tmp/out
scripts/perf/analyze_profile.py  benchmarks/profiles/<name>.json --top 30
scripts/perf/profile_summary.py  benchmarks/profiles/*.json
```

`--unstable-presymbolicate` is required: without the `.syms.json` sidecar,
samply defers symbolication to its web UI and every frame in the saved profile
is a raw address. `analyze_profile.py` resolves frames against that sidecar's
`symbol_table` ranges (its `known_addresses` list carries placeholder names and
is deliberately ignored).

The `profiling` profile differs from `release` in exactly two settings:
`debug = "full"` and `strip = false`. Codegen (`opt-level=3`, `lto=true`,
`codegen-units=1`) is identical, so profiles reflect production code layout.
No timing number in this program comes from a `profiling` binary.

**Idle handling.** Rayon parks 12 worker threads for most of a short build; a
naive sample count is 40–60% `__psynch_cvwait`. `profile_summary.py` excludes
samples whose *leaf* frame is a parking primitive (`__psynch_cvwait`,
`__workq_kernreturn`, `kevent`, `swtch_pri`, `__semwait_signal`) and reports
percentages of the remaining *busy* samples. Percentages are inclusive and
overlap by design (highlighting is inside markdown rendering; lock waits are
inside both).

## Headline: two global locks serialize most of the work

| profile | busy samples | blocked on a mutex | which lock |
| ------- | ------------ | ------------------ | ---------- |
| `markdown-heavy-4000` | 61 447 | **80.6%** | giallo `PatternSet.regset` |
| `data-heavy-4000` | 21 753 | **72.4%** | `LoadData.result_cache` (76%) + giallo (24%) |
| `vomaste proxy` | 10 334 | **61.0%** | `LoadData.result_cache` (100%) |
| `template-heavy-4000` | 10 075 | **57.4%** | giallo |
| `many-taxonomies-4000` | 11 482 | **49.2%** | giallo |
| `mixed-realistic-8000` | 24 955 | **41.1%** | giallo |
| `dense-internal-links-4000` | 9 798 | **39.3%** | giallo |
| `mixed-realistic-4000` | 9 289 | **31.5%** | giallo |
| `simple-pages-1000` | 1 022 | 0.1% | — (no code blocks, no `load_data`) |

Attribution was done by walking every sample whose stack contains
`__psynch_mutexwait` and taking the deepest frame that belongs to Zola or its
crates (not a system library).

### Lock 1 — `LoadData.result_cache` (Zola's own code)

```
tera::vm::interpreter::VirtualMachine::interpret
  └ tera::functions::StoredFunction::new::{{closure}}
     └ <templates::functions::load_data::LoadData as tera::functions::Function<…>>::call
        └ std::sync::poison::mutex::Mutex<T>::lock      ← 6306 / 10334 busy samples (proxy)
           └ __psynch_mutexwait
```

`components/templates/src/functions/load_data.rs:317` takes the cache mutex and
**holds it across the file read (line 323), the HTTP request (326–383) and the
JSON/TOML/CSV/YAML parse (387–395)**, releasing it only when `call` returns. The
guard also covers the `cached_result.clone()` on the hit path (line 319).

Consequence: every `load_data()` call in the whole site is serialized against
every other one, including all disk I/O and all deserialization. On the
reference workload — 3629 pages each loading a *distinct* JSON view model, so
the cache never hits — this converts what should be embarrassingly parallel work
into a single-threaded queue. It is 61% of that build's busy CPU and 72% of
`data-heavy`'s.

The cache itself is also unbounded and keyed on a hash that includes
`get_file_time(path)`, so each call additionally does a `stat` before it can
even look up the key.

### Lock 2 — giallo `PatternSet.regset` (dependency)

```
markdown::render_content
  └ … giallo highlighting …
     └ giallo::grammars::pattern_set::PatternSet::find_at   ← 49 456 / 61 447 (markdown-heavy)
        └ std::sync::Mutex::lock → __psynch_mutexwait
```

`giallo-0.5.2/src/grammars/pattern_set.rs:21` wraps the Oniguruma `RegSet` in a
`Mutex` with the comment *"The RegSet is wrapped in a Mutex because
onig_regset_search writes to internal region"*. The grammar registry is shared
by all rayon workers, so every code-block match in the site contends on one lock
per language.

Effect on the scenarios: highlighting is 94% of busy CPU on `markdown-heavy`,
67% on `template-heavy`, 58% on `many-taxonomies`, 46–48% on the dense/mixed
scenarios — and the majority of that is waiting, not matching. This is why
`markdown-heavy` costs 1.18 ms/page while `simple-pages` costs 0.36 ms/page.

This lock lives in a dependency, so the fix is either upstream in giallo or a
per-thread registry on Zola's side; both need design work (see `HOTSPOTS.md`).

## Parallel efficiency (§13)

Whole-build thread sweeps (`bench.py threads`) turned out to be too noisy to
publish: on `markdown-heavy` the same configuration varied between 2.3 s and
7.1 s across runs, and the proxy sweep came out non-monotonic (t1 7.8 s, t2
2.5 s, t4 3.3 s, t8 1.8 s, t12 2.1 s). Lock-convoy dynamics plus output-tree I/O
make whole-build wall time a bad instrument here. Raw data is kept in
`benchmarks/results/9ec4407a-dirty/threads-*.json`, and no speedup claim is made
from it.

Instead the highlighting phase was isolated with `--timings`, which reports the
`render markdown` span alone. Same 2000-page site, same binary, the *only*
difference being whether `[markdown.highlighting]` is present:

| RAYON_NUM_THREADS | `render markdown` — highlighting ON | `render markdown` — highlighting OFF |
| ----------------- | ----------------------------------- | ------------------------------------ |
| 1 | 3.37 s | 181 ms |
| 2 | 4.25 s | 88 ms |
| 4 | 4.01 s | 63 ms |
| 12 | 3.96 s | 28 ms |

(each cell is the best of 3 runs; with highlighting on, the spread between runs
was under 25%.)

* **Without highlighting**: 181 ms → 28 ms = **6.5× speedup on 12 threads**,
  parallel efficiency 0.54. Markdown rendering parallelises well.
* **With highlighting**: 3.37 s → 3.96 s = **0.85×**. Adding 11 cores makes the
  phase *slower*. Speedup is ≤ 1 at every thread count.
* Highlighting costs 18× single-threaded (3.37 s vs 181 ms) and **142×** at 12
  threads (3.96 s vs 28 ms), because the serial part cannot shrink while
  everything around it does.

This is the cleanest available proof that PERF-002 is a serialization bug rather
than "highlighting is expensive": the same work parallelises fine when the
Oniguruma `RegSet` mutex is not in the path.

## Non-lock consumers

### Output writing

`simple-pages-1000` is the cleanest view because nothing else is happening:

| subsystem | % of busy CPU |
| --------- | ------------- |
| `Queue::write_output` | 57.2% |
| ↳ `std::fs::DirBuilder::create_dir_all` | 18.9% |
| `Site::load` (all of it) | 8.7% |
| tera render | 4.8% |

Self-time for the same profile family is dominated by syscalls: `__open`
(6.3%), `mkdir` (4.9%), `write` (2.2%), `close`, `__unlinkat`.
`create_dir_all` is called once per rendered output
(`components/site/src/queue.rs`), and every call walks and re-creates the whole
parent chain, so a 4-level-deep site issues 4 `mkdir` syscalls per page that all
fail with `EEXIST` after the first page in that directory. Measured: 1237 ms of
CPU in `mkdir` on `mixed-realistic-4000`, 18.9% of busy CPU on
`simple-pages-1000`.

### Cleaning the previous output

`clean_site_output_folder` is 4.0% of busy CPU on `mixed-realistic-4000` but
**663 ms — 36.3% of wall time** on the reference proxy, whose output tree is
73 MB / 6544 files. It is a single-threaded recursive delete that runs before
anything else, so it cannot overlap with any other phase.

### Render cache construction

`render::cache` is 5.0% of busy CPU on `mixed-4000` and 6.5% on `mixed-8000` —
modest in CPU terms, but it is 24.3% of *wall* time because it is entirely
sequential. Inside it, `<tera::value::Value as Serialize>::serialize` accounts
for 2.9–4.2% of busy CPU: this is the deep re-serialization of every page value
into its section's value and into each taxonomy term's value.

### What is *not* hot

* `pulldown_cmark` itself: 0.4% — markdown parsing is negligible next to
  highlighting.
* Link resolution, backlinks, taxonomy construction, sorting, path collision
  detection: all under 1% individually, consistent with the linear scaling
  measured in `SCALING.md`.
* `Page::from_file` parsing: 0.6–1.3%.

## Per-profile summaries

Regenerate with `scripts/perf/profile_summary.py benchmarks/profiles/*.json`.

| subsystem (inclusive, % busy) | simple-1k | mixed-4k | mixed-8k | dense-4k | taxo-4k | tpl-4k | md-4k | data-4k | proxy |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| markdown render | 0.1 | 37.6 | 48.3 | 46.8 | 58.2 | 66.9 | 94.1 | 20.5 | 0.1 |
| ↳ syntax highlighting | 4.2 | 37.7 | 47.9 | 46.1 | 58.2 | 66.9 | 93.7 | 20.6 | — |
| write output | 57.2 | 42.9 | 27.8 | 34.4 | 21.8 | 13.4 | 3.0 | 7.9 | 11.7 |
| ↳ create_dir_all | 18.9 | 13.3 | 7.3 | 10.2 | 7.8 | 4.8 | 1.0 | 1.4 | 4.5 |
| ↳ minify html | — | 5.5 | 5.1 | — | — | — | — | 4.4 | 1.7 |
| tera render | 4.8 | 8.3 | 10.1 | 4.1 | 8.4 | 4.1 | 0.6 | 65.4 | 73.8 |
| ↳ load_data | — | 1.8 | 1.9 | — | 0.0 | 1.6 | — | 63.6 | 71.6 |
| render cache build | 2.1 | 5.0 | 6.5 | 3.8 | 7.6 | 1.4 | 0.5 | 0.5 | 0.9 |
| site load (total) | 8.7 | 4.7 | 6.5 | 5.1 | 7.4 | 3.5 | 0.6 | 1.3 | 5.0 |
| **blocked on mutex** | 0.1 | **31.5** | **41.1** | **39.3** | **49.2** | **57.4** | **80.6** | **72.4** | **61.0** |
