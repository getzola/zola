# Evidence — ZPERF-001

Every significant claim in [paper.md](paper.md), its class, and what backs it.
Classes are defined in [../METHODOLOGY.md](../METHODOLOGY.md). Printed figures
are additionally declared in [data/measurements.toml](data/measurements.toml),
where the machine-checkable ones carry the artifact path and the JSON key that
`scripts/dev.sh papers validate` re-extracts.

**Baseline** throughout: commit `9ec4407a`, upstream Zola 0.23.3 with only the
`--timings` instrumentation added — upstream giallo, hash-ordered maps, platform
allocator. **Candidate**: commit `c712c29d` unless stated otherwise.
**Machine**: Apple M4 Pro, 12 cores, 24 GiB, macOS 26.2, release builds with the
profile pinned by `scripts/perf/build.sh`.

---

## E-001 — Cumulative wall-time and memory results

**Claim.** Wall time fell 14–74% and peak memory 40–89% across nine synthetic
workloads; every wall figure is unanimous across rounds.

**Class.** `measured`

**Source.**
`benchmarks/results/m4-pro-12c-24gb-mac16-8/ab/baseline-vs-current-synthetic.json`

**Method.** `scripts/perf/ab.py`, three interleaved rounds per site, order
flipped between rounds, one discarded warmup per side. The reported statistic is
the median of the per-round paired deltas, with a flag for whether all rounds
agreed on the sign.

**Caveat.** Three CPU deltas (`mixed-realistic-16000`, `template-heavy-4000`,
`deep-sections-4000`) had rounds disagreeing on the sign and are marked
unresolved in the paper's table. The wall and RSS deltas were unanimous
throughout.

---

## E-002 — Reference-site result

**Claim.** −33.3% wall, −35.1% CPU, −25.5% peak RSS on the real 3776-page site.

**Class.** `measured`

**Source.**
`benchmarks/results/m4-pro-12c-24gb-mac16-8/ab/baseline-vs-current-real-site.json`

**Method.** As E-001, three rounds. All three metrics unanimous.

**Caveat.** The machine carried a load average around 26 from unrelated work
during this run, which inflates the absolute seconds (44.3 s and 32.2 s). The
same binary builds the site in 30.2 s on a quiet machine. Pairing is what makes
the delta usable; the absolutes are quoted only to identify what was compared.

---

## E-002b — The reference-site comparison, replicated

**Claim.** Run a second time in a later session with the same binaries and
procedure: CPU −35.6% (against −35.1%), peak RSS −22.0% (against −25.5%), wall
−26.1% (against −33.3%). Both sessions unanimous across rounds.

**Class.** `measured`

**Source.**
`benchmarks/results/m4-pro-12c-24gb-mac16-8/ab/baseline-vs-current-real-site-session2.json`

**Method.** As E-001. This artifact is the first to carry its own provenance —
SHA-256 of both binaries, the repository commit, and the machine — because
`ab.py` was extended to record it after the first session's provenance had to be
written into this manifest by hand.

**Caveat.** "Quiet" describes the machine at the start: it did not stay that way.
The baseline side's wall times spread across 35.1 s and one round took 70 s while
spending an ordinary 280 s of CPU. That is why the paper presents the two
sessions side by side rather than replacing one with the other — the disagreement
is the finding.

**Note on the binaries.** The baseline binary's hash differs from the one used in
the first session because it was rebuilt from the same commit; the current
binary's hash corresponds to the Rust tree at `c712c29d`, which is unchanged at
the repository commit recorded in the artifact (`f0d351a1`). The artifact records
both the hashes and the commit precisely so a reader can check that for
themselves rather than trust this paragraph.

## E-003 — Site shape and build cost of the reference workload

**Claim.** 3776 pages, 1640 sections, 6592 output files, 9.03 GB of HTML,
493 MB peak RSS to build.

**Class.** `measured`

**Source.**
`benchmarks/results/m4-pro-12c-24gb-mac16-8/20260812T184616Z-68d5e8a9/site-vomaste.json`

**Method.** `scripts/perf/bench.py site`, hyperfine-driven, three runs plus a
warmup, output measured after the run.

**Caveat.** The wall figure in that artifact (43.6 s) is contaminated by machine
load and is deliberately not cited in the paper. The counts, output size and
peak RSS are not load-sensitive.

---

## E-004 — A quarter of the reference build's CPU was in the platform allocator

**Claim.** 34 s of 138 s of busy CPU in `_xzm_free`, `xzm_malloc`,
`_xzm_xzone_malloc`, `_malloc_zone_malloc`, `_xzm_xzone_malloc_tiny` and `_free`.

**Class.** `observed`

**Source.** `samply` CPU profile of a `profiling`-profile build, summarised with
`scripts/perf/analyze_profile.py`. Per-symbol table transcribed in
`docs/performance/OPTIMIZATIONS.md` under PERF-012.

**Method.** `samply record --save-only --unstable-presymbolicate -- zola build
--force`, then `analyze_profile.py --top 18`.

**Caveat.** The profile itself is not committed — 3 MB of machine-specific
symbol data — so this figure cannot be re-extracted from the repository. It can
be reproduced by re-recording. The denominator excludes idle worker frames
(`kevent`, `__psynch_cvwait`, `__workq_kernreturn`), which is stated where the
number is used.

---

## E-005 — mimalloc result

**Claim.** −23.7% build CPU and −9.9% peak RSS on the reference site, unanimous
across five rounds; no measurable effect on small-page synthetic fixtures.

**Class.** `measured`

**Source.**
`.../ab/perf012-mimalloc-real-site.json`, `.../ab/perf012-mimalloc-synthetic.json`

**Method.** As E-001, five rounds for the real site, five for the synthetics.

**Caveat.** Wall time on the real site was −20.9% but not unanimous, so the
paper quotes CPU. The synthetic CPU deltas (−7.8%, −1.2%, +1.8%) all had rounds
disagreeing on the sign and are reported as "no measurable effect", not as small
wins. Peak RSS on the synthetics moved *up* 4–6% at 4000 pages and down 7% at
16000; the paper's claim is limited to the workloads measured.

---

## E-006 — Output equivalence for the allocator change

**Claim.** Byte-identical output on `mixed-realistic-1000` and on the full
6592-file reference site.

**Class.** `measured`

**Source.** `scripts/perf/compare_output.py`, `RESULT: IDENTICAL` on both.

---

## E-007 — Hash memoization: unresolved in A/B, visible in the profile

**Claim.** Whole-build paired CPU delta −0.4% with rounds disagreeing;
`compute_hash` self time 2217 ms before, absent from the top 40 after.

**Class.** `measured` (the A/B) and `observed` (the profile)

**Source.** `.../ab/perf013-hash-memo-real-site.json`; profile method as E-004.

**Method.** Five interleaved rounds. The session's paired CPU spread was 76 s
because another program was using three cores, which is why a ~1% effect could
not be resolved.

**Caveat.** The paper states the A/B as a non-result rather than quoting −0.4%
as a win. The profile evidence is what justifies the change, alongside the
correctness argument that the previous code re-read one file up to 5601 times.

---

## E-008 — Rejected: parallelising the static copy

**Claim.** 10–30% slower in every round on 5000 files of 1 KB: 640/814/842 ms
serial against 838/899/1023 ms parallel; no effect on the reference site.

**Class.** `rejected`

**Source.** `docs/performance/OPTIMIZATIONS.md`, rejected-experiment entry for
PERF-007. Phase timings from `zola build --timings`, `copy static` accumulator.

**Method.** Binaries alternated, three rounds, two workloads.

**Caveat.** Terminal-transcribed phase timings; no JSON artifact. The code was
reverted, so only the measurements remain.

---

## E-009 — Rejected: directory caching and output-clean strategies

**Claim.** Caching created directories (two variants), parallel deletion of the
previous output, and renaming the tree aside all failed to move wall time; the
rename-aside variant removed a 930 ms phase from the timeline without changing
the total.

**Class.** `rejected`

**Source.** `docs/performance/OPTIMIZATIONS.md`, PERF-003 and PERF-004 entries.

---

## E-010 — `zola serve --fast` served pre-edit content

**Claim.** A content edit was detected and re-parsed, the render job ran,
`Done in 0ms` was printed, and the server returned the page as it was before the
edit. Present in the baseline binary, therefore upstream behaviour.

**Class.** `observed`

**Source.** Reproduced by hand on a 100-page fixture against both
`/tmp/zola-BASE` (commit `9ec4407a`) and the then-current binary; the same edit
under plain `zola serve` rebuilt correctly in 24 ms. Regression tests in
`components/site/tests/fast_rebuild.rs`; the two covering the bug were confirmed
to fail without the fix.

**Method.** Start `zola serve --fast`, rewrite a page's front-matter title, wait
for the debouncer, request the page.

---

## E-011 — Root cause of E-010

**Claim.** Rendering reads page and section values from `RenderCache`, which the
fast path never refreshed.

**Class.** `code-fact`

**Source.** `components/render/src/renderer.rs` (`render_page` reads
`self.cache.pages`), `components/site/src/lib.rs`
(`add_and_render_page` did not call `rebuild_cache`).

---

## E-012 — `--fast` rebuild latency after the fix

**Claim.** 34–41 ms per content edit on a 4000-page site, against 447–481 ms for
the full rebuild `serve` performs without `--fast`.

**Class.** `observed`

**Source.** The `Done in Nms` line `zola serve` prints, three edits per mode.

---

## E-013 — `zola serve --output-dir` broke every rebuild after the first

**Claim.** Each rebuild failed with *"Directory already exists. Use --force to
overwrite"*, printed `Done in 23ms`, and served the previous build. Present in
the baseline binary.

**Class.** `observed`

**Source.** Reproduced against `/tmp/zola-BASE` and the then-current binary.

**Caveat.** No automated test: the guard lives in the binary's serve loop, which
has no test harness, and extracting it to make it unit-testable would test a
tautology. Verified by hand.

---

## E-014 — `zola serve` memory footprint

**Claim.** 9371 / 9368 / 9405 MB physical footprint before compression, 882 /
870 / 878 MB after, 289 MB with `--store-html`; 493 MB to build the same site.

**Class.** `observed` (footprints) and `measured` (the build figure, E-003)

**Source.** macOS `footprint -p <pid>`, `phys_footprint`, three interleaved
rounds with the binaries alternated.

**Method.** Start `zola serve` against the reference site, wait for the initial
build, read `footprint -p`.

**Caveat.** No committed artifact — these are terminal readings. `ps -o rss`
reports 8–20 MB for the same process and is the wrong metric; the discrepancy is
described in the paper because it nearly caused the finding to be dismissed.

---

## E-015 — Compression ratio on the reference site's HTML

**Claim.** 29× at zstd level 1, per output; 32.1× at level 3, 33.3× at level 6.

**Class.** `observed`

**Source.** 25 outputs sampled at random (seed 7) from a full build, each
compressed individually.

**Caveat.** Sampled, not exhaustive, and specific to this site's shape — 88% of
each page is the same navigation tree. A site without that redundancy will
compress less.

---

## E-016 — Startup cost of compression

**Claim.** Median +13% across eight interleaved rounds; six slower, two faster,
one 27.2 s outlier. Unresolved in sign; consistent with ~2 s of arithmetic.

**Class.** `observed`

**Source.** The `Done in Ns` line after `zola serve`'s initial build, eight
interleaved rounds across two sessions.

**Caveat.** The machine was loaded (load average 9–20) throughout. The paper
reports this as "about two seconds", not as a percentage, and says the sign was
not unanimous.

---

## E-017 — Remaining cost structure

**Claim.** Tera interpretation 28% and minify-html 23% of busy CPU on the
reference site after the changes.

**Class.** `observed`

**Source.** Inclusive time from the post-change `samply` profile, same method as
E-004.

---

## E-018 — Nothing is superlinear

**Claim.** Every scenario is linear in page count from 2k upwards.

**Class.** `measured`

**Source.** `docs/performance/SCALING.md`, growth-model fitting over the
baseline matrix (`scripts/perf/scaling.py`).

---

## E-019 — Fixed startup cost

**Claim.** A 100-page build went from 128 ms / 174 MB to 52 ms / 76 MB, of which
32 ms is the highlighting registry loaded during config parsing.

**Class.** `observed`

**Source.** `docs/performance/BASELINE.md` for the baseline figure;
`/usr/bin/time -l` and `zola build --timings` for the current one.

---

## E-020 — The determinism prerequisite

**Claim.** Before the fix, two runs of the same binary produced different bytes
for any page with more than one taxonomy.

**Class.** `code-fact` and `measured`

**Source.** `page.taxonomies` was a `HashMap` and Tera's maps were hash-ordered;
`compare_output.py` reported differences between two runs of one binary. The fix
is covered by `render::cache::tests::taxonomies_serialize_in_a_stable_order`,
which fails without it.

---

## Claims deliberately not made

* No claim about Linux or Windows: nothing was measured there.
* No claim that "Zola is 33% faster". The 33.3% is one workload, one machine,
  one interleaved session, against a named baseline commit.
* No performance claim of any kind about the content-addressed / Merkle DAG
  direction. It has no implementation and no measurements.
* No claim that the `serve` work is finished. Render-on-demand is a design.
