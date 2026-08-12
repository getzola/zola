# zola (né Gutenberg) <img src="docs/static/logos/Zola-logo-main-coffee.svg" align="right" alt="zola logo" width="30%"/>

[![Build Status](https://dev.azure.com/getzola/zola/_apis/build/status/getzola.zola?branchName=master)](https://dev.azure.com/getzola/zola/_build/latest?definitionId=1&branchName=master)
![GitHub all releases](https://img.shields.io/github/downloads/getzola/zola/total)

A fast static site generator in a single binary with everything built-in.

---

## Curriculo fork

This is Curriculo's fork of [getzola/zola](https://github.com/getzola/zola).
Upstream behaviour is unchanged — we add two subcommands that back
`curriculo.me` (`curriculo-tech/landing-website`, ADR-008 files-as-truth:
content is committed markdown under the site root, no CMS).

| Command | What it does | Network |
|---|---|---|
| `zola translate [--max N] [--dry-run]` | Generate/refresh co-located `index.<lang>.md` siblings via OpenRouter, hash-gated on `extra.source_hash`. | OpenRouter |
| `zola graph migrate --from <url> [--max N] [--force] [--dry-run]` | **Once per origin.** Firecrawl-crawl a live site into markdown + a topical KG under `data/graph/`. Guarded by `meta.source_origin`. | Firecrawl + OpenRouter |
| `zola graph refresh [--max N] [--dry-run]` | **Forever after.** Re-topic stale local markdown into `data/graph/`. Never crawls. | OpenRouter |

**Hard rule:** Firecrawl is `migrate`-only. `refresh` and `build` never crawl.

```bash
# one-time bootstrap from a live site
zola --root <site> graph migrate --from https://curriculo.me

# steady state, after editing content/**
zola --root <site> graph refresh
zola --root <site> build --base-url https://curriculo-me.pages.dev/
# consumers then run enrich_jsonld + parity gates (see landing-website)
```

Secrets: `OPENROUTER_API_KEY` (translate + graph), `FIRECRAWL_API_KEY`
(`graph migrate` only). Consumers pin the binary by release tag
(`v0.23.2-curriculo.N`) via `ZOLA_VERSION` / `ZOLA_BIN_URL` — never `latest`.

Full docs: **[CURRICULO.md](CURRICULO.md)** ·
[docs/curriculo/graph.md](docs/curriculo/graph.md) ·
[CLI reference](docs/content/documentation/getting-started/cli-usage.md).
Issues are disabled — propose changes as PRs against `master`, then cut a new
`v0.23.2-curriculo.N` release for consumers to pin.

---

To find out more see the [Zola Documentation](https://www.getzola.org/documentation/getting-started/overview/), look
in the [docs/content](docs/content) folder of this repository or visit the [Zola community forum](https://zola.discourse.group).

This tool and its template engine [tera](https://keats.github.io/tera/) were born from an intense dislike of the (insane) Golang template engine and therefore of
Hugo that I was using before for 6+ sites.

## List of features

- [Single binary](https://www.getzola.org/documentation/getting-started/cli-usage/)
- [Syntax highlighting](https://www.getzola.org/documentation/content/syntax-highlighting/)
- [Sass compilation](https://www.getzola.org/documentation/content/sass/)
- Assets co-location
- [Multilingual site support](https://www.getzola.org/documentation/content/multilingual/) (Basic currently)
- [Image processing](https://www.getzola.org/documentation/content/image-processing/)
- [Themes](https://www.getzola.org/documentation/themes/overview/)
- [Shortcodes](https://www.getzola.org/documentation/content/shortcodes/)
- [Internal links](https://www.getzola.org/documentation/content/linking/)
- [External link checker](https://www.getzola.org/documentation/getting-started/cli-usage/#check)
- [Table of contents automatic generation](https://www.getzola.org/documentation/content/table-of-contents/)
- Automatic header anchors
- [Aliases](https://www.getzola.org/documentation/content/page/#front-matter)
- [Pagination](https://www.getzola.org/documentation/templates/pagination/)
- [Custom taxonomies](https://www.getzola.org/documentation/templates/taxonomies/)
- [Search with no servers or any third parties involved](https://www.getzola.org/documentation/content/search/)
- [Live reload](https://www.getzola.org/documentation/getting-started/cli-usage/#serve)
- Deploy on many platforms easily: [Netlify](https://www.getzola.org/documentation/deployment/netlify/), [Vercel](https://www.getzola.org/documentation/deployment/vercel/), [Cloudflare Pages](https://www.getzola.org/documentation/deployment/cloudflare-pages/), etc

## License

This project contains code under multiple licenses.

Code introduced after version 0.22 is licensed under the EUPL-1.2.
Code that existed prior to commit 3c9131db0d203640b6d5619ca1f75ce1e0d49d8f remains licensed under the MIT License, including in later versions of the project.

See LICENSE and LICENSE-MIT for details.
