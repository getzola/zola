# Optimizations

Append-only log of completed `PERF-*` items. Every entry carries measured
numbers; nothing here is estimated.

## Measurement method for A/B entries

Sequential A/B (all baseline runs, then all candidate runs) proved unreliable on
this machine: a 5-minute LTO build beforehand left it warm enough that
`data-heavy-4000` measured 5.4 s as "before" and 1.7 s as "after", implying a
3.2× win that **interleaved** measurement did not reproduce. Every A/B below is
therefore measured by alternating baseline and candidate runs in the same loop,
and reports the per-round pairs so the reader can see the spread.

The baseline binary is the previous commit's binary, kept as
`/tmp/zola-baseline-<sha>`, so the only difference between the two is the change
under test.

---

## PERF-001 — `load_data` no longer holds its cache lock across I/O and parsing

**Problem.** `LoadData::call` took `result_cache.lock()` before looking up the
cache and kept the guard alive until the function returned, so the file read,
the HTTP request and the JSON/TOML/CSV/YAML/XML deserialization all happened
inside the critical section. Since `load_data()` is typically called from a
template that runs for every page, every page render serialized against every
other one.

**Evidence (before).** CPU profile of the reference-shaped workload: 6306 of
10 334 busy samples (61.0%) were threads parked in `__psynch_mutexwait` with
`LoadData::call → std::sync::Mutex::lock` as the deepest own frame.
`data-heavy-4000`: 72.4% of busy samples blocked. See `CPU-PROFILE.md`.

**Change.** `components/templates/src/functions/load_data.rs`: scope the lock to
the lookup, drop it, do the fetch and parse unlocked, then re-acquire only to
insert. Two threads racing on the same missing key each parse once and the last
insert wins — the same value, strictly less waiting.

**Results.**

`render: page` accumulator (CPU time summed across threads, reference proxy,
`--timings`, interleaved):

| round | baseline | candidate |
| ----- | -------- | --------- |
| 1 | 6.502 s | 2.742 s |
| 2 | 6.435 s | 1.772 s |

**2.4–3.7× less CPU spent rendering pages.**

Wall time, `data-heavy-4000`, interleaved, 4 rounds:

| round | baseline | candidate |
| ----- | -------- | --------- |
| 1 | 2.90 s | 2.17 s |
| 2 | 3.59 s | 2.78 s |
| 3 | 3.63 s | 2.77 s |
| 4 | 3.48 s | 2.69 s |
| median | **3.53 s** | **2.73 s** (**−23%**) |

Wall time, reference proxy, interleaved, 4 rounds:

| round | baseline | candidate |
| ----- | -------- | --------- |
| 1 | 2.18 s | 1.89 s |
| 2 | 2.57 s | 2.65 s |
| 3 | 2.59 s | 2.50 s |
| 4 | 3.07 s | 2.73 s |
| median | 2.58 s | 2.58 s (**no change**) |

**Honest reading of the result.** The change does exactly what it was meant to
do — it removes the serialization, and the CPU spent in page rendering drops by
2.4–3.7×. It converts to wall-clock time on `data-heavy` (−23%), where per-page
JSON parsing is substantial. It does **not** improve the reference proxy's wall
time, because that build's `render + write outputs` phase is bounded by output
I/O, not by CPU: the freed threads simply idle. Phase wall time there was
675/909/824 ms before and 766/809/837 ms after — indistinguishable.

So: a real fix for a real serialization bug, a measured win on data-parse-heavy
sites, no win (and no regression) on I/O-bound ones. The reference workload's
remaining bottlenecks are PERF-004 (clean), PERF-003 (`create_dir_all`) and
PERF-006 (serial discovery).

**Memory.** Peak RSS unchanged: 374.65 MB before, 374.83 MB after (+0.05%).

**Correctness.**

* `cargo fmt --check` PASS
* `cargo clippy --all-targets --all-features` PASS (0 errors, no new warnings)
* `cargo test --workspace` PASS (461 passed, 0 failed)
* Output equivalence, reference proxy (6544 files): **IDENTICAL**
* Output equivalence, `data-heavy-4000` (4205 files): **IDENTICAL**

**Commit.** `perf(PERF-001): don't hold the load_data cache lock across I/O`

---

## Output determinism — a prerequisite for the equivalence gate (not a PERF item)

**Problem.** Two runs of the *same* binary produced different HTML.
`compare_output.py --baseline X --candidate X` on `mixed-realistic-1000`
reported 484 of 1547 files changed. The mandatory output-equivalence gate was
therefore meaningless for any site with more than one taxonomy on a page.

**Cause.** Two layers of hash-ordered maps:

1. `PageFrontMatter.taxonomies` was a `std::collections::HashMap`, and
2. `tera::Map` is a `HashMap` unless the `preserve_order` feature is enabled.

Templates iterate both directly (`{% for name, terms in page.taxonomies %}`),
so the order came from a per-process random hash seed.

**Change.** `taxonomies` becomes a `BTreeMap`, and Zola enables tera's
`preserve_order` feature so every `Value` map keeps insertion order. Both are
needed: the feature alone would still preserve a random insertion order.

**Test.** `render::cache::tests::taxonomies_serialize_in_a_stable_order` builds
a cache for a page with 20 taxonomies and asserts sorted key order. It fails
without the change (verified by toggling the feature off) and passes with it.
An earlier 3-key version of the same test passed by luck — recorded here
because it is exactly the trap this gate exists to avoid.

**Output impact.** This change is *about* output, so byte-equality with the
previous binary is not expected. On `mixed-realistic-1000`, 487 of 1547 files
differ — and all 487 contain **exactly the same characters** as before, i.e.
the difference is purely reordering. No file is added or removed.

**Performance (interleaved, `mixed-realistic-4000`):**

| round | before | after |
| ----- | ------ | ----- |
| 1 | 2.71 s | 2.51 s |
| 2 | 2.19 s | 2.07 s |
| 3 | 2.37 s | 2.15 s |
| median | 2.37 s | **2.15 s (−9%)** |

Peak RSS: **384 MB → 326 MB (−15%)**. `IndexMap` iterates contiguously and
stores entries more compactly than the hash map it replaces, so the fix is
faster and smaller as well as correct.

**Gates.** `scripts/dev.sh quality`: ALL PASS (fmt, clippy ratchet,
461 tests).

---

## Rejected experiment: caching created directories (hotspot PERF-003)

**Hypothesis.** `write_output` calls `fs::create_dir_all` once per rendered
output; the CPU profile attributed 18.9% of busy samples on
`simple-pages-1000` (and 1237 ms of `mkdir` on `mixed-realistic-4000`) to it.
Remembering the directories already created should remove those syscalls.

**Implementation tried.** An `Arc<Mutex<AHashSet<PathBuf>>>` on `Queue`,
checked before `create_dir_all` and updated after; the lock is held only for
the hash lookup.

**Result — no improvement.** Write-phase CPU (`out: write file` accumulator,
`simple-pages-4000`, interleaved):

| round | without | with |
| ----- | ------- | ---- |
| 1 | 8.84 s | 7.00 s |
| 2 | 7.71 s | 7.67 s |
| 3 | 7.71 s | 7.98 s |
| median | 7.71 s | 7.67 s |

Whole-build wall time was likewise indistinguishable (median 1.31 s vs 1.39 s,
with a 1.03–1.37 s spread on the *same* binary). The mutex costs roughly what
the skipped `mkdir` calls save.

**Second variant, also rejected.** The thread-local set (no shared lock, a few
duplicate `mkdir` calls across workers) was implemented and measured after
PERF-005a and PERF-010 had made the write path the largest remaining item —
`out: write file` was then 7.0 s of CPU across 4804 writes, 1.4 ms each:

| round | `out: write file`, without | with | wall without | wall with |
| ----- | -------------------------- | ---- | ------------ | --------- |
| 1 | 6.989 s | 7.107 s | 1.55 s | 1.63 s |
| 2 | 7.116 s | 6.935 s | 1.68 s | 1.65 s |
| 3 | 7.079 s | 7.440 s | 1.73 s | 1.65 s |
| median | 7.079 s | 7.107 s | 1.68 s | 1.65 s |

No effect either. **PERF-003 is closed as rejected**: the cost in the write
path is the file creation and write themselves, not the redundant `mkdir`
calls, and `mkdir` returning `EEXIST` on APFS is cheap enough that removing it
is unmeasurable. The CPU profile's 18.9% attribution to `create_dir_all` on
`simple-pages-1000` counted samples in the kernel while other workers were
parked, and overstated the share of wall time it could return.

---

## PERF-005a — stop re-serializing page values into sections and taxonomies

**Problem.** `RenderCache::build` embedded already-built page values into their
section and into every taxonomy term by handing them to
`Value::from_serializable`. That function walks a structure through serde and
rebuilds every map, key and string it meets — including `Value`s that were
already built — so a page belonging to a section and four taxonomy terms was
materialised five extra times, each copy carrying the page's full rendered HTML.

**Evidence (before).** `build render cache` was 24.3% of wall time on
`mixed-realistic-4000` while running single-threaded, allocated 7.4–10 M times
(963 MB–1.2 GB) and accounted for ~86% of the peak heap. A controlled
experiment showed each additional taxonomy membership cost ≈67 KB of retained
heap and ≈60 ms per 2000 pages — see `ALLOCATIONS.md`.

**Change.** `components/render/src/cache.rs`: serialize the section / term /
taxonomy struct with an *empty* placeholder for its child collection, then
replace that entry with the `Value`s already in hand. `tera::Value` is
`Arc`-backed, so this is a refcount bump instead of a deep copy. The
placeholder means the key already exists, and re-inserting an existing key in
an order-preserving map keeps its position — so field order, and therefore the
bytes templates produce, are unchanged.

**Results.**

`build render cache` phase (`--timings`, `many-taxonomies-4000`):

| round | before | after |
| ----- | ------ | ----- |
| 1 | 344.6 ms | 22.2 ms |
| 2 | 349.7 ms | 22.4 ms |
| 3 | 339.8 ms | 22.4 ms |
| median | 344.6 ms | **22.4 ms (−94%, 15× faster)** |

Wall time, interleaved:

| site | before (median) | after (median) | change |
| ---- | --------------- | -------------- | ------ |
| `many-taxonomies-4000` | 2.08 s | 1.58 s | **−24%** |
| `mixed-realistic-4000` | 2.04 s | 1.61 s | **−21%** |
| `mixed-realistic-16000` | 8.06 s | 6.49 s | **−19%** |

Peak RSS:

| site | before | after | change |
| ---- | ------ | ----- | ------ |
| `many-taxonomies-4000` | 1303 MB | 273 MB | **−79%** |
| `mixed-realistic-4000` | 1108 MB | 312 MB | **−72%** |
| `mixed-realistic-16000` | 4070 MB | 741 MB | **−82%** |

The memory scaling wall identified in `SCALING.md` is gone: a 16k-page site
that needed 4.1 GB now needs 741 MB, so per-page cost drops from ~330 KB to
~46 KB.

**PERF-005b (parallelising the phase) is no longer worth doing**: the phase it
would parallelise now takes 22 ms.

**Correctness.**

* output equivalence **IDENTICAL** on `many-taxonomies-2000` (2269 files),
  `mixed-realistic-1000`, `deep-sections-1000`, `dense-internal-links-1000`;
* `scripts/dev.sh quality`: ALL PASS (fmt, clippy ratchet, 461 tests).

**Commit.** `perf(PERF-005a): reuse cached page values instead of re-serializing them`

---

## Rejected experiment: parallel output cleaning (hotspot PERF-004)

**Hypothesis.** `clean_site_output_folder` deletes the previous output with a
single-threaded `remove_dir_all`; it is 36% of wall time on the reference
workload (663 ms for 6544 files / 73 MB, and 1.3 s once the site grew to 9 GB).
The top-level entries are independent subtrees, so deleting them with rayon
should shorten the phase.

**Result — slower.** `clean output dir` phase, interleaved (the first round of
each pair cleans an empty directory and is excluded):

| site | serial | parallel |
| ---- | ------ | -------- |
| reference proxy | 3.889 s | 6.784 s |
| reference proxy | 3.457 s | 2.797 s |
| `mixed-realistic-8000` | 978.9 ms | 1.856 s |

Whole-build wall on the reference proxy was 26.5–33.8 s serial against
31.6–35.1 s parallel. Two of three phase samples and all three whole-build
samples were worse.

**Why.** APFS serialises directory-metadata mutations; concurrent `unlink`
storms contend rather than overlap. Parallelism is not the lever here.

**Second variant — rename aside, delete in the background — also rejected.**
Approved and implemented: the previous output is renamed to a sibling scratch
directory (one `rename`, regardless of size), deleted on a background thread
while the build runs, and joined before the build reports success. It worked
exactly as designed at the phase level, on the reference workload:

| phase | before | after |
| ----- | ------ | ----- |
| `clean output dir` | 929.8 ms | 0.2 ms (rename) |
| `wait for background clean` | — | 0.0 ms |

**And it made no difference to wall time.** Interleaved:

| site | before (median) | after (median) |
| ---- | --------------- | -------------- |
| reference proxy (9 GB output) | 24.42 s | 24.55 s |
| `mixed-realistic-8000` | 3.19 s | 3.33 s |
| `markdown-heavy-4000`, 8 rounds | 4.275 s | 4.350 s (+1.8%; min −0.7%) |

`markdown-heavy-4000` is the case that should have shown it most clearly: the
clean is 301 ms of a 4.3 s build (7.1%), the build is CPU-bound, and the run
spread was tight (4.22–4.40 s). The effect is absent.

**Why.** The deletion is not removed, only moved. The build already saturates
all 12 cores and the same disk, so a background deleter competes with the
workers for exactly the resources they need; total work is conserved. Winning
would require not waiting for the deletion at all — detaching it from the
process lifetime — which would let `zola build` return while it is still
writing to disk, and race with whatever consumes the output next.

**Not committed.** What was kept is this record: the clean phase can be made to
disappear from the timeline, and doing so buys nothing on a machine where the
build is already the bottleneck. PERF-004 is closed as rejected.

---

## PERF-006 — one directory read per directory during discovery

**Problem.** The content walk visited every directory twice. The outer
`WalkDir` yielded each entry, and then for every directory a *second*
`WalkDir` with `max_depth(1)` was started just to find its `_index.*` files.
Each file also paid a `path.is_dir()` `stat` that the directory read had
already answered.

**Change.** `components/site/src/lib.rs`: use the `file_type` the walk already
carries instead of `path.is_dir()`, and replace the nested `WalkDir` with a
plain `read_dir` that only stats the handful of `_index.*` candidates (kept, so
a symlinked index file still resolves — issue #1244). `read_dir` order is
filesystem-defined while the previous walk was sorted, and section insertion
order is observable through error ordering, so the candidates are sorted
explicitly.

**Results.** `discover + parse sections` phase, interleaved:

| site | before (median) | after (median) | change |
| ---- | --------------- | -------------- | ------ |
| `deep-sections-8000` (4000 sections) | 82.0 ms | 53.7 ms | **−35%** |
| reference proxy (1640 sections, 5416 files) | 195.5 ms | 200.4 ms | no change |

The reference proxy sees nothing: its discovery cost is reading and parsing
1640 `_index.md` files, not walking directories. On a section-dense tree the
saving is real and reproducible (3 of 3 rounds), but it is ~1% of that build's
wall time — this is a syscall reduction, not a headline win, and it is recorded
as such.

**Correctness.** Output equivalence IDENTICAL on `deep-sections-1000`,
`mixed-realistic-1000`, `simple-pages-1000`, `many-taxonomies-2000` and the
reference proxy. `scripts/dev.sh quality`: ALL PASS.

**Commit.** `perf(PERF-006): read each content directory once during discovery`

---

## Cumulative effect so far

Measured with the binary from the start of this round of work (PERF-001 and the
`--timings` instrumentation, commit `71be7609`) against the current tree
(determinism fix + PERF-005a + PERF-006). Interleaved, on a machine with free
disk — an earlier attempt at this table was invalidated when 9 GB output trees
filled the disk and builds began failing, which is recorded here because the
numbers looked spectacular and were meaningless.

| workload | before (median) | after (median) | wall | peak RSS before → after |
| -------- | --------------- | -------------- | ---- | ----------------------- |
| `mixed-realistic-4000` | 2.10 s | 1.72 s | **−18%** | 1371 MB → 311 MB (**−77%**) |
| `many-taxonomies-4000` | 2.21 s | 1.65 s | **−25%** | 1696 MB → 273 MB (**−84%**) |
| `mixed-realistic-16000` | 8.67 s | 6.80 s | **−22%** | 5152 MB → 742 MB (**−86%**) |
| reference proxy (3776 pages, 9 GB output) | 28.9 s | 24.6 s | **−15%** | — |

Against Zola 0.22 the reference site went from 252 s to ~25 s, though that
comparison also includes the 0.22→0.23 engine work and the template migration
(see `REAL-SITE.md`).

**Open, in priority order:** PERF-002 (highlighting serialized on giallo's
`RegSet` mutex — the largest remaining CPU item on sites with code blocks),
PERF-004 (output cleaning; parallel deletion was measured and rejected, the
rename-aside approach needs a decision), PERF-003 (the thread-local variant),
PERF-007, PERF-009, PERF-010.

---

## PERF-010 — share the highlighting registry instead of deep-copying it

**Problem.** `register_early_global_fns` clones `Config` into four Tera
functions (`get_url`, `trans`, `text_direction`, the `markdown` filter).
`Config` owns the giallo `Registry`, and `Registry::clone` deep-copies every
grammar, theme and injection — `ALLOCATIONS.md` measured the registry at ~23 MB
retained, and the phase at 1.1 M allocations / 132 MB with ~92 MB of that never
released. None of the copies is ever used to highlight anything: highlighting
goes through the `Config` the markdown renderer borrows.

**Change.** `components/config/src/config/markup.rs`: `Highlighting.registry`
becomes `Arc<Registry>`. Cloning a `Config` is then a refcount bump. Nothing
else changes — `Registry`'s methods are reached through `Deref`, and the
registry is still built (and mutated with extra grammars and themes) before it
is wrapped.

**Results.**

`register tera fns (early)` phase, `mixed-realistic-4000`, interleaved:

| round | before | after |
| ----- | ------ | ----- |
| 1 | 40.3 ms | 0.6 ms |
| 2 | 41.1 ms | 0.6 ms |
| 3 | 41.5 ms | 0.6 ms |

Peak RSS:

| site | before | after | change |
| ---- | ------ | ----- | ------ |
| `simple-pages-1000` | 184 MB | 86 MB | **−53%** |
| `mixed-realistic-4000` | 312 MB | 209 MB | **−33%** |
| `markdown-heavy-2000` | 392 MB | 291 MB | **−26%** |

A flat ~100 MB saving on every build, which is most of the remaining baseline
footprint on small sites.

**Correctness.** Output equivalence IDENTICAL on `mixed-realistic-1000` and
`markdown-heavy-1000` (the scenario that actually highlights).
`scripts/dev.sh quality`: ALL PASS.

**Commit.** `perf(PERF-010): share the highlighting registry behind an Arc`

---

## PERF-002 — fixed upstream in giallo: a RegSet per thread

**Problem.** `PatternSet` (`giallo/src/grammars/pattern_set.rs`) held
`Option<Mutex<RegSet>>`, because `onig_regset_search` writes to the regset's
internal region storage and one instance cannot be searched concurrently.
Pattern sets are handed out as `Arc` from the registry's cache, so every worker
highlighting the same language queued behind a single lock. Highlighting did
not parallelise at all: the markdown phase took **1.58 s on one thread and
1.69 s on twelve**.

**Change.** Keep the pattern strings in the `PatternSet` and compile a `RegSet`
per thread on first use, in a thread-local map keyed by a per-pattern-set id.
Only a thread that actually searches a set pays for a copy. The patch is
`docs/performance/giallo-thread-local-regset.patch`, applied to
`getzola/giallo@5e19db8` and measured through Zola with
`[patch.crates-io] giallo = { path = … }`.

**Results.** `render markdown` phase and whole-build wall time, interleaved:

| site | md phase before | md phase after | wall before | wall after |
| ---- | --------------- | -------------- | ----------- | ---------- |
| `markdown-heavy-2000` | 1.690 s | 0.284 s (**−83%**) | 2.25 s | **0.87 s (−61%)** |
| `markdown-heavy-4000` | 3.082 s | 0.505 s (**−84%**) | 4.23 s | **1.67 s (−61%)** |
| `template-heavy-4000` | 268.3 ms | 63.0 ms (−77%) | 1.40 s | 1.17 s (−16%) |
| `mixed-realistic-4000` | 273.0 ms | 68.4 ms (−75%) | 1.57 s | 1.38 s (−12%) |

Thread scaling of the markdown phase on `markdown-heavy-2000` — the measurement
that showed the bug in the first place:

| threads | before | after |
| ------- | ------ | ----- |
| 1 | 1.58 s | 1.58 s |
| 2 | ~1.6 s | 0.94 s |
| 4 | ~1.6 s | 0.56 s |
| 12 | 1.69 s | **0.27 s** (5.9×, efficiency 0.49) |

Peak RSS grows by **5–8 MB** (292 → 297 MB on `markdown-heavy-2000`), not the
~270 MB the per-thread-*registry* workaround would have cost: only the pattern
sets a thread actually uses are compiled, and a compiled regset is small next to
the grammar data.

**Correctness.** Output equivalence IDENTICAL against the unpatched binary on
`markdown-heavy-1000`, `markdown-heavy-4000`, `template-heavy-1000` and
`mixed-realistic-1000`; the patched binary also agrees with itself run to run.
giallo's own test suite is unchanged by the patch: 46 pass and 11 fail both
before and after, all 11 for missing grammar/theme fixtures that the repository
does not ship.

**Not committed to Zola** — there is nothing to commit here. The fix is one file
in a dependency; Zola picks it up by bumping `giallo` once it is released.
Known limitation, stated in the patch: thread-local entries live until the
thread exits, so a process that keeps building registries (`zola serve`
reloading a config) grows the map. Storing a `thread_local::ThreadLocal<RegSet>`
inside `PatternSet` would tie the storage to the object's lifetime instead, at
the cost of one dependency — the maintainer's call.

### PERF-002 without touching giallo — measured, and the trade-off

The same win is reachable from inside Zola: `Registry` is `Clone`, and cloning
starts a fresh pattern cache while keeping the same `Scope` ids (so the global
scope repository stays valid — unlike `Registry::load`, which replaces it).
A `thread_local` clone per worker, made lazily on that worker's first
highlight, gives every thread its own regsets.

Implemented and measured (three-way, interleaved, medians):

| workload | today | Zola-side per-thread `Registry` | patched giallo |
| -------- | ----- | ------------------------------- | -------------- |
| `markdown-heavy-2000` wall | 2.73 s | 1.07 s | 1.02 s |
| `markdown-heavy-4000` wall | 5.20 s | 1.80 s | 1.83 s |
| `mixed-realistic-4000` wall | 1.71 s | 1.63 s | 1.55 s |
| `markdown-heavy-2000` RSS | 291 MB | **600 MB** | 297 MB |
| `markdown-heavy-4000` RSS | 514 MB | **822 MB** | 521 MB |
| `mixed-realistic-4000` RSS | 209 MB | **519 MB** | 216 MB |

**It is as fast as the upstream fix and costs ~310 MB.** That is ~26 MB per
worker: a cloned registry duplicates all the grammars, where the giallo patch
duplicates only the compiled regsets a thread actually uses.

Freeing the clones after the markdown phase (`rayon::broadcast` of a
`clear_local_registries`) was also measured: it returns 36 MB of the 310
(824 → 786 MB, 519 → 483 MB). The peak is reached during highlighting itself and
the allocator keeps the pages, so the machinery does not pay for itself.

Output is byte-identical either way.

**Not committed**, because handing back 310 MB undoes most of what PERF-005a and
PERF-010 won and the choice is a trade, not an improvement. The three ways to
get the fix, in order of preference:

1. **Upstream giallo** — `docs/performance/giallo-thread-local-regset.patch`
   applies to `getzola/giallo@5e19db8`; Zola then bumps the dependency. Best
   result, no cost to Zola, needs a release.
2. **Vendor or fork the patched crate** and point at it with
   `[patch.crates-io] giallo = { path = … }` or a git revision. Same result
   today, at the cost of carrying ~10k lines and a 1.2 MB `builtin.zst` in the
   repository until upstream releases.
3. **The Zola-side per-thread registry above** — no dependency change, ~30
   lines, same speed, +310 MB.
