# Quality gates

One question — *is this branch healthy?* — and one answer per tier.

| Tier | Command | Runs | Use |
| ---- | ------- | ---- | --- |
| fast | `scripts/dev.sh check` | `cargo fmt --check`, `cargo check --workspace --all-targets` | while editing |
| gate | `scripts/dev.sh quality` | fast, plus the clippy ratchet and `cargo test --workspace` | before every commit, and before saying a change works |
| full | `scripts/dev.sh quality-full` | gate, plus generated-file drift, tooling tests, change-impact report | before opening a PR |
| perf | `scripts/dev.sh perf ...` | benchmark and output equivalence | for anything in `docs/performance/HOTSPOTS.md` |

CI runs `cargo fmt --check`, `cargo build --all` and `cargo test --all` on six
targets, plus the lint and drift checks in `.github/workflows/engineering.yml`.
`scripts/dev.sh quality-full` is the local superset.

## The clippy ratchet

The workspace is not clippy-clean — there are 67 pre-existing warnings, most of
them in test code. `-D warnings` would therefore fail on an untouched checkout,
which is how a lint gate ends up being switched off. Instead
`scripts/dev/clippy_gate.py` compares the count per lint against
`scripts/dev/clippy-baseline.json`:

| Situation | Result |
| --------- | ------ |
| a lint that is not in the baseline | fail — you introduced it |
| an existing lint, more frequent | fail — you added occurrences |
| an existing lint, less frequent | fail — bank it with `scripts/dev.sh clippy --update` |
| unchanged | pass |

`scripts/dev.sh clippy --list` shows the current counts. A toolchain upgrade
that adds lints will fail here on purpose: read what it found, then update the
baseline in its own commit.

## Required validation by risk

`scripts/dev.sh impact` classifies the working tree and prints this.

| Risk | What it covers | Required |
| ---- | -------------- | -------- |
| LOW | documentation, tooling | `scripts/dev.sh check` |
| MEDIUM | internal implementation, no observable change | `scripts/dev.sh quality` |
| HIGH | rendering, URLs, parsing, config, template API, CLI | `quality` + a test that fails without the change |
| CRITICAL | output queue, render cache, build pipeline order, output cleaning | the above + byte-for-byte output equivalence |

The point of the table is not ceremony. It is that the four CRITICAL files
decide what ends up on disk, and a mistake there is invisible to `cargo test`.

## Environment note

A global `~/.cargo/config.toml` that sets `rustflags` (`lto`, `panic`,
`target-cpu`) leaks into every build in this repository: clippy can fail to
compile the workspace, and release timings stop being comparable.
`scripts/dev.sh` and `scripts/perf/build.sh` clear `RUSTFLAGS` for that reason.
`scripts/dev.sh doctor` reports whether yours does this.

## Optional git hooks

Off by default. `scripts/dev.sh hooks install` points `core.hooksPath` at
`.githooks/`, where `pre-commit` runs the fast tier and the generated-file
check. It never runs tests or benchmarks. `scripts/dev.sh hooks uninstall`
reverts it.
