# Publishing an engineering paper

For turning completed work into a technical article in `docs/papers/`. The
rules a paper must satisfy are in `docs/papers/METHODOLOGY.md`; this is the
procedure for a session that writes one.

Papers are for findings worth a reader's time: a completed `PERF-*` epic, an
architectural discovery, a correctness bug with a story, a negative result that
saves someone the experiment. Not for every commit.

## 0. Know what you are not doing

A paper is not a performance report. `docs/performance/**` is where the
programme's state and evidence live and it stays authoritative. The paper
*consumes* it. If while writing you discover a finding that is not recorded in
`docs/performance/`, stop and record it there first — the paper cannot be the
only place a fact exists.

## 1. Gather the evidence before writing a sentence

Read, in this order:

```bash
docs/performance/STATUS.md          # what is done, open, rejected
docs/performance/HOTSPOTS.md        # the backlog and its evidence
docs/performance/OPTIMIZATIONS.md   # what landed, what was rejected, with numbers
ls benchmarks/results/**            # the artifacts you may cite
git log --oneline                   # what actually changed, and when
```

Then decide what the evidence supports. This is the step that gets skipped, and
skipping it produces a paper that has to be retracted in pieces.

If a number you want does not exist in an artifact or a recorded observation,
you have found either an experiment to run or a sentence to delete. Both are
fine. Inventing the number is not.

## 2. Scaffold

```bash
scripts/dev.sh papers new --title "..." --type performance-study --perf PERF-012 PERF-016
```

This allocates the next `ZPERF-NNN`, creates the directory, and writes skeletons
for `paper.md`, `evidence.md`, `data/measurements.toml` and the derivatives.

## 3. Declare the figures first, then write around them

Fill `data/measurements.toml` before `paper.md`. Every number the paper will
print gets an entry:

* from a committed benchmark artifact → give `source` and `json_path`, and the
  validator re-extracts and compares it;
* from a terminal or a profiler → give `source_note` describing exactly how it
  was obtained, and class it `observed`.

`text` is the exact string the paper prints. Writing this file first stops the
usual failure, which is prose that rounds a number and then propagates the
rounded version into three other files.

## 4. Write `paper.md`

Structure and voice: `docs/papers/STYLE.md`. The parts most often got wrong:

* **Baseline.** Name the commit and say what it is. "Before" is not a baseline.
* **Costs next to benefits.** Every change has a cost; if you did not find it,
  you did not look.
* **Unresolved means unresolved.** If the harness said the rounds disagreed on
  the sign, the paper says so and does not quote the median as a result.
* **Upstream vs this fork.** Never attribute the fork's behaviour to upstream.
  A bug demonstrated against upstream names the baseline binary used.
* **Future work is future.** No predicted numbers presented as observations.

## 5. Figures

Declare them in `figures/figures.toml`, then:

```bash
scripts/dev.sh papers figures zperf-NNN-<slug>
```

Generated from committed artifacts, never drawn. A chart whose numbers cannot be
regenerated does not go in.

## 6. Derivatives

`social/linkedin.md`, `social/short.md`, `social/thread.md`, derived from the
paper. They select and compress; they never introduce a figure. The validator
fails on any number in a derivative that does not appear in `paper.md`.

## 7. Validate and review

```bash
scripts/dev.sh papers validate
```

Then walk `docs/papers/CHECKLIST.md` — the editorial half is not mechanisable.

Assistant-authored papers are welcome in this fork and may be published here.
`status = "published"` is earned by the evidence: validation passing, the
checklist genuinely walked, and the paper read end to end. If a quantitative
claim is still unverified, leave it at `review` and say which one.

These artifacts are not upstream-bound: the upstream project does not accept
LLM-written documentation, so a paper never becomes part of an upstream pull
request.

## 8. When the numbers later change

In this order, no shortcuts:

```
update the artifact in benchmarks/results/
  → update data/measurements.toml
    → scripts/dev.sh papers figures <paper>
      → audit the prose that cites the changed figures
        → regenerate the affected social derivatives
          → scripts/dev.sh papers validate
```

Patching the four copies by hand is how a published paper ends up disagreeing
with itself.

## Commit shape

Atomic, as everywhere else in this repository:

```
docs(papers): add ZPERF-NNN, <subject>
docs(papers): social derivatives for ZPERF-NNN
```

Raw artifacts a paper cites belong in `benchmarks/results/`, committed
separately from the paper that interprets them.
