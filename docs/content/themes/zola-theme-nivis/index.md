
+++
title = "nivis"
description = "A clean zola theme for bloggers"
template = "theme.html"
date = 2026-09-07T14:00:16+08:00

[taxonomies]
theme-tags = ['Clean', 'Blog', 'Responsive']

[extra]
created = 2026-09-07T14:00:16+08:00
updated = 2026-09-07T14:00:16+08:00
repository = "https://github.com/Resorie/zola-theme-nivis.git"
homepage = "https://github.com/Resorie/zola-theme-nivis"
minimum_version = "0.23.4"
license = "MIT"
demo = "https://blog.resorie.xyz/"

[extra.author]
name = "Resory"
homepage = "https://blog.resorie.xyz/"
+++        

# Nivis

Nivis is a clean, responsive [Zola](https://www.getzola.org/) theme for personal blogs.

![Nivis screenshot](screenshot.png)

[Example site](https://resorie.xyz/zola-theme-nivis/) | [Resory's blog](https://blog.resorie.xyz/)

> **AIGC disclosure:** Parts of this documentation were drafted with AIGC assistance and subsequently checked against the theme source and a local Zola build.

Nivis is inspired by and derived from [Float](https://float-theme.netlify.app/) and [anatole](https://longfangsong.github.io/).

## Features

- Clean, minimalist design
- Responsive desktop and mobile layouts
- Automatic light/dark mode with a manual toggle
- Focused and horizontal homepage layouts
- Multiple-tags, single-category, and taxonomy-free modes
- Post subtitles, abstracts, table of contents, and pinned posts
- About, archive, taxonomy, and friend-links pages
- Image and collapsible-content components
- MathJax and class-based syntax highlighting

## Getting Started

Nivis requires Zola 0.23.4 or later.

Add the theme as a Git submodule:

```bash
git submodule add -b master --depth=1 https://github.com/Resorie/zola-theme-nivis.git themes/nivis
git submodule update --init --recursive
```

Enable it in the site's `config.toml`:

```toml
theme = "nivis"
compile_sass = true

taxonomies = [{ name = "tags", paginate_by = 5 }]

[markdown.highlighting]
style = "class"
light_theme = "one-light"
dark_theme = "one-dark-pro"

[extra]
home_layout = "focus"
taxonomy_mode = "tags"
social_links = []
```

For a new, otherwise empty site, copy the example content and link data:

```bash
cp -R themes/nivis/content/. content/
cp -R themes/nivis/data/. data/
```

Read the example site's [Getting Started guide](https://resorie.xyz/zola-theme-nivis/posts/get-start/) for the complete configuration, required content structure, local preview, and update instructions.

## Configuration Highlights

| Key | Values | Default |
| --- | --- | --- |
| `extra.home_layout` | `focus`, `horizontal` | `focus` |
| `extra.taxonomy_mode` | `tags`, `category`, `none` | `tags` |
| `extra.math_display` | `mathjax` or omitted | Omitted |
| `extra.motto_mode` | `hide`, `single`, `multi` | `hide` |

Nivis continues to use Zola's `tags` taxonomy internally in all three taxonomy modes. Category mode accepts at most one value per post, while `none` hides taxonomy UI. To stop Zola from generating taxonomy pages as well, use `taxonomies = [{ name = "tags", render = false }]`.

Post lists use `description` as their summary. Articles may add a subtitle and a longer article-page abstract:

```toml
description = "A short summary for post lists."

[extra]
subtitle = "An optional subtitle"
abstract = "A longer abstract shown on the article page."
```

The optional copyright line is enabled by providing a holder:

```toml
[extra.footer]
copyright_holder = "Your name"
copyright_since = 2025
```

## Components

Nivis provides `image` and `collapse` components for Markdown content. Their parameters and rendered examples are documented on the example site after completing the Getting Started guide.

## Todo

- [ ] Add a transition when switching light/dark mode
- [ ] Improve special-page customization
- [ ] Minimize external web resources

        