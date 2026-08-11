# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Zola is a static site generator shipped as a single Rust binary. The repo is a Cargo workspace:
the `zola` binary lives in `src/`, all the logic lives in `components/*` crates.

Edition 2024. Version is set once in `[workspace.package]` of the root `Cargo.toml`.

## Commands

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

Docs (`docs/`) are a Zola site built by CI with a released Zola; edit content there, don't
regenerate it from this checkout.

## Contributing rules that affect PRs

- Development targets the `next` branch. Only documentation fixes for the *current* release go to `master`.
- User-visible changes are listed in `CHANGELOG.md`, grouped per release with a `### Breaking` section.
- Syntax highlighting languages/themes live in the separate [Giallo](https://github.com/getzola/giallo) crate, not here.
- `CONTRIBUTING.md` explicitly restricts LLM usage: generated code must be human-reviewed and tested,
  and LLM-written documentation is not accepted.

## Architecture

### Crate graph (bottom-up)

`errors` (re-export of `anyhow`) and `utils` → `config` → `content` → `markdown` / `templates` /
`render` / `search` / `imageproc` / `link_checker` → `site` → binary. `console` is stdout printing only.

A crate must not depend on something above it; `site/src/md_render.rs` exists solely so `content`
doesn't have to depend on `markdown`.

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
