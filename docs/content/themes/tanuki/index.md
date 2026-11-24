
+++
title = "tanuki"
description = "A whimsical, Nintendo-inspired Zola theme with Catppuccin colors, Geist typography, and Lucide icons. Supports documentation (with versioning), e-books, blog/landing pages, and product landing pages. Named after the playful Japanese raccoon dog (タヌキ)."
template = "theme.html"
date = 2026-11-20T19:23:01+01:00

[taxonomies]
theme-tags = []

[extra]
created = 2026-11-20T19:23:01+01:00
updated = 2026-11-20T19:23:01+01:00
repository = "https://github.com/raskell-io/tanuki"
homepage = "https://github.com/raskell-io/tanuki"
minimum_version = "0.23.0"
license = "MIT"
demo = "https://tanuki.raskell.io"

[extra.author]
name = "Raffael Schneider"
homepage = "https://raskell.io"
+++        

<div align="center">

<h1 align="center">
  <img src="static/img/tanuki-icon.png" alt="tanuki mascot" width="96" />
  <br>
  Tanuki
</h1>

<p align="center">
  <em>An opinionated Zola theme for documentation, books, and blogs.</em><br>
  <em>Beautiful, accessible, and thoughtfully designed.</em>
</p>

<p align="center">
  <a href="https://www.getzola.org/">
    <img alt="Zola" src="https://img.shields.io/badge/Zola-0.23+-000000?logo=zola&logoColor=white&style=for-the-badge">
  </a>
  <a href="https://catppuccin.com/">
    <img alt="Catppuccin" src="https://img.shields.io/badge/Catppuccin-cba6f7?style=for-the-badge">
  </a>
  <a href="LICENSE">
    <img alt="License" src="https://img.shields.io/badge/License-MIT-a6e3a1?style=for-the-badge">
  </a>
</p>

<p align="center">
  <a href="https://tanuki.raskell.io">Live Demo</a> •
  <a href="https://tanuki.raskell.io/docs/">Documentation</a> •
  <a href="https://tanuki.raskell.io/book/">Book Example</a> •
  <a href="https://tanuki.raskell.io/blog/">Blog Example</a>
</p>

<hr />

</div>

![Tanuki Theme](screenshot.png)

## Features

- **Three Modes** — Documentation (with versioning), Book, and Blog layouts
- **Catppuccin Colors** — Soothing Mocha (dark) and Latte (light) palettes
- **Geist Typography** — Clean, readable variable fonts
- **Lucide Icons** — Crisp, consistent iconography
- **Resizable Sidebar** — Drag to resize, persists across sessions
- **Full-text Search** — Elasticlunr-powered instant search
- **Dark/Light Toggle** — Three-way toggle with system preference detection
- **Print Support** — Print all pages as a single document (docs/book modes)
- **Keyboard Navigation** — Arrow keys for prev/next, `/` for search
- **SEO & Accessibility** — JSON-LD structured data, ARIA landmarks, semantic HTML

## Requirements

Tanuki requires **Zola 0.23 or later** (Tera v2 templates, Giallo syntax
highlighting). On Zola 0.19 to 0.21, use the
[`zola-0.19-0.21`](https://github.com/raskell-io/tanuki/tree/zola-0.19-0.21)
tag instead:

```bash
git clone --branch zola-0.19-0.21 https://github.com/raskell-io/tanuki themes/tanuki
```

## Installation

```bash
cd your-zola-site
git clone https://github.com/raskell-io/tanuki themes/tanuki
```

Or as a git submodule:

```bash
git submodule add https://github.com/raskell-io/tanuki themes/tanuki
```

## Quick Start

### Documentation Mode

```toml
base_url = "https://docs.example.com"
title = "My Project Docs"
theme = "tanuki"
build_search_index = true

[markdown.highlighting]
theme = "catppuccin-mocha"
style = "class"
data_attr_position = "both"

[extra]
mode = "docs"
github = "https://github.com/you/project"

# Optional: version picker
[extra.versions]
current = "2.0.0"
list = [
    { version = "2.0.0", url = "/", label = "latest" },
    { version = "1.0.0", url = "/v1/" },
]
```

### Code-Block Validation (optional)

Code blocks always get a copy button and a language label. A site can also have
them validated in the browser by supplying a WASM validator:

```toml
[extra.code_validation]
wasm_module = "/wasm/my_validator.js"          # required to enable
language = "kdl"                               # default: "kdl"
playground_url = "https://example.com/play/"   # optional; adds a link on success
skip_paths = ["/reference/directives"]         # optional; pages of partial snippets
```

Blocks in `language` get a Validate button. On success, and only when
`playground_url` is set, a link appears that opens the snippet in that
playground.

The module is loaded lazily on first use and must export `default` (an init
function) and `validate`; `init_panic_hook` is called if exported. `validate`
should return `{ valid, errors, warnings }`. Ship it from your own site's
`static/`, not from the theme.

Without this section the theme never loads a validator and never adds a Validate
button.

### Book Mode

```toml
base_url = "https://book.example.com"
title = "The Complete Guide"
theme = "tanuki"
build_search_index = true

[markdown.highlighting]
theme = "catppuccin-mocha"
style = "class"
data_attr_position = "both"

[extra]
mode = "book"
github = "https://github.com/you/book"
```

### Blog Mode

```toml
base_url = "https://blog.example.com"
title = "My Blog"
theme = "tanuki"
generate_feeds = true

taxonomies = [
    { name = "tags", feed = true },
]

[markdown.highlighting]
theme = "catppuccin-mocha"
style = "class"
data_attr_position = "both"

[extra]
mode = "blog"

[extra.hero]
title = "Welcome to my blog"
subtitle = "Thoughts on code and craft"

[[extra.nav]]
name = "Blog"
url = "/blog/"

[[extra.nav]]
name = "About"
url = "/about/"
```

### Syntax Highlighting

With `style = "class"` in `[markdown.highlighting]`, Zola writes the theme CSS
into the output directory and Tanuki links it: `giallo.css` for a single
`theme`, or `giallo-light.css` / `giallo-dark.css` (by `prefers-color-scheme`)
when `light_theme` and `dark_theme` are set. With the default inline style no
stylesheet is needed. `data_attr_position = "both"` puts `data-lang` on the
`<pre>` as well, which the language label uses in blog mode. Theme names come
from [Giallo](https://github.com/getzola/giallo?tab=readme-ov-file#built-in).

## Components

The reusable pieces in `templates/macros/` are Tera components, available in
any template or in Markdown content without an import, for example:

```jinja
{{/*<posts.post_list pages={section.pages} limit={6} />*/}}
{{/*<ui.button text="Learn more" url="/about/" />*/}}
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `←` / `→` | Previous / Next page |
| `/` | Open search |
| `Esc` | Close overlays |

## Browser Support

Modern browsers (Chrome 88+, Firefox 78+, Safari 14+, Edge 88+)

## Credits

- [Catppuccin](https://catppuccin.com) — Color palette
- [Geist](https://vercel.com/font) — Typography
- [Lucide](https://lucide.dev) — Icons
- [Zola](https://www.getzola.org) — Static site generator

## License

[MIT](LICENSE)

---

<p align="center">Made with care by <a href="https://raskell.io">raskell.io</a></p>

        