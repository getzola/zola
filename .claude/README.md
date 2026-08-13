# `.claude/` — engineering support for coding agents

Nothing in this directory is required to build, test or contribute to Zola.
`cargo build --all` and `cargo test --all` remain the whole story for an
ordinary contributor. This is an acceleration layer: it exists so that an agent
(or a returning human) starts a session oriented instead of guessing.

The rules that apply to *everyone* live in `AGENTS.md` and `CLAUDE.md` at the
repository root. This directory holds the longer procedures those two files
point at.

## Layout

| Path | Contents |
| ---- | -------- |
| `commands/` | Claude Code slash commands. Each is a thin wrapper over a workflow. |
| `workflows/` | The procedures themselves. Readable as plain documentation. |
| `templates/` | Starting points for a session record and a backlog item. |
| `context/` | Ephemeral session state. **Gitignored** — never committed. |

## Workflows

| File | Use when |
| ---- | -------- |
| `workflows/session.md` | starting, running or ending a work session |
| `workflows/investigate.md` | you do not yet know why something behaves as it does |
| `workflows/implement.md` | you know what to change and need it to land safely |
| `workflows/performance.md` | anything touching a `PERF-*` item or build cost |
| `workflows/quality.md` | proving a branch is healthy |
| `workflows/publication.md` | turning completed work into a paper in `docs/papers/` |

## Commands

| Command | Does |
| ------- | ---- |
| `/session-start` | collect repository state, orient, write the objective |
| `/session-end` | evidence, honest status, promote durable findings, archive |
| `/investigate` | map, trace, read the tests, measure, report findings |
| `/implement` | plan at file level, change one thing, run the required gates |
| `/quality` | run a gate tier and report each result truthfully |
| `/perf` | work a `PERF-*` item under the measurement doctrine |
| `/paper` | write or update an engineering paper, evidence first |

## Tooling this points at

Everything is under `scripts/`, is plain bash and Python 3.11+ (for `tomllib`),
and works without an agent. Anything that needs Python degrades to a skip with
an explanation, never a traceback:

```
scripts/dev.sh doctor          what this machine can do
scripts/dev.sh check           format + type check
scripts/dev.sh quality         format + lint + tests
scripts/dev.sh quality-full    the above + drift checks + tooling tests
scripts/dev.sh generate        rewrite generated documents
scripts/dev.sh impact          changed components, risk class, documentation impact
scripts/dev.sh clippy          lint ratchet: --list current debt, --update to bank a win
scripts/dev.sh map             workspace map + architecture invariants
scripts/dev.sh perf-index      PERF-* backlog integrity
scripts/dev.sh test-tooling    tests for the scripts above
scripts/dev.sh session ...     start | show | end
scripts/dev.sh hooks ...       install | uninstall | status  (opt-in)
scripts/dev.sh perf ...        forwards to scripts/perf/run.sh
scripts/dev.sh papers ...      engineering papers: validate | index | new | figures
```

## Generated files

Never edit these by hand; edit the source and run `scripts/dev.sh generate`.

| Generated | Source | Generator |
| --------- | ------ | --------- |
| `docs/architecture/COMPONENTS.md` | crate manifests + `scripts/dev/components.toml` | `scripts/dev/repo_map.py` |
| `docs/performance/STATUS.md` | `docs/performance/HOTSPOTS.md`, `OPTIMIZATIONS.md` | `scripts/dev/perf_index.py` |
| `docs/papers/INDEX.md` | each paper's `metadata.toml` | `scripts/papers/papers.py` |
| `docs/papers/*/figures/*.svg` | `figures.toml` + benchmark artifacts | `scripts/papers/figures.py` |
| man pages, shell completions | `src/cli.rs` | `build.rs`, at build time |

CI fails if a generated file is out of date.

## Maintaining this directory

* A change to `scripts/dev.sh`'s command list means updating the table above and
  the one in `CLAUDE.md`. `scripts/dev.sh impact` reminds you.
* A new workflow gets a row in the tables above.
* Changing any script under `scripts/dev/` requires `scripts/dev.sh test-tooling`
  to still pass; add a case for whatever behaviour you added.
