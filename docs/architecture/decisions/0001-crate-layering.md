# 0001 — The workspace is a layered DAG, and it is enforced

Status: accepted
Date: 2026-08-11

## Context

The workspace has twelve crates under `components/` plus the binary. The
layering has always been real — `errors` and `console` are leaves, `site`
depends on almost everything — but it existed only as prose. Two consequences
followed.

First, one edge is load-bearing and easy to break by accident: `content` must
not depend on `markdown`. `components/site/src/md_render.rs` exists for exactly
this reason — it renders markdown on behalf of content so that `content` stays
renderer-agnostic. Nothing prevented a future change from adding the dependency
and deleting that seam by accident.

Second, the prose description had drifted from the manifests. It described
`markdown`, `templates`, `render` and `search` as one layer above `content`,
when in fact `render` sits between them: `markdown` depends on `render`, and
`render` depends on `content`. Anyone reasoning about where to put new code from
the description alone would have got it wrong.

## Decision

The crate graph is a DAG with a computed layering, and both are checked
mechanically. `docs/architecture/COMPONENTS.md` is generated from the crate
manifests plus one-line responsibilities in `scripts/dev/components.toml`, so
the documented graph is the actual graph by construction. Forbidden edges are
declared in the same file with a reason each, and violating one fails the check.

## Alternatives

**Prose only, reviewed by humans.** What we had. It drifted within a few
releases and gave no signal when it did.

**A general layering rule ("a crate may only depend on strictly lower
layers").** Rejected: the layer numbers are *derived* from the dependencies, so
the rule is vacuous — it can never fail. The meaningful invariants are named
edges and acyclicity.

**A third-party architecture-linting crate.** Rejected as a dependency and a
build-time cost for a check that is thirty lines of manifest reading.

## Consequences

* Adding a dependency between components changes `COMPONENTS.md`; that shows up
  in review as a deliberate line in the diff rather than an invisible edge.
* Adding a new crate requires an entry in `scripts/dev/components.toml`. The
  check names the missing entry and what to write.
* Removing a forbidden edge from the rules is possible — it is one line — but it
  must be argued in the commit message, which is the point.
* The generated document has to be regenerated when manifests change, and CI
  fails when it is not. That is a small, recurring cost paid to keep the map
  true.

## Evidence

The drift is checkable: `render` appears in `components/markdown/Cargo.toml`
dependencies, and `content` in `components/render/Cargo.toml`, which is
incompatible with the previous flat description. The generated layering places
`content` at 3, `render` at 4, `markdown` at 5.

## Enforcement

```bash
scripts/dev.sh map        # invariants + drift
scripts/dev.sh generate   # regenerate after a manifest change
```

Failures name the offending edge, its declared reason, and the file to edit.
Also run by `scripts/dev.sh quality-full` and by CI.
