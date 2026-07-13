
+++
title = "volks-typo"
description = "Minimalist blog theme with bold industrial typography and an 8-point grid (Zola port of the Astro Volks-Typo theme)."
template = "theme.html"
date = 2026-07-10T11:28:41+02:00

[taxonomies]
theme-tags = ['blog', 'minimal', 'dark-mode', 'search', 'responsive', 'bauhaus']

[extra]
created = 2026-07-10T11:28:41+02:00
updated = 2026-07-10T11:28:41+02:00
repository = "https://gitlab.com/tisgoud/zola-volks-typo-theme"
homepage = "https://gitlab.com/tisgoud/zola-volks-typo-theme"
minimum_version = "0.19.0"
license = "MIT"
demo = "https://tisgoud.gitlab.io/zola-volks-typo-theme/"

[extra.author]
name = "tisgoud"
homepage = "https://tisgoud.nl"
+++        

# Volks-Typo

A Zola port of the Astro **Volks-Typo** theme: a blog theme with bold, industrial typography built on an 8-point grid. It pairs a sidebar layout with a monumental red accent rule, dark-mode support, client-side search, table of contents on posts, and full taxonomy (categories + tags) support.

- Original Astro theme: [Astro - Volks-Typo](https://astro.build/themes/details/volks-typo/) | [Github - Volks-Typo](https://github.com/jdrhyne/volks-typo)

[Demo](https://tisgoud.gitlab.io/zola-volks-typo-theme/)

![screenshot Volks-Typo theme](https://gitlab.com/tisgoud/zola-volks-typo-theme/-/raw/main/static/screenshots/screenshot.png)

## Features

- Bauhaus inspired design - Bauhaus functionalism meets monumental aesthetics.
- Home - Landing page.
- Dark Mode Toggle - Light, dark and automatic.
- Sidebar - Left (default), right or hidden.
- Banner post-image - Image in the blog-list.
- Blog-list with partial post-image - Post-image displayed in the blog-list (default on).
- Reading Time Display - Automatic calculation and display of estimated reading time.
- Table of Contents - Auto-generated TOC.
- Ultra-Minimalist - Clean, focused design with purposeful use of space.
- Responsive Layout - Mobile-first with elegant desktop sidebar.
- Minimal JavaScript - Essential JS only for enhanced features, ensuring fast performance.
- Full Blog Support - Categories, tags, archives, and recent posts.
- SEO Optimized - Built-in meta tags and structured data.
- Accessibility First - Semantic HTML and ARIA attributes.
- Self-Hosted Fonts - No external dependencies for privacy.
- Git-"Star" - From the original Astro theme, now configurable/optional.

## Screenshots

**Landing page**

![Landing page](https://gitlab.com/tisgoud/zola-volks-typo-theme/-/raw/main/static/screenshots/volks-typo-home.png)

**Sidebar left**

![Sidebar right](https://gitlab.com/tisgoud/zola-volks-typo-theme/-/raw/main/static/screenshots/volks-typo-blog-sidebar-left.png)

**Sidebar right**

![Sidebar right](https://gitlab.com/tisgoud/zola-volks-typo-theme/-/raw/main/static/screenshots/volks-typo-blog-sidebar-right.png)

## Install

The easiest way to install this theme is to clone this repository in the themes directory:

```shell
git clone https://gitlab.com/tisgoud/zola-volks-typo-theme.git
```

ro use it as a submodule:

```
git submodule add https://gitlab.com/tisgoud/zola-volks-typo-theme.git themes/volks-typo
```
then set volks-typo as your theme in config.toml.

```
theme = "volks-typo"
```

Enable it in your `config.toml`:

```toml
theme = "volks-typo"
```

Minimum Zola version: **0.19.0** (developed and tested on Zola 0.22.1).

## Required configuration

The theme relies on several built-in Zola features and a set of `[extra]` keys. A minimal working `config.toml`:

```toml
base_url = "https://example.com"
title = "volks-typo"   # site title — used in <title>, header alt, RSS channel
description = "A blog exploring the intersection of design, typography, and history"
compile_sass = true
build_search_index = true
theme = "volks-typo"
# Site-wide feed off; the blog section emits a blog-only feed at /blog/rss.xml.
generate_feeds = false
feed_filenames = ["rss.xml"]

taxonomies = [
  { name = "categories", feed = true },
  { name = "tags", feed = true },
]

[markdown]
highlight_code = true

[markdown.highlighting]
# Zola bundles "nord" (a dark theme close to the original's catppuccin-mocha).
theme = "nord"

[extra]
# Navigation — add / remove / reorder without touching theme files
main_menu = [
  { name = "Home", url = "/" },
  { name = "Blog", url = "/blog" },
  { name = "Categories", url = "/categories" },
  { name = "About", url = "/about" },
]

author_name = "Your Name"
author_bio = "Writer, designer, and explorer of aesthetic tensions between past and present."
# author_avatar = "/images/avatar.jpg"   # optional

# Social links (footer, About author bio, Contact). `icon` is a bundled icon in
# static/icons/ (github, gitlab, codeberg, x, instagram, linkedin, mastodon,
# email) or a custom path like "/icons/gitea.svg".
social_links = [
  { name = "Gitlab", icon = "gitlab", url = "https://gitlab.com/tisgoud/zola-volks-typo-theme/" },
  { name = "Mastodon", icon = "mastodon", url = "https://fosstodon.org/@you" },
  { name = "Email",    icon = "email",    url = "mailto:you@example.com" },
]
```

### `[extra]` keys read by the theme

Site title and description come from the **top-level** `title` / `description` keys
(used by `<title>`, header logo `alt`, meta description, and the RSS channel).

| Key | Required | Purpose |
| --- | --- | --- |
| `main_menu` | yes | Header/mobile navigation links (`{ name, url }`); edit to change the nav |
| `sidebar_position` | no | `left` (default) or `right`. A header button also lets visitors show/hide the sidebar (remembered per browser; disabled on pages without a sidebar) |
| `list_images` | no | Show each post's `[extra].image` as a cropped thumbnail above the title in post listings (blog, categories, tags). Default `true` |
| `favicon` | no | Favicon path (in `static/`). Defaults to the bundled `favicon.svg` |
| `logo` | no | Header logo image path (in `static/`). Omit to show the site title as text |
| `repo_url` | no | Repo URL for the header repository button. Omit to hide the button |
| `repo_icon` | no | Bundled icon name (`github`, `gitlab`, `codeberg`) or a custom SVG path in `static/`. Default `github` |
| `repo_label` | no | Repo button text. Default `Star` |
| `author_name` | yes | Author shown in sidebar / footer |
| `author_bio` | yes | Short bio in the sidebar |
| `author_avatar` | no | Optional sidebar avatar image path |
| `social_links` | no | Social links (`{ name, icon, url }`) shown in footer, About, and Contact. `icon` is a bundled name or a custom SVG path |

Bundled social/forge icons live in `static/icons/`: `github`, `gitlab`,
`codeberg`, `x`, `instagram`, `linkedin`, `mastodon`, `email`. Add your own by dropping an SVG in `static/icons/` (or anywhere) and referencing it by path.

Any omitted social key simply hides that link.

## Content structure

```
content/
├── _index.md                 # home page
├── about.md                  # template = "about.html"
├── contact.md                # template = "contact.html"
├── search/
│   └── _index.md             # search index data (required for search)
└── blog/
    ├── _index.md             # blog list section
    └── <post>.md             # individual posts
```

### Blog section — `content/blog/_index.md`

```toml
+++
title = "Blog"
sort_by = "date"
paginate_by = 5
template = "blog.html"
page_template = "page.html"
+++
```

### Post frontmatter (`+++` TOML)

```toml
+++
title = "Understanding Bauhaus"
date = 2024-01-15
description = "A short summary used for meta and feed."

[taxonomies]
categories = ["Design"]
tags = ["bauhaus", "history"]

[extra]
excerpt = "Optional excerpt shown in listings and the RSS feed."
author = "Your Name"          # optional, falls back to author_name
image = "/img/post.jpg"       # optional; path to a file in static/.
                              # Shown as a hero image on the post
                              # and used for OpenGraph/Twitter
                              # social-share previews.
+++
```

### About / Contact pages

Edit the page text directly in the markdown **body** — the templates render
`page.content`, so no template override is needed. The About page also shows the
author bio + social links (from `[extra]`), and Contact shows the social links;
those come from config, the prose comes from the markdown.

```md
+++
title = "About"
template = "about.html"   # or "contact.html"
+++

## About This Blog

Your prose here. Headings, lists, **bold**, links — all standard markdown.
```

### Search section — `content/search/_index.md`

Required so the search overlay has data to query:

```toml
+++
title = "Search"
template = "search_json.html"
+++
```

## Deviations from the Astro original

This is a faithful structural port. A few intentional differences:

1. **Syntax highlighting theme** — uses Zola's bundled `nord` theme. The original's
   `catppuccin-mocha` is not bundled with Zola; both are dark themes, so the visual
   match is close.
2. **No MDX** — Zola is Markdown-only. The original's MDX support and its `mdx-test`
   sample page were dropped.
3. **Search implementation** — uses a Zola-generated search index JSON with the
   original theme's own substring matching behavior (kept faithful), with
   `build_search_index = true` enabled.
4. **Images** — fixed post-images and added optional image display in the blog-list.

## Credits & licenses

- **Theme code** — MIT (see `LICENSE`).
- **Original theme** — a port of the Astro [Volks-Typo](https://github.com/jdrhyne/volks-typo)
  theme by Jonathan Rhyne (MIT). This Zola port keeps the design and much of the
  markup/CSS; credit for the original design belongs to the upstream project.
- **Fonts** (`static/fonts/`) — Oswald, Work Sans, JetBrains Mono
  ([SIL Open Font License 1.1](https://openfontlicense.org/)); Roboto Condensed
  (Apache License 2.0). Self-hosted woff2 subsets.
- **Icons** (`static/icons/`) — brand/forge marks from
  [Simple Icons](https://simpleicons.org/) (CC0 1.0). The `email` icon and the
  theme-toggle sun/moon/bolt icons are original.

        