# GLM BRIEF — fix `zola graph migrate` content extraction

You are the coding executor. The orchestrator plans and judges only and will NOT
write code. Do not stop until every Definition-of-Done box is checkable.

Worktree: `/home/dev/dev_ws/c/.wt-zola/migrate-quality`
Branch:   `fix/graph-migrate-extraction` (already checked out, based on `origin/master`)
Base for PR: `master` (this fork has **issues disabled** — no issue to link)

---

## What happened (the bug you are fixing)

`zola graph migrate` ran for real against `https://curriculo.me` and committed 29
pages to the landing-website repo. **All 29 are polluted with WordPress theme
chrome**, and 19 of 29 have that chrome in their front-matter `description`, so
they would ship meta descriptions like:

```
description = "Hit enter to search or ESC to closeSearch [Close Search](https://…/#) [Resume Tips](https://…)"
description = "![CurriculoATS](https://curriculo.me/wp-content/uploads/2026/04/curriculo-logo-144.webp)CurriculoATS [Features](https://curriculo.me/features/) [AI Screening](h"
```

Measured on the committed output (29 migrated files):

| marker | files |
|---|---|
| `wp-content` | 29 / 29 |
| `Hit enter to search` | 21 |
| `Close Search` | 21 |
| `litespeed` | 16 |
| chrome inside front-matter `description` | 19 |

The pollution also flows into the knowledge graph: `migrate.rs` passes the same
dirty markdown into `topics::enrich_one` and into `content_hash`, so topics were
extracted from nav/footer text too.

### Real polluted examples (read these — they are your fixtures)

Committed on `curriculo-tech/landing-website` `master`. A local checkout with
them already exists at
`/home/dev/dev_ws/c/.wt-landing-website/dupe-probe/zola/content/`:

- `ai-resume-builder/blogs/how-ats-works-2026/index.md`
- `ai-resume-builder/blogs/resume-summary-examples-2026/index.md`
- `ai-ats-for-founders/index.md`

Observed chrome classes, **leading**:

- logo + brand: `![CurriculoATS](…/curriculo-logo-144.webp)CurriculoATS`
- one-line nav link runs: `[Features](…) [AI Screening](…) [Impact Scoring](…) …` (15+ links)
- auth CTAs: `[Log in](https://ats.curriculo.me/sign-in) [Start Free](…/sign-up)`
- bare breadcrumb words on their own line, e.g. `Blog`
- search widget: `Hit enter to search or ESC to close`, `Search`, `[Close Search](…#)`
- category link runs: `[ATS Optimization](…/category/…) [Resume Tips](…/category/…)`
- author/avatar block: `![Dev](…/wp-content/litespeed/avatar/….jpg?ver=…)[Dev](…/author/bill/)May 19, 2026`

…and **trailing**:

- footer/related link lists where link text is a mashed title+blurb:
  `- [AI Resume BuilderBuild an ATS-ready resume that gets past the filters.](…)`
- `[Close Menu](…#)`
- cookie banner: `We use cookies to improve your experience and analyze site traffic. [Privacy Policy](…)` followed by `RejectAccept`

Root cause: `firecrawl.rs` already sends `"onlyMainContent": true`, but this
WordPress theme lacks clean `<main>`/`<article>` semantics so Firecrawl returns
essentially the whole body. Do **not** assume the flag will start working.

---

## Tasks

### T1 — front-matter `description` must come from source metadata

`src/cmd/graph/migrate.rs` currently does:

```rust
d = summarize(&fetched.markdown),
```

which is why the body's first junk lines become the meta description.

- Add `pub description: String` to `FetchedPage` in `src/cmd/graph/firecrawl.rs`.
- Populate it from the Firecrawl response, first non-empty of:
  `data.metadata.description`, `data.metadata["og:description"]`,
  `data.metadata.ogDescription`.
- In `migrate.rs`, use `fetched.description` when non-empty; otherwise fall back
  to `summarize(<cleaned body>)` (cleaned, per T2 — never the raw body).
- Trim/collapse whitespace; keep it single-line so the TOML stays valid.
- Also pass it as `TopicInput.description` (currently hardcoded `String::new()`).

### T2 — strip theme boilerplate from the fetched markdown

Add a dedicated, well-tested module (e.g. `src/cmd/graph/clean.rs`) exposing
something like `pub fn strip_boilerplate(md: &str) -> String`.

Use **two independent layers** (defense in depth) — do not rely on either alone:

1. **Ask Firecrawl for less.** In `firecrawl.rs`, keep `onlyMainContent: true`
   and additionally send `excludeTags` for structural chrome
   (`nav`, `header`, `footer`, `aside`, `form`, `script`, `style`, `noscript`,
   plus obvious theme selectors you can justify). Keep the payload readable.
2. **Deterministic trim** in `clean.rs`, applied to whatever markdown comes back:
   - Drop leading chrome: iterate from the top and remove lines that are
     link-only runs, image-only lines, known widget strings, bare breadcrumb
     words, or the author/avatar/date line — stopping at the first line of real
     prose or the first `#`/`##` heading.
   - Truncate trailing chrome at the first footer marker (cookie banner,
     `Close Menu`, a run of mashed-title link list items).
   - Remove the author/avatar line anywhere it appears
     (`wp-content/litespeed/avatar`).

**Conservatism is the hard requirement:** it must never eat real article prose.
Prefer leaving a stray line in over truncating content. Every rule you add needs
a test proving it does not damage a real body.

### T3 — skip non-HTML sitemap URLs

The live run hard-failed on `https://ats.curriculo.me/favicon.ico`
(`SCRAPE_UNSUPPORTED_FILE_ERROR`, Firecrawl HTTP 500), which counted as a
failure and tripped the DIV-3 fail-through for an otherwise fine run.

- Filter asset URLs out of the crawl scope **before** fetching (extension check:
  `.ico .png .jpg .jpeg .gif .webp .svg .avif .css .js .json .xml .pdf .zip
  .mp4 .webm .woff .woff2 .ttf`, case-insensitive, ignoring any query string).
- Apply the filter **before** the `--max` cap so the cap yields N real pages.
- Log the skipped count; skipped URLs are **not** failures.

### T4 — use the cleaned body everywhere

In `migrate.rs`, clean once then use that value consistently for: the file body
written by `write_page`, `content_hash`, `Page.summary`, and `TopicInput.body`.
No call site may still see the raw markdown.

### T5 — regression guard

Add a test that fails if known chrome markers survive end-to-end: feed a
realistic polluted fixture (copy a real body from the files listed above into
`tests/` or an inline `const`) through the migrate write path with a
`MockFetcher`, then assert the written `index.md` contains none of
`Hit enter to search`, `Close Search`, `Close Menu`, `RejectAccept`,
`wp-content/litespeed`, and that `description` is the metadata description.

---

## Constraints

- **TDD**: write the failing test first, then the fix. Small, frequent commits.
- **No network in tests.** Use the existing `MockFetcher` pattern; extend it for
  the new `description` field.
- `cargo test` must pass. Keep `cargo fmt` clean and do not add new clippy warnings.
- Follow existing style in `src/cmd/graph/*` (module doc comments, the `ponytail:`
  convention for noting known ceilings, `errors::Result`).
- Do **not** touch `refresh.rs`'s hard rule: Firecrawl must never be used on refresh.
- Do **not** change the graph JSON schema or `SCHEMA_VERSION`.
- Scope is this repo only. Do not edit landing-website content; retiring the
  duplicate curated pages and the re-migrate are the orchestrator's job.

## Definition of Done

- [ ] T1–T5 implemented with tests, `cargo test` green.
- [ ] `cargo fmt --check` clean; no new clippy warnings.
- [ ] Branch pushed.
- [ ] PR opened against `master` describing each fix and showing before/after of
      a `description` field and a cleaned body excerpt.
- [ ] PR body notes that the orchestrator must cut a new release tag and bump
      `ZOLA_VERSION` / `ZOLA_BIN_URL` in landing-website before the re-migrate.
