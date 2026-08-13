# Paper backlog

Prospective papers. Ideas, not commitments — a topic earns a directory when the
evidence for it exists. Nothing here is a promise that the work will be done, and
nothing here should be cited as though it had been.

An idea graduates when someone can answer, from artifacts already in the
repository: *what was measured, against what baseline, and what did it show?*

## Ready when the work is

### Why `zola serve` held gigabytes, and what the serving architecture became

`PERF-016` is only half addressed. Compressing the in-memory map and letting
`--store-html` serve from disk both landed and are measured; **render-on-demand
is not built**. A paper here needs that work, or an honest account of why the
cheaper fixes were enough.

Type: `architecture`, `performance-study`. Depends on: PERF-016.

### Correct incremental rebuild semantics

`zola serve --fast` re-renders the changed page and nothing that embeds it:
section listings, taxonomy terms, feeds and the sitemap stay stale until a full
rebuild. `docs/performance/INCREMENTAL-BUILD-DESIGN.md` enumerates the
invalidation rules; none of them are implemented. The paper is worth writing
once they are, because the interesting content is the correctness gate — an
incremental rebuild that disagrees with a clean build is a bug generator.

Type: `architecture`, `design-proposal`.

### What failed

A collection paper. This program has rejected four attractive optimizations on
measurement: caching created directories, parallel output cleaning, the
rename-aside output clean, and parallelising the static copy. Three of the four
failed the same way, which is itself the finding: on this platform, filesystem
metadata operations do not parallelise, they anti-parallelise.

Type: `negative-result`. Evidence: `docs/performance/OPTIMIZATIONS.md`.

### Scaling behaviour before and after an incremental architecture

Requires the architecture to exist. Today's answer is in `SCALING.md`: nothing in
the build is superlinear, so scaling work is about constants, not exponents.

Type: `performance-study`.

## Architectural research direction — not implemented

Recorded here so the idea is not lost and, more importantly, so nobody mistakes
it for something that exists.

**Content-addressed intermediate artifacts with a dependency DAG.** The shape:

```
content-addressed intermediate artifacts
+ dependency DAG
+ reverse dependency index
+ precise transitive invalidation
+ persistent reusable artifact cache
+ clean build as the correctness oracle
```

and, potentially, an evolution of that into a **Merkle DAG**, in which a node's
identity is derived from:

```
local semantic inputs
+ hashes of its dependencies
+ the configuration that reaches it
+ a build-semantics version
```

so that an unchanged subtree is provably reusable across runs and across
machines, and a changed input invalidates exactly the transitive closure that
depends on it.

**Status: design hypothesis.** No implementation, no prototype, no measurements.
It is motivated by two things this program did measure — that a rebuild's cost is
dominated by producing output that mostly did not change, and that the
long-running `serve` path has an entirely different cost structure from the batch
build — but motivation is not evidence. Any paper touching it must classify it as
`proposal` and must not attach predicted numbers to it as if they had been
observed.
