# Zola Graph

**Status:** Shipped (`v0.23.2-curriculo.2`+; extraction quality in `.3`)  
**Home:** `src/cmd/graph/` in this fork

## Problem

Curriculo’s Zola fork maintains a **topical knowledge graph** (pages ↔ topics ↔
relations) that:

1. Bootstraps **once** from an existing site via sitemap + Firecrawl.
2. Emits **committed JSON** under the site tree (files-as-truth).
3. **Maintains** the KG from local markdown after that — never re-crawls with Firecrawl.

After migrate, **committed markdown under `content/**` is the source of truth**.
`graph refresh` and `zola build` only read local files.

## Non-goals

- Baking networked KG into `zola build` (build stays offline).
- Re-running Firecrawl on refresh or on every CI push.
- Google KG Search / DataForSEO / torch-GLiNER in v1.
- Pushing this code upstream to vs-stack.

## Hard rule: Firecrawl once

| Command | Firecrawl | OpenRouter | Reads |
|---------|-----------|------------|-------|
| `graph migrate` | **yes, once per origin** | yes | remote sitemap + HTML |
| `graph refresh` | **never** | yes (stale pages only) | local `content/**` + `data/graph/**` |
| `build` | never | never | local files only |

`migrate` refuses a second crawl if `data/graph/meta.json` already has
`source_origin` for that site unless `--force` (ops escape hatch only).

## CLI

```bash
zola --root <site> graph migrate --from https://example.com [--max N] [--force] [--dry-run]
zola --root <site> graph refresh [--max N] [--dry-run]
```

### Migrate behavior

1. Discover sitemap (`sitemap_index.xml` / `sitemap.xml`), collect leaf URLs.
2. Skip non-HTML assets (`.ico`, images, fonts, etc.) **before** applying `--max`.
3. Scrape each URL via Firecrawl (`onlyMainContent` + `excludeTags`).
4. Strip residual theme chrome deterministically (`src/cmd/graph/clean.rs`) —
   WordPress themes often ignore clean `<main>` semantics.
5. Front-matter `description` comes from Firecrawl metadata
   (`description` → `og:description` → `ogDescription`); falls back to a summary
   of the **cleaned** body only if metadata is empty.
6. Write `content/<path>/index.md` with `extra.source_url` + `extra.content_hash`.
7. Enrich topics via OpenRouter; write `data/graph/{pages,topics,relations,meta}.json`.

DIV-3: partial success is committed; the process still exits non-zero if any
page failed (e.g. scrape timeout).

### Refresh behavior

Re-hash local default-language markdown; re-topic only stale/new pages (capped
by `--max`). Never calls Firecrawl. Updates `meta.last_refresh`.

## Artifacts (committed)

Under the site root (landing: `zola/`):

```
data/graph/
  pages.json       # url, path, title, summary, content_hash, topic_ids
  topics.json      # id, label, aliases, page_ids
  relations.json   # {from, to, kind}  kind ∈ page_topic | topic_topic | page_page
  meta.json        # source_origin, migrated_at, schema_version, last_refresh
```

## Operator / test loop

```bash
zola graph migrate --from https://curriculo.me   # ONCE: Firecrawl + write content + data/graph
zola build --base-url https://curriculo-me.pages.dev/
# enrich_jsonld + ./scripts/parity/parity gates  (#79 bar — landing-website)
# …edit / add a blog in content/…
zola graph refresh                               # local markdown only → update KG
zola build --base-url https://curriculo-me.pages.dev/
# repeat last two forever
```

Deploy today is **`curriculo-me.pages.dev`**. Live `curriculo.me` cutover is Z-5
(founder-gated DNS), not this loop.

## Build bar (landing-website, PR #79)

Every master build must keep:

| Expectation | Gate |
|-------------|------|
| Valid JSON-LD on every page | `check_jsonld.py` |
| Content depth ≥800 chars in `<main>` (with exempts) | `check_content_depth.py` |
| Layout shell for sitemap URLs | `sitemap_parity.py` |
| vs-stack finders clean (canonical / orphan / **duplicate** / …) | `check_finders.py` |

Migrate may add content, but **must not** ship a build that fails these gates.
Exact duplicates (e.g. curated `/blogs/*` twins of scraped legacy paths) must be
retired — scraped/legacy URLs win when both exist.

## Secrets

| Secret | Source | Used by |
|--------|--------|---------|
| `OPENROUTER_API_KEY` | GH Actions (landing) + agenix | migrate topics + every refresh |
| `FIRECRAWL_API_KEY` | agenix → landing GH secret | **`migrate` only** |

## GH Actions (landing-website)

- **`workflow_dispatch` `graph-migrate.yml`:** run **once** (or `--force` remigrate);
  commits content + graph. Sole Firecrawl entrypoint.
- **`master.yml` graph path:** `graph refresh` → `zola build` → enrich → **parity
  gates** → deploy to Cloudflare Pages. Translate is currently commented out so
  it cannot starve the single self-hosted runner; restore later with
  `needs: [deploy-curriculo]`.

## Implementation map

| Path | Role |
|------|------|
| `src/cmd/graph/mod.rs` | Subcommand wiring |
| `src/cmd/graph/schema.rs` | Load/save `data/graph` |
| `src/cmd/graph/sitemap.rs` | Sitemap discover + parse |
| `src/cmd/graph/firecrawl.rs` | Migrate-only scraper |
| `src/cmd/graph/clean.rs` | Boilerplate strip |
| `src/cmd/graph/migrate.rs` | One-shot migrate |
| `src/cmd/graph/refresh.rs` | Local KG refresh |
| `src/cmd/graph/topics.rs` / `openrouter.rs` | Topic enrichment |
