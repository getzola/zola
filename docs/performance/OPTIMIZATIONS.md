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

**Not committed.** The doctrine rejects optimizations whose benefit cannot be
demonstrated. The hotspot itself is real; what is disproven is this particular
fix. The next variant worth measuring is a **thread-local** set of created
directories: it keeps the syscall saving without any shared lock, at the cost
of a few duplicate `mkdir` calls across worker threads.
