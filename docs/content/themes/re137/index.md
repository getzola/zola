
+++
title = "re137"
description = "A Chinese friendly zola theme. Inspired by lightspeed."
template = "theme.html"
date = 2024-12-26T20:59:14+09:00

[taxonomies]
theme-tags = []

[extra]
created = 2024-12-26T20:59:14+09:00
updated = 2024-12-26T20:59:14+09:00
repository = "https://github.com/tinikov/re137.git"
homepage = "https://github.com/tinikov/re137"
minimum_version = "0.4.0"
license = "MIT"
demo = "https://re137.vercel.app"

[extra.author]
name = "tinikov"
homepage = "https://tinikov.com"
+++        

# re137

![screenshot](screenshot.png)

[Demo](https://re137.vercel.app)

## Installation

Clone this theme to your `themes` directory:

```bash
cd themes
git clone https://github.com/tinikov/re137
```

or add it by submodule:

```bash
cd themes
git submodule add git@github.com:tinikov/re137
```

Then enable it in your `config.toml`:

```toml
theme = "re137"
```

## Structure and must configs

The posts should be directly under the `content` folder, and the single pages (e.g. `about.md`) should be under the `content/pages` folder.

Your index section in content (`content/_index.md`) should enable the posts sorted by date:

```toml
sort_by = "date"
```

And the index section in pages (`content/pages/_index.md`) should disable the `render` option:

```toml
render = false
```

In both index sections, it's recommended to enable the option for anchors:

```toml
insert_anchor_links = "right"
```

## Options

### Enable categories

To enable category page, `taxonomies` should be set in `config.toml`:

```toml
taxonomies = [{ name = "categories" }]
```

### Top-menu

Set a field in `[extra]` of `config.toml` with a key of `re137_menu_links`:

```toml
re137_menu_links = [
    { url = "$BASE_URL", name = "主页" },
    { url = "$BASE_URL/categories", name = "分类" },
    { url = "$BASE_URL/about", name = "关于" },
    { url = "$BASE_URL/rss.xml", name = "RSS" },
]
```

### Page options

In frontmatter of posts and single pages, supported options are listed below:

```toml
title = " "
path = "about" # This is only for single page
authors = ["TC", "BB"] # If there are several authors
date = 2000-01-01T00:00:00+00:00
updated = 2004-01-01T00:00:00+00:00
description = " "
draft = true
[taxonomies]
categories = [" "]
[extra]
author_gen = false # Do not generate author
toc_gen = true # Generate table of contents
```

### Misc

You can add more customed options in `[extra]` of `config.toml`

```toml
# Image shown when you share this website to others
ogimage = " "

# SEO settings
seo = true
google_search_console = " "

# Show jump buttons for Indiewebring
indiewebring = true

# For authentication in mastodon
mastodon = "https://o3o.ca/@tinikov"

# Only user name is needed
github = "tinikov"

# Show establish date in the footer
establishdate = "2024"

# CC BY-NC 4.0 (https://creativecommons.org/licenses/by-nc/4.0/)
license = true

# Show "🫧 Zola theme re137" in the footer
show_zola_theme = true
```

        