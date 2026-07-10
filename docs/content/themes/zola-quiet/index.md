+++
title = "zola-quiet"
description = "A quiet, three-skin Zola theme: Minima-flavoured typography, monospace terminal, and a TUI/ncurses look with ASCII article frame. Every skin ships with matched light and dark modes plus a runtime toggle. Docs-style on-page TOC, addendum convention, iframe theme-sync protocol baked in."
template = "theme.html"
date = 2026-07-10T00:00:00+05:30

[taxonomies]
theme-tags = []

[extra]
created = 2026-07-10T00:00:00+05:30
updated = 2026-07-10T00:00:00+05:30
repository = "https://github.com/johnnybravo-xyz/zola-quiet.git"
homepage = "https://github.com/johnnybravo-xyz/zola-quiet"
minimum_version = "0.19.0"
license = "MIT"
demo = "https://johnnybravo.xyz"

[extra.author]
name = "Ritesh Shrivastav"
homepage = "https://johnnybravo.xyz"
+++

# zola-quiet

A quiet, three-skin Zola theme. Three stylesheets ship together — a
Minima-flavoured typographic skin, a monospace terminal skin, and a
TUI/ncurses skin with an ASCII-drawn article frame. Each ships with
a matched light and dark mode, and two runtime toggles in the
top-right flip between them (skin cycle + light/dark). Both choices
persist to `localStorage`. Posts with headings get a docs-style "on
this page" TOC pinned under the sidebar on the terminal + tui skins,
with scroll-spy for the active section.

No frameworks, no fonts hot-linked from a CDN, no analytics, no
search, no comments. Static HTML out of Zola, three CSS files, two
tiny inline scripts.

## Install

```bash
cd your-site
git submodule add https://github.com/johnnybravo-xyz/zola-quiet themes/zola-quiet
```

Then in your `config.toml`:

```toml
theme = "zola-quiet"
```

## `[extra]` keys the templates read

All optional. Every block is `{% if %}`-guarded so the theme
degrades cleanly if a key isn't set.

| Key                         | Effect |
|-----------------------------|--------|
| `extra.author`              | Name in `<meta>` and footer copyright line |
| `extra.github`              | Username; GitHub icon in the footer |
| `extra.linkedin`            | Username; LinkedIn icon in the footer |
| `extra.email`               | Address; mail icon in the footer |
| `extra.ascii_signature`     | Multi-line ASCII shown as a quiet signature above the content |
| `extra.homepage_post_limit` | Cap the front-page post list to N most recent posts |

## Shareable heading anchors

Set `insert_anchor_links = "left"` on a page (or section) and the
theme's `anchor-link.html` override renders each heading anchor as
a dim `>` prefix — subtle by default, coloured on hover.

## License

MIT — see the [repository](https://github.com/johnnybravo-xyz/zola-quiet).
