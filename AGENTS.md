# AGENTS.md

Engineering expectations for coding agents working in this repository. Written
to be tool-neutral: it applies to any assistant, and every rule here is one a
human reviewer would apply too.

`CLAUDE.md` covers how the codebase is arranged. This file covers what you are
required to do. Longer procedures live in `.claude/workflows/`.

## Scope

Zola is a static site generator that ships as one binary. Its users run it in
CI and publish the bytes it writes. That makes output stability, URL stability
and error messages part of the product, not implementation detail.

Development targets the `next` branch; only documentation fixes for the current
release go to `master`. New features are discussed on the forum before a PR
exists — do not open one uninvited.

### Assistant-authored work

**In this fork, work written by an assistant is welcome, including
documentation and published papers.** The repository owner has said so
explicitly; that is their call to make about their fork, and it is the policy
here.

`CONTRIBUTING.md` is upstream's file and states upstream's position. This fork
does not send work there and has its own guard against an accidental merge, so
read that file as describing another project rather than as a rule binding this
one.

What authorship does not change: everything in this file about measuring before
optimizing, reporting failures, and not claiming a gate passed when it did not.
Those rules exist because they produce correct work, not because of who wrote
it.

## Before changing anything

1. Read the code you are about to change, including the parts you are not
   changing. Guessing at architecture from file names produces confident,
   wrong patches.
2. Find who owns the behaviour: `docs/architecture/COMPONENTS.md` (generated
   from the manifests) maps every crate to its responsibility and dependencies.
3. Read the tests that cover it. `components/site/tests/site.rs` asserts exact
   page and section counts against `test_site`; adding a fixture file breaks
   unrelated tests, and that is deliberate.
4. If the question is "why is this slow", measure before forming a plan.
   See `.claude/workflows/performance.md`.

## Source of truth

| Question | Answer lives in |
| -------- | --------------- |
| what a crate is for, what it depends on | `docs/architecture/COMPONENTS.md` (generated) |
| how a build executes, in order | `CLAUDE.md`, `docs/performance/ARCHITECTURE.md` |
| what is slow and why we think so | `docs/performance/HOTSPOTS.md` + its evidence documents |
| the state of the performance backlog | `docs/performance/STATUS.md` (generated) |
| why an architectural choice was made | `docs/architecture/decisions/` |
| what users are told | `docs/content/documentation/` |
| what we have published about this work, and its evidence | `docs/papers/` |

Do not restate these in new documents. Link to them.

## Validation

One command answers "is this healthy":

```bash
scripts/dev.sh quality        # fmt --check, the clippy ratchet, cargo test --workspace
```

The workspace has pre-existing clippy warnings, so the lint gate is a ratchet
rather than `-D warnings`: `scripts/dev/clippy-baseline.json` records the count
per lint, and the gate fails when a lint is new or more frequent than before.
If you *reduce* a count it also fails, asking you to bank the win with
`scripts/dev.sh clippy --update` — the number is only allowed to go down, and
only deliberately.

`scripts/dev.sh check` is the fast tier for use while editing;
`scripts/dev.sh quality-full` adds generated-file drift and tooling tests and is
what to run before opening a PR.

How much validation a change needs depends on what it touches.
`scripts/dev.sh impact` prints the classification and the required gates:

| Risk | Examples | Required |
| ---- | -------- | -------- |
| LOW | docs, tooling | fast tier |
| MEDIUM | internal implementation | `quality` |
| HIGH | rendering, URLs, parsing, config, template API, CLI | `quality` + a test that fails without the change |
| CRITICAL | output queue, render cache, pipeline order, output cleaning | the above + byte-for-byte output equivalence |

## Tests

* A bug fix needs a test that fails before it and passes after. If you cannot
  write one, say so explicitly and explain why.
* `markdown` and `templates` use `insta` snapshots. Review snapshot changes
  (`cargo insta review`); never accept them wholesale to make a run green.
* Do not weaken an assertion to make a test pass. If an assertion is wrong,
  say why in the commit message.

## Documentation

`scripts/dev.sh impact` lists the documents that describe the behaviour behind
each changed path. It is a reminder, not a verdict: a refactor under
`components/config/` may need no documentation at all — but say that, rather
than ignoring it silently.

User-visible changes belong in `CHANGELOG.md`, in the unreleased section, with a
`### Breaking` entry when applicable.

Documentation prose for the public site is written by humans. Point out what
needs updating instead of writing it.

## Performance changes

A performance claim without a reproducible measurement is not admissible. The
full doctrine is in `.claude/workflows/performance.md`; the parts that are not
negotiable:

* measure before and after, on a release build, interleaved;
* output must be byte-identical unless the change is about output;
* report peak RSS as well as time;
* one hotspot per commit;
* record the negative results too.

## Dependencies

Declared once in the root `[workspace.dependencies]`; component manifests use
`{ workspace = true }`. Adding a dependency to a static site generator that
people install as a single binary is a real cost — propose it, with the
alternative you rejected, before adding it.

## `unsafe`

There is effectively none. Do not introduce any without a comment that proves
soundness and a measurement that shows it is worth it. "It should be faster" is
not a measurement.

## Compatibility

Treat as public API: the CLI (`src/cli.rs`), the configuration file, the
template functions and filters, the front-matter format, and the shape of the
generated output tree. Changing any of them is a breaking change and needs a
`CHANGELOG.md` entry under `### Breaking`.

`src/cli.rs` is `include!`d by `build.rs` to generate man pages and shell
completions — keep it compilable standalone, and never hand-edit the generated
artifacts.

## Generated files

`docs/architecture/COMPONENTS.md` and `docs/performance/STATUS.md` are produced
by `scripts/dev.sh generate`. Man pages and completions are produced by the
build. Edit the source, regenerate, commit both. CI fails on drift.

## Commits

* Conventional format: `type(scope): description`. Performance work uses the
  backlog id: `perf(PERF-004): parallelise the output clean`.
* One logical change per commit. Do not fold a refactor into a behaviour change
  that has to be reviewed for output equivalence.
* Never `--no-verify` to get past a hook you have not read.
* Commit only when asked to.

## Reporting completion

Say what you ran and what it said. Specifically:

* a gate you did not run is reported as "not run", never as passing;
* a partial result is reported as partial, with the part that is missing named;
* a hypothesis that the evidence disproved is a result — report the evidence,
  do not keep trying until something passes;
* if you stopped because the change was growing past its scope, say where the
  boundary was.
