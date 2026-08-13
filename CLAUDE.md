# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

`AGENTS.md` states what you are required to do. This file describes how the code
is arranged. Longer procedures live in `.claude/workflows/`; the tooling behind
them is `scripts/dev.sh`, which is plain bash and works without an agent.

## Project

Zola is a static site generator shipped as a single Rust binary. The repo is a Cargo workspace:
the `zola` binary lives in `src/`, all the logic lives in `components/*` crates.

Edition 2024. Version is set once in `[workspace.package]` of the root `Cargo.toml`.

## Commands

```bash
scripts/dev.sh doctor                  # what this machine can do
scripts/dev.sh check                   # fast: fmt --check + cargo check
scripts/dev.sh quality                 # the gate: fmt + clippy ratchet + tests
scripts/dev.sh quality-full            # + generated-file drift + tooling tests
scripts/dev.sh impact                  # changed components, risk class, docs affected
scripts/dev.sh clippy --list           # current lint debt (ratchet baseline)
scripts/dev.sh generate                # rewrite generated documents
scripts/dev.sh perf <cmd>              # forwards to scripts/perf/run.sh
scripts/dev.sh papers <cmd>            # engineering papers: validate | index | new | figures
```

```bash
zola build --timings                   # per-phase breakdown of a build (developer diagnostic)
cargo build --release --features alloc-stats   # + per-phase allocation counts in that report
```

The underlying cargo commands, which is what CI runs:

```bash
cargo build --all                      # what CI builds
cargo test --all                       # what CI runs
cargo fmt --check                      # CI fails on this; rustfmt.toml sets use_small_heuristics = "max"

cargo test -p site                     # one crate
cargo test -p site --test site         # one integration test target (tests/site.rs)
cargo test -p site --test site can_parse_site   # one test
cargo insta review                     # review changed snapshots (markdown/templates use insta)

cargo run -- --root test_site build    # run the binary against a fixture site
cargo run -- --root test_site serve
cargo bench -p markdown                # criterion; site benches need `python components/site/benches/gen.py` first
```

Optional features (forwarded from the root crate to `search`): `indexing-zh`, `indexing-ja`.

Note on this machine class: a global `~/.cargo/config.toml` that sets
`rustflags` (`lto`, `panic`, `target-cpu`) leaks into every build here — clippy
can fail to compile the workspace and release timings stop being comparable.
`scripts/dev.sh` and `scripts/perf/build.sh` clear `RUSTFLAGS`; `scripts/dev.sh
doctor` tells you whether yours does this.

Docs (`docs/`) are a Zola site built by CI with a released Zola; edit content there, don't
regenerate it from this checkout.

## Contributing rules that affect PRs

- Development targets the `next` branch. Only documentation fixes for the *current* release go to `master`.
- User-visible changes are listed in `CHANGELOG.md`, grouped per release with a `### Breaking` section.
- Syntax highlighting languages/themes live in the separate [Giallo](https://github.com/getzola/giallo) crate, not here.
  This fork currently builds against `vendor/giallo`, a patched copy pulled in with
  `[patch.crates-io]`; see `vendor/README.md` for what the patch does and how to drop it.
- `CONTRIBUTING.md` explicitly restricts LLM usage: generated code must be human-reviewed and tested,
  and LLM-written documentation is not accepted.

## Architecture

`docs/architecture/COMPONENTS.md` is generated from the crate manifests and is
the authoritative map: layer, responsibility, dependencies, dependents, test and
bench targets, and which components carry open `PERF-*` items.

### Crate layering

`errors` and `console` are leaves. Above them: `utils` → `config` → `content` →
`render` → (`markdown`, `search`) → `templates` → `site` → the binary, with
`imageproc` and `link_checker` sitting beside `content` on top of `config`.

A crate must not depend on something above it. Two edges are forbidden outright
and checked by `scripts/dev.sh map`: `content` → `markdown` (content stays
renderer-agnostic; `site/src/md_render.rs` bridges them) and `config` →
`content`.

### Build pipeline (`components/site/src/lib.rs`)

1. `Site::new` — read `zola.toml`/`config.toml`, merge `theme.toml` `[extra]`, build the Tera instance.
2. `Site::load` — walk `content/`, parse sections serially (dirs first, so a drafted section can
   `skip_current_dir`), parse pages in parallel with rayon, then populate sections/taxonomies and
   fill `Library`.
3. `Site::render_markdown` — per page/section: optional Tera templating of the raw markdown, then
   `markdown::render_content`.
4. `Site::rebuild_cache` — pre-serialize everything into `RenderCache`.
5. `Site::build` — Sass, search index, highlighting CSS, then `Queue::full_build(...).process()`,
   then image processing, then copying `static/`.

### Key types

- `content::Library` — the whole site in memory: `pages`/`sections` keyed by absolute `PathBuf`,
  plus `translations`, `backlinks`, `reverse_aliases`, `colocated_assets`, taxonomy definitions.
  Pages are keyed by on-disk path; `FileInfo::canonical` (path minus language code) is what links
  translations of the same content together.
- `render::RenderCache` — pages/sections/taxonomies/config pre-serialized to `tera::Value` once,
  so rendering N pages doesn't re-serialize the library N times. Anything added to what templates
  can see must be threaded through here, not just through the `Serializing*` structs.
  A container (section, taxonomy term) must **not** be serialized with its children inside it:
  `Value::from_serializable` walks a structure through serde and rebuilds every `Value` it finds,
  which used to materialise each page once per section and once per taxonomy term it belonged to.
  Serialize the container with an empty placeholder and splice the existing `Arc`-backed values in
  (`replace_entry` in `render/src/cache.rs`) — that was 86% of the peak heap.
- `render::Renderer` — the only place that calls `tera.render`. Missing `page.html`/`section.html`/
  taxonomy templates fall back to `render/src/default_tpl.html` instead of erroring.
- `site::queue` — the output layer. Every artifact (page, section, paginated page, taxonomy list/term,
  feed, sitemap, alias, 404, robots) becomes a `Job`; `process()` runs them with rayon and writes
  each `RenderedOutput` to disk and/or into `SITE_CONTENT`.
- `site::BuildMode` + `SITE_CONTENT` — `zola build` writes to disk (`Disk`); `zola serve` keeps
  HTML/XML in the `SITE_CONTENT` global map (`Memory`) and only writes assets, unless `--store-html`
  (`Both`).

### Templating and content

Shortcodes were removed in 0.23. Markdown content is now run through Tera before being parsed
(`site/src/md_render.rs`), guarded by a `memchr` scan for `{{`/`{%` and skippable per-file via the
`skip_content_templating` config globset. Tera macros/components in content therefore need
`{% raw %}`.

Template lookup uses Tera 2 fallback prefixes set in `templates::load_tera`: site `templates/` wins
over `<theme>/templates/` which wins over `__zola_builtins/` (built-in 404, feeds, sitemap,
robots.txt, anchor-link, summary-cutoff — all `include_str!`'d into the binary).

Tera functions/filters live in `components/templates/src/{functions,filters}`; they are registered
twice — in `ZOLA_TERA` (validation-only defaults) and again with real site data in
`site::tpls::{register_early_global_fns, register_tera_global_fns}` (the early pass runs before
markdown rendering, since content templating needs those functions). A new function needs both
registrations.

### Serve mode

`src/cmd/serve.rs` is an axum server plus a `notify` debouncer. `src/fs_utils.rs` classifies each
filesystem event into a `ChangeKind` (Content/Templates/Themes/StaticFiles/Sass/Config/ExtraPath)
and `serve.rs` matches on it to pick the cheapest rebuild. `--fast` restricts content changes to
`Queue::single_page`/`single_section` instead of a full rebuild. A config change rebuilds the whole
`Site`. Live reload is a WebSocket; build errors are injected into responses by
`error_injection_middleware`.

## Conventions

- Dependencies are declared once in the root `[workspace.dependencies]`; component `Cargo.toml`s use
  `{ workspace = true }`.
- Errors: `errors::{Result, anyhow, bail, Context}` (thin anyhow re-export). Add context with
  `.with_context(|| ...)` naming the offending file — error messages are the main UX of the CLI.
- Filesystem access goes through `fs_err as fs` (or `utils::fs`) so IO errors mention the path.
- Hash maps in hot paths are `ahash::AHashMap`; parallelism is rayon, not tokio (tokio is only for
  the serve HTTP server).
- **Anything templates iterate must have a stable order.** `tera` is built with `preserve_order`
  and `PageFrontMatter.taxonomies` is a `BTreeMap`, because a `HashMap` anywhere on that path makes
  two runs of the same binary emit different HTML. There is a test for it:
  `render::cache::tests::taxonomies_serialize_in_a_stable_order`.
- CLI flags are defined in `src/cli.rs`, which `build.rs` `include!`s to generate man pages and shell
  completions — keep it free of anything that can't be compiled standalone.

## Tests

- Fixture sites live at the repo root: `test_site`, `test_site_i18n`, `test_sites_invalid`.
  `components/site/tests/common.rs` resolves them via `../..` from the crate dir and builds into a
  `tempdir`, with `file_exists!`/`file_contains!` macros for assertions.
- `tests/site.rs` asserts exact counts (`library.pages.len() == 41`, section counts, etc.), so adding
  or removing a file in `test_site` will break unrelated tests — update the counts deliberately.
- `markdown` and `templates` use `insta` snapshots under `tests/snapshots/`.
- `templates` uses `mockito` for `load_data` HTTP tests.

## Working rules

These exist because each one has cost a session here.

- **Read before editing.** Including the parts of the file you are not changing.
- **Do not guess the architecture.** `docs/architecture/COMPONENTS.md` and the
  pipeline above are cheap to check.
- **Do not declare success without running the gate** in the tree's current
  state. A gate you did not run is reported as "not run".
- **Do not optimise without a benchmark.** `docs/performance/HOTSPOTS.md`
  records several places that look expensive and are not.
- **Do not change observable behaviour silently.** Output bytes, URLs, error
  messages and their order, and template-visible structure are all observable.
- **Do not hand-edit generated output.** `docs/architecture/COMPONENTS.md`,
  `docs/performance/STATUS.md`, man pages and completions come from generators.
- **Do not mix unrelated refactors** into a change that must be reviewed for
  output equivalence.
- **Do not leave temporary files behind.** Generated benchmark sites, profiler
  output and scratch scripts are not commits.
- **Check documentation impact.** `scripts/dev.sh impact` lists the documents
  that describe what you changed. Saying "no documentation change needed" is
  fine; ignoring it is not.

## Publishing

A completed `PERF-*` epic, an architectural discovery or a correctness bug with a
story may become a paper in `docs/papers/`. Papers consume performance reports;
they never replace them, and a fact must exist in `docs/performance/` before a
paper cites it. **Never duplicate a benchmark number by hand across paper, figure
and social post** — declare it once in the paper's `data/measurements.toml`,
which `scripts/dev.sh papers validate` checks against the artifact it came from.
Procedure: `.claude/workflows/publication.md`. Assistant-authored papers are
welcome here and may be published; what gates `published` is validation passing
and the checklist walked, not who wrote it. Nothing in `docs/papers/` is
upstream-bound — upstream does not accept LLM-written documentation.

## Performance work

There is an active program: `docs/performance/README.md` for the harness and
the measured baseline, `docs/performance/HOTSPOTS.md` for the evidence-backed
`PERF-*` backlog, `docs/performance/STATUS.md` for its state.

The rules that make a result believable — measure first, release builds,
interleaved A/B, byte-identical output, one hotspot per change, report memory,
record negative results — are in `.claude/workflows/performance.md`. Read it
before touching anything in the backlog.

`docs/performance/FINAL-REPORT.md` answers where the time goes today.
`OPTIMIZATIONS.md` is append-only and includes the **rejected** experiments —
caching created directories and both ways of speeding up the output clean —
which is the part worth reading before re-proposing an obvious idea.

## Where to look

| Need | Path |
| ---- | ---- |
| what a crate owns | `docs/architecture/COMPONENTS.md` |
| why an architectural choice was made | `docs/architecture/decisions/` |
| what is slow, with evidence | `docs/performance/HOTSPOTS.md` |
| session, investigation, implementation, quality, performance, publication procedures | `.claude/workflows/` |
| published technical narratives built from that evidence | `docs/papers/` |
| the tooling itself | `scripts/dev.sh`, `scripts/dev/`, `scripts/perf/` |
</content>
