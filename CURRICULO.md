# Curriculo Zola fork

This is Curriculo’s fork of [getzola/zola](https://github.com/getzola/zola). Upstream
behavior is unchanged; we add two subcommands used by `curriculo-tech/landing-website`
(ADR-008 files-as-truth: content lives as committed markdown under the site root).

| Release pin | What it ships |
|---|---|
| `v0.23.2-curriculo.1` | `zola translate` |
| `v0.23.2-curriculo.2` | `zola graph migrate` / `graph refresh` |
| `v0.23.2-curriculo.3` | Clean migrate extraction (metadata description, boilerplate strip, asset URL skip) |

Landing CI pins the binary via `ZOLA_VERSION` / `ZOLA_BIN_URL` (never `latest`).

## Commands

### `zola translate`

Generate/refresh co-located `index.<lang>.md` siblings via OpenRouter
(`openai/gpt-4o-mini`). Hash-gated on `extra.source_hash`. Needs
`OPENROUTER_API_KEY`.

```bash
zola --root <site> translate [--max N] [--dry-run]
```

### `zola graph`

Topical knowledge graph: pages ↔ topics ↔ relations, committed as JSON under
`data/graph/`. See [docs/curriculo/graph.md](docs/curriculo/graph.md).

```bash
# ONCE per origin (Firecrawl + OpenRouter) — writes content/** + data/graph/**
zola --root <site> graph migrate --from https://curriculo.me [--max N] [--force] [--dry-run]

# Forever after (OpenRouter only) — updates data/graph from local markdown
zola --root <site> graph refresh [--max N] [--dry-run]
```

**Hard rule:** Firecrawl is migrate-only. `refresh` and `build` never crawl.

## Operator loop (curriculo.me → pages.dev)

```bash
zola graph migrate --from https://curriculo.me   # once
zola build --base-url https://curriculo-me.pages.dev/
# enrich_jsonld + parity gates (#79 bar) in landing-website
# …edit content/**…
zola graph refresh                               # local KG only
zola build --base-url https://curriculo-me.pages.dev/
# repeat refresh + build forever
```

Deploy target today is **`curriculo-me.pages.dev`** (Cloudflare Pages). Live
`curriculo.me` DNS cutover is a separate founder-gated step (Z-5).

## Secrets

| Secret | Used by |
|--------|---------|
| `OPENROUTER_API_KEY` | `translate`, `graph migrate`, `graph refresh` |
| `FIRECRAWL_API_KEY` | `graph migrate` only |

## Related

- Design: [docs/curriculo/graph.md](docs/curriculo/graph.md)
- Landing CI: `curriculo-tech/landing-website` (`.github/workflows/master.yml`, `graph-migrate.yml`)
- Workspace design notes: `~/dev_ws/c/docs/superpowers/specs/2026-08-11-zola-graph-design.md`
