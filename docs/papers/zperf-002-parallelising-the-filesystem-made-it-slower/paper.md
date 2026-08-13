# Parallelising the Filesystem Made It Slower

**Four optimizations rejected on measurement, and the platform rule three of them found**

> Zola at Scale, ZPERF-002. Status: **review**, 2026-08-13.
>
> This work was done in a fork of [Zola](https://github.com/getzola/zola). It is
> not affiliated with, endorsed by, or speaking for the upstream project. None of
> the changes described here shipped — in this fork or anywhere else. They were
> implemented, measured, and reverted.

## Abstract

A performance program on a fork of the static-site generator Zola produced a
ranked backlog of hotspots from profiles and phase timings. Four of those items
were then rejected, three of them after being built and measured, because the
numbers did not support them. Three were filesystem work: remembering which
output directories a build has already created instead of calling
`create_dir_all` per file (PERF-003),
parallelising the recursive delete of the previous output (PERF-004), and
parallelising the copy of the `static/` tree (PERF-007). All three failed, and
the three failures have one shape. Parallel deletion was slower on two of three
phase samples and on all three whole-build samples. The parallel static copy was
slower in every round on the workload it should have won — 5000 files of 1 KB,
where per-file syscall latency dominates — by 10–30%. Directory caching, in two
variants, moved nothing at all. The fourth rejection (PERF-009) is different in
kind: the item was correct when it was written and was invalidated by a
re-profile after an earlier fix landed. The generalisation the first three
support, on this platform and this filesystem, is that metadata operations do
not parallelise; they anti-parallelise. Bulk throughput belongs to the disk, and
the loop around it is not what costs.

## Context

Zola builds a static site from Markdown and Tera templates. The work described
here happened in a fork carrying a long-running performance program, whose
evidence lives in `docs/performance/`: a hotspot inventory of numbered `PERF-*`
items, an append-only optimization log that records the failures alongside the
wins, and benchmark artifacts under `benchmarks/results/`. The wins from that
program are the subject of a separate paper
([ZPERF-001](../zperf-001-faster-without-computing-less/paper.md)). This one is
about the four items that were tried and rejected.

Two workloads recur below:

* **Synthetic fixtures** generated deterministically from a seed
  (`scripts/perf/gen_site.py`), each isolating one cost. `simple-pages-1000` and
  `simple-pages-4000` isolate the per-page floor; `mixed-realistic-*` is a
  profile calibrated against the real site; `markdown-heavy-4000` is CPU-bound.
* **A reference proxy** — the real motivating site's content with substitute
  templates (`scripts/perf/make_proxy_site.py`), 3776 pages and 1640 sections.
  Its output tree was 6544 files / 73 MB when the earliest of these experiments
  ran and about 9 GB after the site's own template migration, so the output-clean
  experiments below were measured against very different amounts of output.

**Machine.** Apple M4 Pro, 12 cores, 24 GiB, macOS 26.2, APFS on the internal
SSD. Release builds with the profile pinned by `scripts/perf/build.sh`. Every
number here is from that one machine, and the conclusion is explicitly scoped to
it.

## Methodology

Each experiment compared the tree immediately before the change against the same
tree with the change applied, as two binaries, **alternated within each round**.
The program had already learned why that is not optional: an early sequential
comparison — all runs of A, then all runs of B — produced a −69% result that was
entirely thermal drift.

Three of these four effects are too small to resolve in whole-build wall time on
a laptop, so the primary instrument was the phase accumulator printed by
`zola build --timings`: `out: write file`, `clean output dir`, `copy static`.
Whole-build wall time is reported next to it in every case, because a phase that
gets faster while the build does not is exactly the failure mode two of these
experiments hit.

**These figures are transcribed phase timings, not committed benchmark JSON.**
The A/B harness writes artifacts under `benchmarks/results/` and those exist for
the program's accepted changes; for a rejected experiment the record is the
numbers copied into `docs/performance/OPTIMIZATIONS.md` at the time, with the
code reverted afterwards. Every experimental figure below is therefore classed
`rejected` or `observed` in this paper's
[measurement manifest](data/measurements.toml) and never `measured`, which in
this series means re-extractable from a committed artifact: a reader cannot pull
these out of a file, only re-implement the experiment that produced them. The
only `measured` figures here are the reference workload's input counts.

No output-equivalence gate was run for any of these four, because none of them
landed.

## Experiment 1 — caching the directories a build has already created

**Hypothesis.** `write_output` calls `fs::create_dir_all(parent)` before every
`fs::write`, and each call walks and re-creates the whole parent chain, so a
four-level site issues four `mkdir` syscalls per page and all but the first fail
with `EEXIST`. The CPU profile attributed **18.9% of busy CPU** on
`simple-pages-1000` to `create_dir_all`, and **1237 ms** of `mkdir` CPU on
`mixed-realistic-4000`. Remembering the directories already created should
remove nearly all of those syscalls.

**What was built.** An `Arc<Mutex<AHashSet<PathBuf>>>` on the output queue,
checked before `create_dir_all` and updated after, with the lock held only for
the hash lookup.

**Result — nothing.** `out: write file` CPU on `simple-pages-4000`, interleaved:

| round | without the cache | with it |
| ----- | ----------------- | ------- |
| 1 | 8.84 s | 7.00 s |
| 2 | 7.71 s | 7.67 s |
| 3 | 7.71 s | 7.98 s |
| median | 7.71 s | 7.67 s |

Whole-build wall time was indistinguishable: **1.31 s against 1.39 s, with a
1.03–1.37 s spread on the same binary**. Round 1 looks like a win, neither later
round reproduces any part of it, and the two medians — 7.71 s and 7.67 s — differ
by far less than one round of noise.

**Second variant, also rejected.** A thread-local set — no shared lock, a few
duplicate `mkdir` calls across workers — was implemented later, once two other
fixes had made the write path the largest remaining item in the build:
`out: write file` was then **7.0 s of CPU across 4804 writes, 1.4 ms each**. The
result was **7.079 s against 7.107 s** of phase CPU and **1.68 s against 1.65 s**
of wall time, medians of three interleaved rounds.

**Why.** The cost in the write path is creating and writing the file, not the
redundant `mkdir`. On APFS a `mkdir` that returns `EEXIST` is cheap enough that
removing it is unmeasurable, and the mutex costs about what the skipped syscalls
save. The 18.9% profile attribution was counting samples parked in the kernel
while other workers waited, which overstates the share of *wall* time that path
can return.

## Experiment 2 — parallelising the output clean, then moving it aside

**Hypothesis.** `clean_site_output_folder` deletes the previous output with a
single-threaded recursive delete before anything else runs, so it overlaps with
nothing. It measured **663 ms — 36.3% of wall** on the reference proxy at
6544 files / 73 MB, and **1.3 s** once that site's output grew to about 9 GB. Its
top-level entries are independent subtrees, so deleting them with rayon should
shorten the phase.

**Result — slower.** `clean output dir`, interleaved; the first round of each
pair cleans an empty directory and is excluded:

| workload | serial | parallel |
| -------- | ------ | -------- |
| reference proxy | 3.889 s | 6.784 s |
| reference proxy | 3.457 s | 2.797 s |
| `mixed-realistic-8000` | 978.9 ms | 1.856 s |

Two of three phase samples were worse, and so were all three whole-build
samples: **26.5–33.8 s serial against 31.6–35.1 s parallel**. Concurrent
`unlink` storms contend on directory metadata rather than overlapping.

**Second variant — rename aside, delete in the background.** If the deletion
cannot be made faster, it can be taken off the critical path: rename the previous
output to a sibling scratch directory (one `rename`, whatever its size), delete
it on a background thread while the build runs, and join before reporting
success. This worked exactly as designed at the phase level — `clean output dir`
went from **929.8 ms → 0.2 ms**, and the join at the end cost 0.0 ms.

**And it changed wall time by nothing:**

| workload | before | after |
| -------- | ------ | ----- |
| reference proxy (9 GB output) | 24.42 s | 24.55 s |
| `mixed-realistic-8000` | 3.19 s | 3.33 s |
| `markdown-heavy-4000`, 8 rounds | 4.275 s | 4.350 s |

The last row is the case that should have shown the effect most clearly: the
clean is **301 ms of a 4.3 s build**, the build is CPU-bound, and the run spread
was tight, 4.22–4.40 s. Over eight interleaved rounds the median was **+1.8%**,
with a best round of **−0.7%**. The effect is absent.

**Why.** The deletion is not removed, only moved. The build already saturates
twelve cores and the same disk, so a background deleter competes with the
workers for precisely the resources they need. Winning would mean not waiting
for the deletion at all — detaching it from the process lifetime — which would
let `zola build` return while it is still writing to disk, and race with whatever
consumes the output next.

## Experiment 3 — parallelising the static copy

This is the one worth the reader's time, because it was run twice and the second
run is the informative one.

**Hypothesis.** `copy_directory` walks `static/` and copies one file at a time,
calling `metadata()` on source and destination for each. On the reference site
that tree is **989 files / 55 MB** and the phase costs **170–190 ms** of serial
wall time. Copying the files with rayon after the (serial) walk should turn that
into a fraction of itself.

**What was built.** Collect the walk into a list, create the directories
serially in walk order, then `par_iter()` the copies; collect every result and
report the first failure *in walk order*, so the error a user sees does not
depend on scheduling.

**Result on the real tree — nothing.** `copy static`, alternating binaries:
**191.0 / 196.6 / 211.0 ms serial against 193.6 / 211.6 / 170.9 ms parallel**.
That is not a surprise in hindsight: 55 MB in about 190 ms is **~290 MB/s**,
which is the disk's number, not the loop's.

So the experiment was repeated on the case parallelism *should* win — 5000 files
of 1 KB each, where per-file syscall latency dominates and throughput does not:

| round | serial | parallel |
| ----- | ------ | -------- |
| 1 | 640.1 ms | 837.6 ms |
| 2 | 813.6 ms | 899.0 ms |
| 3 | 841.9 ms | 1023 ms |

**Unanimously worse, by 10–30%.** Twelve threads creating files in the same
handful of directories contend on directory metadata, and each copy also probes
for its parent, so they hammer `exists()` on the same paths simultaneously. The
two distributions do overlap — the best parallel round, 837.6 ms, is faster than
the worst serial one, 841.9 ms — but the pairing is what decides it, and the
pairing is unanimous.

A `file_type()` micro-fix that removed one `stat` per entry was written
alongside the parallel copy and went back with the revert: the saving is real
and far below what this phase's noise can resolve, and carrying a change no
measurement supports is how a codebase accumulates performance folklore.

## Experiment 4 — the `stat` per `load_data` cache key

The fourth rejection is not a parallelism failure. It is a backlog item that was
true when it was written and stopped being true underneath itself.

`load_data` computes its cache key with `get_file_time()`, a `stat` per call —
**4843 calls** on the reference workload. The item was filed to be done *after*
the fix that stopped `load_data` holding its cache mutex across I/O and parsing,
on the reasoning that once the lock was gone this would be the remaining serial
syscall on that path.

Re-profiling after that fix landed says otherwise. **Every** `stat` in the whole
build is **439 ms** of self time, and `canonicalize` another **360 ms** —
together **0.9% of the build**, and this item is one caller inside that. The
timestamp it reads is also what stops the cache serving a data file that has
changed since it was loaded, so removing it trades a few milliseconds for a
correctness hazard. Closed as rejected without an implementation.

The general point is about backlogs rather than filesystems: **a ranked hotspot
list is a snapshot, and fixing its top item invalidates rows below it.**

## What the three failures have in common

Three independent experiments, three different phases, one shape:

| Experiment | The idea | What happened |
| ---------- | -------- | ------------- |
| PERF-003, both variants | skip redundant `mkdir` syscalls | no measurable change in phase CPU or wall time |
| PERF-004, parallel delete | delete independent subtrees concurrently | slower on 2 of 3 phase samples, 3 of 3 whole-build samples |
| PERF-004, rename aside | remove the phase from the critical path | phase gone (929.8 ms → 0.2 ms), wall time unchanged |
| PERF-007, parallel copy | overlap per-file syscall latency | 10–30% slower, unanimously, on the best case for it |

The program stated the generalisation once the third one landed, and it is the
reusable part of this paper:

> On this platform, filesystem metadata operations do not parallelise. They
> anti-parallelise. Bulk data throughput is the disk's business, and the loop
> around it is not what costs.

Three mechanisms, one conclusion. Metadata mutations in a directory serialise
somewhere below the syscall, so concurrent creators and concurrent unlinkers
queue instead of overlapping and pay synchronisation for the privilege. Where
the operation is bulk data instead, the serial loop was already at the device's
throughput and there was nothing to recover. And where the work is unavoidable,
moving it off the critical path of a build that already saturates every core and
the same disk conserves it exactly.

The practical rule the fork adopted from this: **a proposal of this shape needs a
measurement before it gets a review.** All three looked obviously correct on
paper — the rename-aside variant was explicitly approved as a design before it
was built — and all three were reverted.

## Limitations

* **One machine, one filesystem.** Everything here is an Apple M4 Pro on macOS
  with APFS — twelve cores and one SSD. A different filesystem, a different
  kernel, network storage where per-operation latency is orders of magnitude
  higher and concurrency hides it, or a machine with far more parallel I/O
  capability, may all behave differently. Nothing here was measured on Linux or
  Windows.
* **These are transcribed phase timings.** No committed JSON artifact backs the
  four experiments; the rejected code was reverted, so only the record remains.
  That is why the paper quotes round-level numbers rather than one summary
  statistic — the raw rounds are what was written down.
* **A negative result at this effect size is a bound, not a zero.** What the
  measurements support is that the directory caches are smaller than the noise
  floor of this machine, and that the parallel delete and the parallel copy are
  reliably negative. "Exactly zero" is not claimed anywhere.
* **`hard_link_static` still exists.** The static-copy phase remains serial, and
  a user who wants it cheaper has that option; the finding is that the loop is
  not what makes it cost.

## What surprised us

1. **The best case for parallelism was the worst result.** The static copy was
   re-run on 5000 tiny files precisely because that is where per-file latency
   should dominate and rayon should win. It lost by more there than anywhere
   else.
2. **A profile attribution of 18.9% returned nothing.** Samples parked in the
   kernel on a path are not the same as time that path can give back, and the
   distinction is invisible in a flat profile.
3. **Removing a phase from the timeline entirely did not make the build
   faster.** `929.8 ms → 0.2 ms` is as clean a phase-level win as this program
   produced, and the build took the same time.
4. **A backlog item can rot.** PERF-009 was correct when it was filed and wrong
   by the time it came up, because the fix ranked above it changed the profile
   it was ranked on.

## What was kept

No code. What was kept is the record: the rejected experiments written up with
their numbers in [`OPTIMIZATIONS.md`](../../performance/OPTIMIZATIONS.md), four
rows marked `rejected` in
[`HOTSPOTS.md`](../../performance/HOTSPOTS.md) rather than deleted, and the
platform rule quoted above.

An append-only log that includes failures is worth more than one that does not,
for a reason specific to performance work: the ideas that fail are attractive,
so they recur. Without the record, the next person to have the parallel-copy
idea has no way to know it was already measured, and the cost of finding out is
the whole experiment again.

## Reproduction

The rejected changes are not in the tree, so reproducing these results means
re-implementing them. What the repository does provide is the harness, the
fixtures, and the instrumentation that produced every number above:

```bash
# the benchmark binary, with a pinned release profile
scripts/perf/run.sh build

# fixtures used above (deterministic from a seed)
python3 scripts/perf/gen_site.py --scenario simple-pages --pages 4000
python3 scripts/perf/gen_site.py --scenario mixed-realistic --pages 8000
python3 scripts/perf/gen_site.py --scenario markdown-heavy --pages 4000

# the phase accumulators these experiments were judged on:
#   `clean output dir`, `copy static`, `out: write file`
cd benchmarks/sites/mixed-realistic-8000 && zola build --force --timings -o /tmp/out

# interleaved A/B of two binaries, paired per-round deltas
scripts/perf/run.sh ab /tmp/zola-BEFORE /tmp/zola-AFTER benchmarks/sites/markdown-heavy-4000

# the correctness gate any such change would have had to pass
scripts/perf/run.sh equivalence /tmp/zola-BEFORE /tmp/zola-AFTER
```

Two things are not reproducible from the harness alone. The `clean output dir`
phase is invisible on a first build into an empty directory, which is why the
benchmark harness deliberately pre-populates the output directory before timing
it; an experiment on that phase that does not do the same measures nothing. And
the small-file tree of Experiment 3 is not a fixture: the record keeps its shape
— 5000 files of 1 KB under a site's `static/` — and not the script that made it.

## Evidence index

Every claim in this paper, its class and where it came from:
[evidence.md](evidence.md). Every printed figure, with the method that produced
it: [data/measurements.toml](data/measurements.toml).
