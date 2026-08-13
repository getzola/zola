# `docs/papers/` — engineering papers

Publication-ready technical narratives built from evidence that already exists
in this repository. One paper per finding worth reading about: a completed
`PERF-*` epic, an architectural discovery, a correctness bug with a story, a
negative result that saves someone else the experiment.

The series is **Zola at Scale**. It documents work done in *this fork*. It is
not affiliated with, endorsed by, or speaking for the upstream Zola project, and
every paper is required to say which behaviour is upstream's and which is ours.

## This is not where performance work is recorded

That distinction is the whole point of the directory, so it is worth being blunt
about it:

| Path | Holds | Is |
| ---- | ----- | -- |
| `benchmarks/results/**` | benchmark JSON, A/B artifacts | **measurement truth** |
| `docs/performance/**` | HOTSPOTS, OPTIMIZATIONS, BASELINE, … | engineering interpretation and live project state |
| `docs/papers/**` | papers and their derivatives | **publication narrative** |
| `scripts/perf/**` | the measurement harness | how truth is produced |
| `scripts/papers/**` | scaffolding, validation, figures | how papers are kept honest |

The dependency runs one way. A paper consumes performance reports; a performance
report never cites a paper; neither ever edits a benchmark artifact to agree
with prose. If a number in a paper disagrees with the JSON, the JSON is right.

## Layout of a paper

```
docs/papers/zperf-001-<slug>/
  metadata.toml     identity, status, type, related PERF items
  paper.md          the canonical text — the only authored narrative
  evidence.md       every significant claim, its class, and where it comes from
  data/
    measurements.toml   canonical figures, machine-checked against benchmark JSON
  figures/          generated SVG, never hand-drawn
  social/
    linkedin.md     long-form social derivative
    short.md        one-paragraph summary
    thread.md       sequential posts
```

`paper.md` is the single source of narrative truth. Everything in `social/` is a
derivative: it may select and compress, never restate a number the paper does
not contain. The validator enforces exactly that.

## Lifecycle

```
idea → draft → review → published → superseded
```

* **idea** — a row in `BACKLOG.md`, no directory yet.
* **draft** — being written; numbers may still move.
* **review** — content complete, awaiting the human author's review.
* **published** — a human has read it, stands by it, and it has gone out.
* **superseded** — later evidence replaced it; the paper stays, with a pointer.

`CONTRIBUTING.md` requires that a human review, edit, test and stand by anything
an assistant produces, and does not accept assistant-written *documentation*.
Papers are neither upstream documentation nor code: they are this fork's
publication artifacts. The rule still binds. **A paper written by an agent stays
at `review` until a human has read it and moved it to `published`.** No agent
sets `status = "published"`.

## Commands

```bash
scripts/dev.sh papers new --title "..." --type performance-study   # scaffold
scripts/dev.sh papers validate                                     # the gate
scripts/dev.sh papers index                                        # rewrite INDEX.md
scripts/dev.sh papers figures zperf-001-<slug>                     # regenerate figures
```

`validate` is what CI and `scripts/dev.sh quality-full` care about. It checks
metadata, identity uniqueness, that referenced `PERF-*` items and files exist,
that every declared figure matches the benchmark JSON it claims to come from,
that no number appears in a social derivative without appearing in the paper,
and that no local absolute path or placeholder marker survived.

## Writing one

Read, in this order:

1. `METHODOLOGY.md` — what may be claimed, and how a claim is classified.
2. `STYLE.md` — voice, structure, what not to write.
3. `CHECKLIST.md` — the review gate; run through it before `review`.
4. `.claude/workflows/publication.md` — the procedure for an agent session.

The short version: gather the evidence first, decide what it actually supports,
and only then write. If you find yourself wanting a number that no artifact
contains, you have found either an experiment to run or a sentence to delete.
