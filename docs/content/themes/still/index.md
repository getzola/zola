
+++
title = "still"
description = "A minimal Zola blog theme focused on reading and writing"
template = "theme.html"
date = 2026-04-10T11:04:31+09:00

[taxonomies]
theme-tags = []

[extra]
created = 2026-04-10T11:04:31+09:00
updated = 2026-04-10T11:04:31+09:00
repository = "https://github.com/wjianbo/zola-theme-still.git"
homepage = "https://github.com/wjianbo/zola-theme-still"
minimum_version = "0.22.0"
license = "MIT"
demo = "https://wjianbo.github.io/zola-theme-still/"

[extra.author]
name = "Jianbo Wang"
homepage = "https://wjianbo.github.io/"
+++        

# still

**still** is a minimal Zola theme for personal blogs, essays, notes, and other reading-first sites.

[Live demo](https://wjianbo.github.io/zola-theme-still/) · [Theme repository](https://github.com/wjianbo/zola-theme-still)

## Overview

The theme keeps the structure intentionally spare:

- the homepage can render an introduction followed by recent posts
- section pages list entries with minimal chrome
- post pages focus on typography, metadata, and a simple back link
- light and dark color schemes are supported automatically

## Features

- Minimal monochrome presentation for long-form writing
- Homepage intro plus post listing by default
- Section index pages with consistent date formatting
- Optional author signature in post footers
- Feed link support when `generate_feeds = true`
- Basic social metadata and canonical URLs from your Zola config

## Demo

A live demo is available here:

- https://wjianbo.github.io/zola-theme-still/

This repository also serves as the demo site source, so the files in `content/`, `templates/`, and `sass/` double as working examples for the theme.

## Installation

Create a new Zola site:

```bash
zola init myblog
cd myblog
```

Add the theme under `themes/still`:

```bash
git submodule add https://github.com/wjianbo/zola-theme-still themes/still
```

Enable the theme in `config.toml`:

```toml
theme = "still"
compile_sass = true
build_search_index = false
```

## Quick start

A small site configuration can look like this:

```toml
base_url = "https://example.com"
title = "My Blog"
description = "Notes, essays, and writing."
author = "Your Name"
default_language = "en"
theme = "still"
compile_sass = true
build_search_index = false
generate_feeds = true
feed_filenames = ["atom.xml"]

[extra]
still_date_format = "%Y-%m-%d"
still_show_author = true
```

Then add content like this:

```text
content/
├── _index.md
└── posts/
    ├── _index.md
    └── my-first-post.md
```

Example `content/_index.md`:

```md
+++
title = "My Blog"
+++

Welcome to my site. This introduction will appear above the post list on the homepage.
```

Example `content/posts/_index.md`:

```md
+++
title = "Posts"
sort_by = "date"
+++
```

Example `content/posts/my-first-post.md`:

```md
+++
title = "My First Post"
date = 2026-03-22
+++

Hello world.
```

## Configuration

These optional values can be set in your site `config.toml`:

```toml
[extra]
still_date_format = "%Y-%m-%d"
still_show_author = true
```

### Supported options

- `default_language`: controls the HTML `lang` attribute.
- `still_date_format`: used on the homepage, section pages, and post pages.
- `still_show_author`: shows or hides the footer signature when `author` is set in the main site config.
- `generate_feeds` and `feed_filenames`: add an Atom or RSS `<link rel="alternate">` tag in the page head.

### Content notes

- Put introductory copy for the homepage in `content/_index.md`.
- Root section pages are rendered on the homepage with the intro first and page listings after it.
- Posts work well in a dated section such as `content/posts/`.
- If a page sets `extra.location`, it is displayed next to the author in the post footer.

## Local development

Preview the theme from this repository with:

```bash
zola serve
```

Or from a separate Zola site that uses the theme:

```bash
zola serve
```

Then open the local server shown by Zola, usually <http://127.0.0.1:1111>.

## Demo deployment

This repository includes a GitHub Actions workflow that builds the demo site and publishes it to the `gh-pages` branch.

To enable it:

1. Open the repository settings on GitHub.
2. Go to **Pages**.
3. Set the source to **Deploy from a branch**.
4. Select the `gh-pages` branch and the `/(root)` folder.
5. Push to `main`.

The workflow runs on every push to `main` and updates the published files in `gh-pages`.

## Philosophy

The goal of **still** is to remove structural noise without removing the basic affordances a blog still needs:

- a clear homepage entry point
- predictable navigation back to section or home
- stable typography across reading contexts

        