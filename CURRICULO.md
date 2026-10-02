# Curriculo Zola fork

This is Curriculo’s fork of [getzola/zola](https://github.com/getzola/zola). Upstream
behavior is unchanged; we add two subcommands used by `curriculo-tech/landing-website`
(ADR-008 files-as-truth: content lives as committed markdown under the site root).

| Release pin | What it ships |
|---|---|
| `v0.23.2-curriculo.1` | `zola translate` |
| `v0.23.2-curriculo.2` | `zola graph migrate` / `graph refresh` |
| `v0.23.2-curriculo.3` | Clean migrate extraction (metadata description, boilerplate strip, asset URL skip) |
| `v0.23.2-curriculo.11` | `zola serve --graph-refresh` — KG refresh after every content rebuild |

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
zola --root <site> graph migrate --from https://example.com [--max N] [--force] [--dry-run]

# Forever after (OpenRouter only) — updates data/graph from local markdown
zola --root <site> graph refresh [--max N] [--dry-run]

# Opt-in: same refresh automatically after every content rebuild while serving
zola --root <site> serve --graph-refresh
```

**Hard rule:** Firecrawl is migrate-only. `refresh` and `build` never crawl.
`serve --graph-refresh` reuses `refresh` (local markdown + OpenRouter for
stale pages); it never crawls, and `zola build` stays offline.

## Operator loop

Identity (org, pillars, forbidden related-pairs) lives in the site's
`config.toml` `[extra.graph]`, not in this binary.

```bash
zola graph migrate --from https://example.com   # once
zola graph refresh
zola build --base-url https://example.com/
```

`zola build` stays offline. Firecrawl is migrate-only.

## Secrets

| Secret | Used by |
|--------|---------|
| `OPENROUTER_API_KEY` | `translate`, `graph migrate`, `graph refresh` |
| `FIRECRAWL_API_KEY` | `graph migrate` only |

## Related

- Design: [docs/curriculo/graph.md](docs/curriculo/graph.md)
- Landing CI: `curriculo-tech/landing-website` (`.github/workflows/master.yml`, `graph-migrate.yml`)
- Workspace design notes: `~/dev_ws/c/docs/superpowers/specs/2026-08-11-zola-graph-design.md`
