# Evidence — ZPERF-002

Every significant claim in [paper.md](paper.md), its class, and what backs it.
Classes are defined in [../METHODOLOGY.md](../METHODOLOGY.md). Printed figures
are additionally declared in [data/measurements.toml](data/measurements.toml).

**Nothing in this paper is `measured` in the strict sense of the series** —
re-extractable from a committed artifact under `benchmarks/results/` — except
the three input counts of the reference workload (E-008). The four experiments
were judged on phase accumulators printed by `zola build --timings` and on
`samply` CPU profiles, transcribed into `docs/performance/OPTIMIZATIONS.md`,
`HOTSPOTS.md` and `CPU-PROFILE.md` when they ran. The rejected code was reverted
in every case, so the experiments cannot be re-run from this tree; they can only
be re-implemented. That is the weakest evidence position in the series and is
stated in the paper's Limitations section as well as here.

**Machine** throughout: Apple M4 Pro, 12 cores, 24 GiB, macOS 26.2, APFS on the
internal SSD; release builds with the profile pinned by `scripts/perf/build.sh`.

**Baseline and candidate.** Each experiment is its own pair: the tree
immediately before the change against the same tree with the change applied,
built as two binaries and alternated within each round. There is no single
baseline commit for the paper, because the four experiments ran at four
different points in the program — PERF-003's second variant, in particular, was
deliberately re-run *after* PERF-005a and PERF-010 had changed the balance of the
build. Where that matters it is said in the paper.

---

## E-001 — PERF-003 hypothesis: `create_dir_all` looked expensive

**Claim.** The CPU profile attributed 18.9% of busy CPU on `simple-pages-1000`
to `std::fs::DirBuilder::create_dir_all`, and 1237 ms of `mkdir` CPU on
`mixed-realistic-4000`.

**Class.** `observed`

**Source.** `docs/performance/CPU-PROFILE.md` (call-tree share and the syscall
self-time paragraph); repeated in `docs/performance/HOTSPOTS.md`, PERF-003.

**Method.** `samply record --save-only` of a release build with debug symbols,
summarised with `scripts/perf/analyze_profile.py`.

**Caveat.** The profile is not committed. This figure is the *hypothesis*, and
the paper's point is that it did not survive: the attribution counts samples
parked in the kernel while other workers wait, which is not time the path can
give back.

---

## E-002 — PERF-003, shared-set variant: no effect

**Claim.** `out: write file` CPU on `simple-pages-4000` was 8.84 / 7.71 / 7.71 s
without the cache against 7.00 / 7.67 / 7.98 s with it (medians 7.71 s and
7.67 s); whole-build wall 1.31 s against 1.39 s, against a 1.03–1.37 s spread on
one binary.

**Class.** `rejected`

**Source.** `docs/performance/OPTIMIZATIONS.md`, rejected-experiment entry for
PERF-003.

**Method.** Three interleaved rounds, binaries alternated, phase accumulator
from `zola build --timings`.

**Caveat.** Round 1 favours the cache by 1.84 s and the other two do not; with a
same-binary spread of 0.34 s on the whole build, this session cannot resolve an
effect of the size claimed for it. The paper says "nothing", not "zero".

---

## E-003 — PERF-003, thread-local variant: also no effect

**Claim.** `out: write file` 7.079 s against 7.107 s; wall 1.68 s against
1.65 s. At the time the write path was 7.0 s of CPU across 4804 writes, 1.4 ms
each.

**Class.** `rejected`

**Source.** `docs/performance/OPTIMIZATIONS.md`, PERF-003, second variant; the
per-write cost also appears in `docs/performance/FINAL-REPORT.md` section 8.

**Method.** As E-002, three interleaved rounds, measured after PERF-005a and
PERF-010 had made the write path the largest remaining item — i.e. under the
conditions most favourable to the change.

---

## E-004 — PERF-004 hypothesis, and the parallel delete that was slower

**Claim.** The serial clean was 663 ms and 36.3% of wall on the reference proxy
at 6544 files / 73 MB, and 1.3 s once the output grew to about 9 GB.
Parallelising it measured 3.889 s against 6.784 s, 3.457 s against 2.797 s, and
978.9 ms against 1.856 s on `mixed-realistic-8000`; whole-build wall was
26.5–33.8 s serial against 31.6–35.1 s parallel.

**Class.** `observed` (the hypothesis) and `rejected` (the result)

**Source.** `docs/performance/CPU-PROFILE.md` and `HOTSPOTS.md` for the
hypothesis; `docs/performance/OPTIMIZATIONS.md`, rejected-experiment entry for
PERF-004, for the result.

**Method.** `clean output dir` phase accumulator, binaries alternated. The first
round of each pair cleans an empty directory and was excluded. The benchmark
harness pre-populates the output directory deliberately, because this phase does
not exist on a first build.

**Caveat.** One of the three phase pairs favoured the parallel version
(3.457 s against 2.797 s) and the paper reports that rather than the median of
the three, which would have hidden it. The whole-build samples were unanimous.

---

## E-005 — PERF-004, rename-aside: the phase disappeared, the build did not

**Claim.** `clean output dir` went from 929.8 ms to 0.2 ms with the join costing
0.0 ms, and wall time did not move: 24.42 s → 24.55 s on the reference proxy,
3.19 s → 3.33 s on `mixed-realistic-8000`, 4.275 s → 4.350 s over eight rounds
on `markdown-heavy-4000` (median +1.8%, best round −0.7%), where the clean is
301 ms of a 4.3 s build and the spread was 4.22–4.40 s.

**Class.** `rejected`

**Source.** `docs/performance/OPTIMIZATIONS.md`, PERF-004, second variant.

**Method.** Interleaved, binaries alternated; eight rounds for the
`markdown-heavy-4000` case because it is the one that should have shown the
effect most clearly.

**Caveat.** This variant was approved as a design before it was implemented, and
it did exactly what the design said it would do at the phase level. The
rejection is not that it failed to work; it is that the work was moved rather
than removed.

---

## E-006 — PERF-007: the parallel static copy was slower

**Claim.** On the reference site's static tree (989 files, 55 MB, a
170–190 ms phase) the parallel copy measured 191.0 / 196.6 / 211.0 ms serial
against 193.6 / 211.6 / 170.9 ms parallel — no effect. On 5000 files of 1 KB it
measured 640.1 / 813.6 / 841.9 ms serial against 837.6 / 899.0 / 1023 ms
parallel: worse in every round, by 10–30%.

**Class.** `rejected`

**Source.** `docs/performance/OPTIMIZATIONS.md`, rejected-experiment entry for
PERF-007; the file count is also in the committed artifact of E-008
(`results.0.input.static_files`).

**Method.** `copy static` phase accumulator, binaries alternated, three rounds
per workload.

**Caveat.** `~290 MB/s` is arithmetic over 55 MB and ~190 ms, not an independent
measurement of disk throughput, and is classed `interpretation`. The per-round
regressions (+30.9%, +10.5%, +21.5%) are computed here from the pairs above; the
"10–30%" range is what the log recorded at the time and is what the paper
prints.

---

## E-007 — PERF-009: rejected on re-profiling, not on an implementation

**Claim.** `get_file_time()` is called 4843 times on the reference workload, but
every `stat` in the whole build is 439 ms of self time and `canonicalize` a
further 360 ms — 0.9% of the build between them — so this item is worth a few
milliseconds. The timestamp it reads is what stops the cache serving a data file
that has changed.

**Class.** `observed` (the profile figures) and `code-fact` (what the timestamp
is for)

**Source.** `docs/performance/HOTSPOTS.md`, PERF-009 and PERF-015 rows;
`components/templates/src/functions/load_data.rs` for the cache-key computation.

**Method.** `samply` profile of the reference site taken *after* PERF-001
removed the lock this item was queued behind, summarised with
`scripts/perf/analyze_profile.py`.

**Caveat.** No implementation exists: this one was closed on the profile alone.
The 439 ms is an upper bound on what removing this single caller could save,
since it covers every `stat` in the build from every caller.

---

## E-008 — The reference workload's shape

**Claim.** 3776 pages, 1640 sections, 989 static files.

**Class.** `measured`

**Source.**
`benchmarks/results/m4-pro-12c-24gb-mac16-8/20260812T184616Z-68d5e8a9/site-vomaste.json`,
keys `results.0.input.{page_files,section_files,static_files}`.

**Caveat.** The artifact measures the real site. The "reference proxy" used in
the experiments is that site's content with substitute templates
(`scripts/perf/make_proxy_site.py`), so the input counts are the same and the
output tree is not — 6544 files / 73 MB at the time of the earliest experiment
here, about 9 GB after the site's own template migration. The site's content is
not redistributable.

---

## E-009 — The generalisation

**Claim.** On this platform, filesystem metadata operations do not parallelise;
they anti-parallelise. Bulk data throughput is the disk's business, and the loop
around it is not what costs.

**Class.** `interpretation`

**Source.** E-002, E-003, E-004, E-005 and E-006 — three independent
experiments across three different phases. Stated in
`docs/performance/OPTIMIZATIONS.md` at the end of the PERF-007 entry, which is
where the program first wrote it down.

**Caveat.** This is a conclusion drawn from three results on one machine with
one filesystem, not a measurement of the filesystem itself. No microbenchmark of
APFS metadata concurrency was run, and none is offered. The mechanism given in
the paper — that metadata mutations in a directory serialise below the syscall —
is the program's reading of why, and it is consistent with all three results
without having been tested directly.

---

## E-010 — Why the comparisons are interleaved

**Claim.** A sequential "N runs of A, then N runs of B" comparison early in this
program produced a −69% result that was entirely thermal drift.

**Class.** `observed`

**Source.** `docs/performance/README.md`, in the statement of the rule.

---

## Claims deliberately not made

* **No claim that any of these changes is exactly zero.** Three of the results
  are "smaller than this session could resolve", and the paper says so; only
  PERF-004's parallel delete and PERF-007's parallel copy are claimed as
  negative.
* **No claim about Linux, Windows, other filesystems, or network storage.**
  Nothing was measured there, and the anti-parallelism claim is scoped to macOS
  on APFS in the paper's own wording.
* **No claim about how APFS is implemented.** The mechanism offered is an
  interpretation consistent with the three results, not a measurement of the
  filesystem.
* **No output-equivalence claim.** None of these changes landed, so none went
  through `scripts/perf/compare_output.py`.
* **No estimate of the engineering time these four cost.** It was not recorded,
  and inventing it to make the "measure first" point land harder would be the
  same error the paper is about.
* **No claim that the rule has since prevented a specific proposal.** It is
  written down in `OPTIMIZATIONS.md`; what it has caught since is not recorded.
