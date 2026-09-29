
+++
title = "archie-zola"
description = "A zola theme based on Hugo archie."
template = "theme.html"
date = 2026-09-09T10:49:51+08:00

[taxonomies]
theme-tags = []

[extra]
created = 2026-09-09T10:49:51+08:00
updated = 2026-09-09T10:49:51+08:00
repository = "https://github.com/XXXMrG/archie-zola.git"
homepage = "https://github.com/XXXMrG/archie-zola"
minimum_version = "0.23.4"
license = "MIT"
demo = "https://archie-zola.netlify.app/"

[extra.author]
name = "Keith"
homepage = "https://github.com/XXXMrG"
+++        

# archie-zola

A clean, minimal [Zola](https://www.getzola.org/) theme forked from
[archie](https://github.com/athul/archie), with dark/light mode support.

## Table of Contents

- Demo
- Features
- Installation
- Quick Start
- Configuration
- Content Management
- Customization
- Migrating an existing site
- Troubleshooting
- Contributing

## Demo

**Live Demo:** [https://archie-zola.netlify.app](https://archie-zola.netlify.app)

| Light Mode | Dark Mode |
|------------|-----------|
| ![Light](https://archie-zola.netlify.app/screenshot/screenshot-light.png) | ![Dark](https://archie-zola.netlify.app/screenshot/screenshot-dark.png) |

## Features

- Responsive layout
- Automatic dark mode or a manual dark/light toggle
- Local fonts and icons, with optional CDN loading
- Syntax-highlighted code blocks
- Optional KaTeX math and Google Analytics
- Custom CSS/JS and page meta tags
- Tags, homepage pagination, and social links

## Installation

**Requires Zola 0.23.4 or newer; the tested version is 0.23.4**, recorded in
[.zola-version](.zola-version). Download the archive for your operating system
and architecture from the official
[Zola 0.23.4 release](https://github.com/getzola/zola/releases/tag/v0.23.4),
extract it, and put the `zola` executable on your `PATH`. Check before building:

```bash
zola --version
# Expected: zola 0.23.4
```

This is a **breaking Tera 2 migration**. The current theme does **not** support
Zola 0.22.x or older. Read Migrating an existing site
before updating your theme or local template overrides.

### Add the theme to an existing site

From the site's root:

```bash
git submodule add https://github.com/XXXMrG/archie-zola.git themes/archie-zola
```

Without Git submodules, clone to the same directory instead:

```bash
git clone https://github.com/XXXMrG/archie-zola.git themes/archie-zola
```

Set `theme = "archie-zola"` at the top level of your `config.toml`, before any
`[extra]` or other tables. Do not append a duplicate `theme` key. If your site
uses Zola's newer `zola.toml` filename, edit that file instead; keep only one
site configuration file.

### Updating the theme

```bash
git submodule update --init --recursive
git submodule update --remote themes/archie-zola
zola build
# Review and commit the theme submodule update in your own site repository.
```

If you need the pre-migration theme, pin this existing historical commit;
there is no new compatibility release or tag:

```bash
git -C themes/archie-zola checkout f47231e1331e3df31e0f7490170bb76b9cf1d9da
```

That snapshot is not compatible with Zola 0.23.4. Keep the Zola version and
site configuration you previously verified with it. Its old demo
`highlight_code` / `highlight_theme` settings are also invalid on Zola 0.22.x;
the pin alone is not a fix for those settings.

## Quick Start

After installing Zola 0.23.4, create an empty site and add the theme:

```bash
mkdir my-blog
cd my-blog
git init
mkdir -p themes content/posts
git submodule add https://github.com/XXXMrG/archie-zola.git themes/archie-zola
```

Create `config.toml`:

```toml
base_url = "https://example.com/"
title = "Your Blog"
description = "Your blog description"
theme = "archie-zola"
default_language = "en"
taxonomies = [{ name = "tags" }]

[markdown.highlighting]
theme = "one-light"
style = "inline"
```

No `[extra]` table is required: the theme supplies English labels, Home/All
posts/Tags menus, toggle mode, local fonts/icons, and its packaged favicon.
Analytics, KaTeX, and social links are off until configured. The repository's
`config.toml` is a fuller demo, not a required starting point.

Create `content/_index.md` to paginate the homepage:

```toml
+++
paginate_by = 5
sort_by = "date"
+++
```

Create **`content/posts/_index.md`** to define the posts archive:

```toml
+++
template = "posts.html"
transparent = true
sort_by = "date"
+++
```

The folder determines the section URL `/posts/`; do not add the obsolete
section front-matter `path = "posts"`. `transparent = true` makes posts
available to the homepage paginator as well as the archive.

Create `content/posts/hello-world.md`:

```markdown
+++
title = "Hello World"
date = 2024-01-01
description = "My first post"

[taxonomies]
tags = ["hello"]
+++

Your first post content here.
```

Build, then preview:

```bash
zola build
zola serve
```

## Configuration

### Basic settings

Set your site's `base_url`, `title`, and `description` at the top level, as in
Quick Start. The homepage uses `description` for the text beneath its title.
Optional theme settings go under `[extra]`:

```toml
[extra]
mode = "toggle"              # auto | dark | toggle
useCDN = false               # use packaged fonts/icons
favicon = "/icon/favicon.png"
copyright = "Your Name"
# ga = "G-XXXXXXXXXX"        # optional Google Analytics ID
# katex_enable = true        # optional math support
```

- `auto` follows the system's dark-mode preference.
- `dark` always uses the dark stylesheet.
- `toggle` shows the manual theme-switch button.

### Navigation and translations

English labels and Home/All posts/Tags links are supplied by `theme.toml`.
To override them, retain the legacy **`[[extra.translations.en]]`**
array-of-tables shape, including the double brackets. Provide a complete
entry: arrays replace the theme defaults rather than merging labels inside
an entry. Components fall back to English labels for missing keys. Only add
an About link if you create a page at `/about/`.

```toml
[extra.translations]
languages = [{ name = "en", url = "/" }]

[[extra.translations.en]]
show_more = "Read more ⟶"
previous_page = "← Previous"
next_page = "Next →"
posted_on = "on "
posted_by = "Published by"
read_time = "minute read"
all_tags = "All tags"
menus = [
    { name = "Home", url = "/", weight = 1 },
    { name = "Posts", url = "/posts", weight = 2 },
    { name = "Tags", url = "/tags", weight = 3 },
]
```

For another language, configure it in Zola and provide the corresponding
`[[extra.translations.<language-code>]]` labels and menus. The theme's English
entry remains available when adding other language entries.

### Social links

```toml
[[extra.social]]
icon = "github"
name = "GitHub"
url = "https://github.com/yourusername"

[[extra.social]]
icon = "linkedin"
name = "LinkedIn"
url = "https://linkedin.com/in/yourusername"
```

Icon names come from [Feather Icons](https://feathericons.com/).

### Syntax highlighting

Use Zola 0.23.4's [highlighting configuration](https://github.com/getzola/zola/blob/v0.23.4/docs/content/documentation/content/syntax-highlighting.md):

```toml
[markdown.highlighting]
theme = "one-light"
style = "inline"
```

The demo replaces the old Syntect `OneHalfLight` palette, which is not a
built-in Giallo theme, with the supported `one-light` light palette. It is a
close alternative, not an exact colour match. Explicit inline styling keeps
code colours self-contained without another generated colour stylesheet.
Code blocks remain light in dark mode, as with the previous single light
highlight theme; this migration does not redesign the theme switcher.

Remove the obsolete `[markdown]` keys `highlight_code` and `highlight_theme`.
See the [versioned configuration reference](https://github.com/getzola/zola/blob/v0.23.4/docs/content/documentation/getting-started/configuration.md)
for other supported options. Custom class-based or dual-theme highlighting
requires corresponding CSS/template integration and is not enabled by default.

## Content Management

### Posts and pagination

Create dated posts in `content/posts/`. The Quick Start's transparent posts
section passes them to the homepage; set `paginate_by` and `sort_by = "date"`
in `content/_index.md` to paginate homepage cards. The `/posts/` archive uses
`posts.html` and lists all posts by default. Set `paginate_by` in
`content/posts/_index.md` if you also want to paginate that archive; this is
independent of homepage pagination.

### Tags and optional post metadata

Register `taxonomies = [{ name = "tags" }]` at the top level of the site's
configuration. In a post's front matter:

```toml
+++
title = "Your Post"
date = 2024-01-01
description = "Post description"

[taxonomies]
tags = ["tech", "rust"]

[extra]
author = { name = "Your Name", social = "https://github.com/yourusername" }
tldr = "A quick summary of this post"
meta = [
    { property = "og:title", content = "Custom OG Title" },
    { property = "og:image", content = "https://example.com/image.jpg" },
]
+++
```

Without custom values, the theme generates page title/description meta tags.
For an About page, create `content/about.md` with a title and body, then add
its link to your menu.

### Literal template syntax in Markdown

Zola 0.23 renders content with Tera 2 before Markdown. Backticks and fenced
code blocks alone do **not** protect literal `{{/* ... */}}`, `{%/* ... */%}`, or old
shortcode examples. Wrap only the literal example in a raw block, leaving
real template expressions/components elsewhere in the article active:

````markdown
{%/* raw */%}
```html
{{/* example.variable */}}
{%/* include "example.html" */%}
```
{%/* endraw */%}
````

For entire reference files that must stay literal, Zola also supports the
top-level `skip_content_templating` list of file glob patterns. Prefer raw
blocks for mixed articles and keep any skip patterns narrowly targeted;
do not disable content templating globally just to work around old examples.

## Customization

### Custom CSS and JavaScript

Place files in your site's `static/` directory, then list paths relative to
that directory. Merge these settings into your existing `[extra]` table:

```toml
[extra]
custom_css = ["css/custom.css"]
custom_js = ["js/custom.js"]
```

The theme includes these files in the page's `<head>`.

### LaTeX math support

Enable KaTeX with `katex_enable = true` under `[extra]`, then use:

```markdown
Inline math: \\( E = mc^2 \\)

Block math:
$$
\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}
$$
```

KaTeX loads from its CDN when enabled, independently of `useCDN` for fonts
and icons.

## Migrating an existing site

1. Install and verify Zola **0.23.4** before updating the theme.
2. Use an absolute `base_url`, for example `https://example.com/`, not a
   protocol-relative `//example.com/` URL.
3. Replace old highlighting keys with `[markdown.highlighting]`, as above.
4. Remove obsolete section `path` fields from `_index.md`. Page front-matter
   `path` remains valid, for example the demo's custom About URL.
5. Check Markdown for literal template or shortcode syntax and protect only
   those examples with raw blocks.
6. Migrate **your site's template overrides** as well as the theme. Zola loads
   local overrides first, so an old `templates/page.html`, partial, or macro
   file can still cause a Tera parse error after the theme itself is updated.
   Tera 1 macro declarations/imports and `namespace::macro(...)` calls must be
   migrated to Tera 2 components, not mixed with the new templates.
7. Preserve your translation array-of-tables shape and run `zola build`.

### Template API changes

Shared components are now defined in
[`templates/components.html`](templates/components.html), under the `archie.*`
namespace. The old `templates/macros/macros.html` file and `post_macros` import
namespace are removed. For example, replace this Tera 1 override:

```jinja
{%/* import "macros/macros.html" as post_macros */%}
{{/* post_macros::content(page=page, extra=config.extra) */}}
```

with a Tera 2 component call (no macro import):

```jinja
{{/*<archie.content page={page} extra={config.extra} lang={lang} />*/}}
```

The shared API also includes `archie.list_posts(pages, extra, lang)`,
`archie.list_title(pages, tag_name="")`, `archie.tags(page, lang, short=false)`,
`archie.translate(key, extra, lang)`, and
`archie.pagination(paginator, extra, lang)`. These are component signatures,
not function-call syntax: invoke them with `{{/*<archie.name ... />*/}}` and pass
expression-valued arguments in braces. Components receive their data
explicitly; pass `lang` and `extra` where required instead of relying on old
macro scope. If you override the shared component file, update its component
definitions and every caller together.

Consult the theme's current `templates/` files and
[Zola's theme extension guide](https://www.getzola.org/documentation/themes/extending-a-theme/)
when rebasing overrides. Do not copy the historical macro files into the new
version to suppress a missing-component error.

## Troubleshooting

- **Theme not loading:** check the top-level `theme` setting and
  `themes/archie-zola/` directory. Initialize submodules with
  `git submodule update --init --recursive`.
- **Tera parse error near `macro`, `import`, or `::`:** check `zola --version`
  and migrate local template overrides; downgrading to 0.22.x is not supported
  by this theme version.
- **Unknown highlighting field/theme:** use the new schema and a Giallo theme
  name, not the old `highlight_code`, `highlight_theme`, or `OneHalfLight`.
- **No posts appearing:** use dated posts and follow Quick Start's section
  setup. A site without a posts section can build, but create
  `content/posts/_index.md` when you want the `/posts/` archive.
- **Unexpected English labels after an override:** missing keys fall back to
  English. Supply all desired labels in your `[[extra.translations.en]]`
  (or localized) entry, not just `menus`.
- **Dark-mode problems:** check `extra.mode` and clear stale browser assets.
- **Custom assets missing:** verify that each configured CSS/JS path exists
  below your site's `static/` directory.

## Contributing

Report bugs with reproduction steps, suggest features, submit focused PRs,
or improve documentation.

The repository root is already a demo site. No nested test site or symlink
is needed:

```bash
git clone https://github.com/XXXMrG/archie-zola.git
cd archie-zola
zola --version  # use 0.23.4
zola build
zola serve
```

Run the real-build regression suite from this directory with Python 3 and
Zola 0.23.4:

```bash
python3 -m unittest discover -s tests -v
# Or select an executable without changing PATH:
ZOLA_BIN=/absolute/path/to/zola python3 -m unittest discover -s tests -v
```

## License

This theme is released under the MIT License. See [LICENSE](LICENSE).

## Credits

- Original [Archie theme](https://github.com/athul/archie) by @athul
- Ported to Zola by @XXXMrG
- Icons by [Feather Icons](https://feathericons.com/)

        