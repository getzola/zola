# Hotspot inventory (M10)

Ranked, evidence-backed backlog. Every row is supported by a measurement in
`BASELINE.md`, `SCALING.md`, `CPU-PROFILE.md` or `ALLOCATIONS.md` — nothing here
comes from reading code alone.

Priority classes: **P0** catastrophic scaling · **P1** major build cost ·
**P2** significant allocation/I/O cost · **P3** micro optimization.

## Milestone report (M1–M10)

### TOP 10 bottlenecks

| # | Bottleneck | Where | Evidence | Est. contribution |
| - | ---------- | ----- | -------- | ----------------- |
| 1 | Syntax highlighting serialized on one mutex | giallo `PatternSet` via `markdown::render_content` | 80.6% of busy CPU blocked (markdown-heavy); 0.85× speedup on 12 cores vs 6.5× with highlighting off | 60–90% of markdown time on any site with code blocks |
| 2 | `load_data` holds its cache mutex across I/O + parse | `load_data.rs:317` | 61.0% of busy CPU blocked (reference proxy), 72.4% (data-heavy) | ~60% of template time on data-driven sites |
| 3 | `RenderCache::build` sequential + deep re-serialization | `cache.rs:59` | 24.3% of wall (mixed-4k), 86% of live heap, 7.4–10 M allocations | 0.35–0.5 s at 4k pages, ~1 GB RSS |
| 4 | `clean_site_output_folder` single-threaded | `utils/fs.rs:216` | 663 ms = 36.3% of wall on the reference proxy | largest single item on the reference workload |
| 5 | `create_dir_all` per output file | `queue.rs` `write_output` | 18.9% busy CPU (simple-1k), 1237 ms of `mkdir` at 4k | 5–19% of build |
| 6 | Discovery + section parsing serial, double traversal | `site/lib.rs:190` | 232–357 ms = 13–27% of wall (1640 sections) | 0.23 s at 1.6k sections |
| 7 | Static tree copy serial, 2 `stat`s per file | `utils/fs.rs:107` | 138–170 ms = 9–10% of wall (989 files) | 0.17 s |
| 8 | Tera function registration clones config/permalinks/Tera, twice | `site/tpls.rs:5` | 1.1 M allocations, 132 MB, ~92 MB retained, 44–62 ms | 3% of wall, 8% of heap |
| 9 | HTML minification | `queue.rs` `write_output` | 1.0 s CPU / 4802 calls (mixed-4k), 5.5% busy CPU | 5% of build when enabled |
| 10 | Fixed startup: highlighting registry | `config` | 110–130 ms and ~170 MB before any page | dominates sub-500-page sites |

### TOP 10 algorithmic risks

None of these is currently superlinear; they are the places where that could
change, ranked by how close they are to the edge.

| # | Risk | Current shape | Why it is safe today | What would break it |
| - | ---- | ------------- | -------------------- | ------------------- |
| 1 | Page value duplication in the render cache | O(P × memberships) *materialised copies* | memberships are small on most sites | many taxonomies per page: measured +67 KB heap and +60 ms per 2000 page-memberships |
| 2 | `populate_sections` page→section attach | O(P × transparent-chain length) | chains are short | deeply nested `transparent = true` sections |
| 3 | `page_template` inheritance lookup | O(P × depth) `PathBuf` joins | depth ≤ 5 | very deep trees with unset `template` |
| 4 | Ancestor construction | O(S × depth) with a `PathBuf` per component | same | same |
| 5 | Per-section `sort_pages` using rayon | S invocations of `par_sort` on tiny slices | rayon overhead is small vs. the rest | thousands of sections with 1–2 pages each (the reference shape!) — worth re-checking after P0/P1 |
| 6 | `find_taxonomies` | O(T × K × ‖term‖ log ‖term‖) | K is small | high-cardinality taxonomies |
| 7 | Sitemap dedupe + sort | O((P+S+K) log …) with an `extra` clone per page | linear-ish | very large sites with big `[extra]` blocks |
| 8 | Backlink resolution per page | O(b log b) per page | b is small | wiki-style sites with thousands of backlinks per page |
| 9 | `check_internal_links_with_anchors` | O(L) with a `PathBuf` build + source-path clone per link | L is modest | link-dense sites (measured 0.8 ms at 6.6k links — fine) |
| 10 | `LoadData.result_cache` unbounded growth | O(distinct data files × size) retained | data files are small | the reference site retains 65 MB of parsed JSON |

### TOP 10 allocation sources

| # | Source | Allocations | Bytes | Retained? |
| - | ------ | ----------- | ----- | --------- |
| 1 | `RenderCache::build` re-serialization | 7.4–10 M | 0.96–1.2 GB | yes, ~86% of peak heap |
| 2 | `render + write outputs` (render → minify → write strings) | 9.4 M | 1.0 GB | no, transient |
| 3 | `render markdown` (HTML + event buffers) | 1.6 M | 310 MB | partly (~163 MB is the HTML) |
| 4 | `register_early_global_fns` clones | 1.1 M | 132 MB | ~92 MB |
| 5 | `parse pages` (front matter, slug, `FileInfo` paths) | 276 k | 53 MB | ~42 MB |
| 6 | `index` (`permalinks`, `reverse_aliases`) | 48 k | 23 MB | yes |
| 7 | `config` + highlighting registry | 185 k | 33 MB | 23 MB |
| 8 | `populate sections` (cloned `ancestors`/`subsections` vectors) | 93 k | 6.2 MB | yes |
| 9 | `fill_backlinks` | 97 k | 9.8 MB | yes |
| 10 | `discover + parse sections` | 58 k | 6.6 MB | yes |

### TOP 10 recommended changes

1. **PERF-001** — release the `load_data` lock before doing I/O and parsing.
2. **PERF-003** — remember directories already created instead of
   `create_dir_all` per file.
3. **PERF-005a** — build section/term cache values by inserting existing
   `Arc`-backed `Value`s instead of round-tripping them through serde.
4. **PERF-005b** — parallelise the per-page serialization pass in
   `RenderCache::build`.
5. **PERF-004** — parallelise (or defer) the output-directory clean.
6. **PERF-006** — parse sections in parallel; single `read_dir` per directory
   instead of a nested `WalkDir`.
7. **PERF-002** — per-thread highlighting registries, or an upstream giallo fix
   so `RegSet` is not shared.
8. **PERF-007** — parallelise the static copy.
9. **PERF-010** — register Tera functions once, and share `Config`/`permalinks`
   behind `Arc` instead of cloning per function.
10. **PERF-009** — avoid the `stat` per `load_data` cache-key computation once
    PERF-001 lands (it becomes the remaining serial syscall on that path).

## Summary table

| ID | Location | Operation | Called | Complexity | Evidence | Baseline cost | Priority |
| -- | -------- | --------- | ------ | ---------- | -------- | ------------- | -------- |
| PERF-001 | `templates/src/functions/load_data.rs:317` | global mutex held across file read + parse | once per `load_data()` | O(calls) forced serial | profile: 61.0% of busy CPU (reference proxy), 72.4% (data-heavy-4k), attributed to `LoadData::call → Mutex::lock` | 6306/10334 busy samples | **P0** |
| PERF-002 | `giallo-0.5.2 pattern_set.rs:21` via `markdown::render_content` | `Mutex<RegSet>` around every Oniguruma match | once per code-block token | O(matches) forced serial | profile: 80.6% of busy CPU blocked (markdown-heavy-4k), 39–57% elsewhere | 49 456/61 447 busy samples | **P0** |
| PERF-003 | `site/src/queue.rs` `write_output` | `create_dir_all` per rendered output | once per output file | O(outputs × depth) syscalls | profile: 18.9% busy CPU (simple-1k) — **but two fixes measured no gain, see OPTIMIZATIONS.md** | none recoverable | **rejected** |
| PERF-004 | `utils/src/fs.rs:216` `clean_site_output_folder` | single-threaded recursive delete of previous output | once per build | O(output files) serial | timings: 663 ms = 36.3% of wall — **but parallel deletion and rename-aside both measured no wall gain, see OPTIMIZATIONS.md** | none recoverable | **rejected** |
| PERF-005 | `render/src/cache.rs:59` `RenderCache::build` | sequential; deep serde re-serialization of every page value into its section and every taxonomy term | once per build | O(P × value size × memberships), single-threaded | timings: 346 ms = 24.3% of wall (mixed-4k); RSS 406 KB/page (many-taxonomies) vs 77 KB/page (simple) | 346 ms wall, ~2.6 GB RSS at 8k | **P1** |
| PERF-006 | `site/src/lib.rs:190` discover loop | serial `WalkDir` + a second `WalkDir(max_depth=1)` per directory + serial `Section::from_file` | once per build | O(dirs) serial, 2 traversals | timings: 232–357 ms = 13–27% of wall (reference proxy, 1640 sections) | 232 ms wall | **P1** |
| PERF-007 | `utils/src/fs.rs:107` `copy_directory` | serial copy of the static tree, 2 `stat`s per file | once per build | O(static files) serial | timings: 138–170 ms = 9–10% of wall (989 files / 55 MB) | 170 ms wall | **P2** |
| PERF-008 | `render/src/cache.rs:84`,`151` sibling injection | `Value::into_map()` on a shared `Arc<Map>` forces a map copy per page with siblings | once per page | O(P × map size) | profile: `Value as Serialize` 2.9–4.2% busy CPU; part of PERF-005's memory | included in PERF-005 | P2 |
| PERF-009 | `templates/src/functions/load_data.rs:157` | `get_file_time()` `stat` on every cache-key computation | once per `load_data()` | O(calls) syscalls | 4843 calls on the reference workload; `stat` visible in self time | small but on the P0 path | P2 |
| PERF-010 | `site/src/tpls.rs:5` `register_early_global_fns` | clones `Config`, `permalinks`, `colocated_assets` and the whole `Tera` per registration; runs twice | 2× per build | O(P) copies | timings: 44 ms = 3.1% of wall (mixed-4k) | 44 ms wall | P2 |
| PERF-011 | `config` highlighting registry init | ~170 MB RSS and ~110 ms before any page is processed | once | O(1) | baseline: 100-page build = 128 ms / 174 MB | fixed overhead | P3 |

## Detail

### PERF-001 — `load_data` serializes every data load (P0)

**Problem.** `LoadData::call` acquires `self.result_cache.lock()` at line 317 and
the guard stays alive until the function returns at line 401. Everything in
between happens under the lock: `read_file` (323), the blocking HTTP request
(326–383) and the JSON/TOML/CSV/YAML/XML parse (387–395). On the hit path the
`Value` clone at line 319 is also under the lock.

**Evidence.** On the reference-shaped workload 6306 of 10 334 busy samples
(61.0%) are threads parked in `__psynch_mutexwait` with
`LoadData::call → std::sync::Mutex::lock` as the deepest own frame. `data-heavy`
shows the same at 72.4%. The reference site loads 4695 *distinct* JSON files,
so the cache never hits and every call does full I/O plus parse inside the
critical section.

**Current complexity.** Work is O(files × size) but with a serialization factor
of 1 — no matter how many cores, `load_data` work is single-threaded.

**Expected complexity.** O(files × size) spread across all workers, with the
lock held only for a hash lookup and an insert.

**Proposed change.** Take the lock, look up, drop the guard; on a miss do the
read/fetch/parse unlocked, then re-acquire only to insert. Two threads racing on
the same key would each parse once and one insert wins — same observable result,
strictly less time. (A per-key `OnceLock`-style entry would avoid even that
duplication, but adds machinery for a case that costs nothing today.)

**Correctness risk.** Low. Observable behaviour is unchanged: the same `Value`
is returned, errors are still produced per call, and the cache key already
includes the file mtime.

**Benchmark.** `data-heavy` 1k–8k and the reference proxy; plus a thread sweep
(`bench.py threads`) to show parallel efficiency going from ~1 to ~N.

### PERF-002 — syntax highlighting is serialized (P0)

**Problem.** `giallo::grammars::pattern_set::PatternSet` wraps its Oniguruma
`RegSet` in a `Mutex` because `onig_regset_search` mutates internal state. The
registry is shared by all rayon workers, so every pattern match in every code
block in the site contends on one lock per language.

**Evidence.** 49 456 of 61 447 busy samples (80.6%) on `markdown-heavy-4000` are
blocked with `PatternSet::find_at` as the deepest own frame; 57.4% on
`template-heavy`, 49.2% on `many-taxonomies`, 39–48% on the dense/mixed
scenarios. Highlighting is 94% of busy CPU on `markdown-heavy`, and that
scenario costs 1.18 ms/page against 0.36 ms/page for `simple-pages`.

**Isolated proof.** Same 2000-page site, same binary, only
`[markdown.highlighting]` removed (`CPU-PROFILE.md` §Parallel efficiency):

| threads | `render markdown`, highlighting ON | OFF |
| ------- | ---------------------------------- | --- |
| 1 | 3.37 s | 181 ms |
| 12 | 3.96 s | 28 ms |

Highlighting parallelises at **0.85×** on 12 cores (i.e. it gets slower), while
the identical markdown work without it reaches **6.5×**. The phase is 142×
slower with highlighting at 12 threads, but only 18× slower at 1 thread — the
gap *is* the serialization.

**Note on scope.** The reference site has highlighting disabled, so this hotspot
does not affect it — but it affects the majority of Zola sites, and it is the
single largest CPU item across the synthetic suite.

**Investigated 2026-08-12 — the fix belongs upstream.** Reading giallo:

* `PatternSet { rule_refs, regset: Option<Mutex<RegSet>> }`
  (`grammars/pattern_set.rs:21`) is the only lock on the hot path.
* Pattern sets are shared: `Registry::get_or_create_pattern_set` hands out
  `Arc<PatternSet>` from a registry-wide cache, so every worker highlighting the
  same language contends on one mutex.
* `Scope::new` also takes a global lock (`scope.rs:216`) but only during grammar
  compilation, not per token — it is not part of this problem.

A Zola-side workaround would mean a per-thread `Registry`, rebuilt from
`Registry::dump()` bytes with `Registry::load()`. That was rejected without
implementing it:

* memory: the registry retains ~23 MB (measured in `ALLOCATIONS.md`, the
  `config` phase), so 12 workers would add ~270 MB — comparable to the entire
  peak heap after PERF-005a;
* correctness: `Registry::load` calls `replace_global_scope_repo`, documented
  as "only the first call succeeds". Scope values are indices into that
  repository, so several registries sharing one repo is safe only as long as
  every thread loads a byte-identical dump. That is true today and would be an
  invisible trap tomorrow.

**Ceiling of a proper fix.** The phase costs 3.37 s single-threaded and 3.96 s
on 12 threads (`markdown-heavy-2000`). If matching parallelised like the rest of
the markdown work, the phase should approach 3.37/12 ≈ 0.3 s — roughly 3.6 s per
2000 code-heavy pages.

**Upstream ask.** giallo needs the `RegSet` to be per-thread (a thread-local or
a small pool of regsets per pattern set) rather than one mutex per pattern set.
`onig_regset_search` writes to internal region storage, which is why the mutex
exists, so the fix is to give each searcher its own storage rather than to
remove the lock.

**Correctness risk.** Medium — highlighting output must be byte-identical, which
the output-equivalence gate checks directly.

### PERF-003 — `create_dir_all` per output file (P1)

Every rendered output calls `fs::create_dir_all(parent)` before `fs::write`.
For a 4-level-deep site that is up to 4 `mkdir` syscalls per page, all but the
first failing with `EEXIST`. Measured at 18.9% of busy CPU on `simple-pages-1000`
(where nothing else competes) and 1237 ms of `mkdir` CPU on `mixed-realistic-4000`.

**Proposed change.** Keep a shared set of directories already created during
this build and skip the syscall when the parent is known. Alternatively derive
the directory set from the job list and create it once, up front, in parallel.

### PERF-004 — cleaning the output directory (P1)

`clean_site_output_folder` runs first, single-threaded, and deletes the whole
previous output tree. On the reference workload that is 6544 files / 73 MB and
takes **663 ms — 36.3% of the entire build**. It is invisible on a first build
into an empty directory, which is why the benchmark harness deliberately
pre-populates the output directory.

**Proposed change.** Parallelise the delete across top-level entries, and/or
rename the tree aside and delete it while the build proceeds. Needs care: the
build writes into that directory immediately afterwards, and error reporting
must stay the same.

### PERF-005 — render cache: sequential and duplicating (P1)

Two separate problems in one phase:

1. **Sequential.** `RenderCache::build` is 346 ms (24.3% of wall) on
   `mixed-realistic-4000` and runs on one core while 11 others idle.
2. **Duplicating.** `SerializingSection.pages: Vec<Value>` (`cache.rs:125`) and
   `SerializedTaxonomyTerm.pages` (`cache.rs:200`) embed the member pages'
   values, and `Value::from_serializable` re-serializes them through serde
   (`tera-2.1.1 value/mod.rs:1062` walks maps/strings instead of sharing the
   underlying `Arc`s). A page belonging to 6 taxonomy terms is materialised 7+
   times, each copy carrying the page's full rendered HTML.

   This is what the memory curve shows: 77 KB/page for `simple-pages` versus
   406 KB/page for `many-taxonomies`, whose only structural difference is 6
   terms per page. At 16k pages `mixed-realistic` already needs 5.1 GB.

**Proposed change.** Build the cached section/term maps directly from already
built `Value`s (inserting the existing `Value`, which is `Arc`-backed and clones
in O(1)) instead of round-tripping the whole struct through serde; and
parallelise the per-page serialization pass. Template-visible structure must be
identical — the equivalence gate covers it.

### PERF-006 — discovery and section parsing are serial (P1)

The content walk is a single-threaded `WalkDir`, and for every directory it
starts a *second* `WalkDir` with `max_depth(1)` to find `_index.*` files, then
parses each section inline. Pages are parsed in parallel afterwards, sections
are not. On the reference workload (1640 sections) this is 232–357 ms, 13–27%
of wall time.

**Proposed change.** Walk once to collect directories, then parse sections in
parallel; replace the nested `WalkDir` with a single `read_dir` per directory.
The draft-section `skip_current_dir` behaviour and the error ordering must be
preserved.

### PERF-007 — static copy is serial (P2)

`copy_directory` walks and copies file by file, doing `metadata()` on source and
destination for each. 989 files / 55 MB costs 138–170 ms of wall time on the
reference workload. Parallelising with rayon is straightforward; `hard_link_static`
already exists as a user-side mitigation.

## What is explicitly *not* a hotspot

Recorded so future work does not re-litigate these:

* **No superlinear behaviour.** Every scenario is linear in time from 2k pages
  up (`SCALING.md`); the architecture is already index-based.
* **Page/permalink lookup** is `AHashMap`-keyed everywhere; no linear scans were
  found in the static audit or the profiles.
* **Internal link resolution, backlinks, taxonomy construction, sorting, path
  collision detection**: each under 1% of busy CPU.
* **`pulldown-cmark`**: 0.4% of busy CPU. Markdown *parsing* is not the problem;
  highlighting is.
* **Regex compilation**: every regex in the codebase is a `LazyLock` static.

## Execution order

1. **PERF-001** — biggest win on the motivating workload, smallest and safest change.
2. **PERF-003** — cheap, benefits every site, no semantic surface.
3. **PERF-005** — largest memory win plus 20%+ wall on mixed workloads.
4. **PERF-004** — biggest single item on the reference workload after PERF-001.
5. **PERF-006** — unlocks the remaining sequential chunk of `load`.
6. **PERF-002** — largest CPU item overall, but needs dependency-level design.
7. **PERF-007**, then re-profile before touching anything in P2/P3.
