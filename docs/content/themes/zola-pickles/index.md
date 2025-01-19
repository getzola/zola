
+++
title = "pickles"
description = "A modern, simple, clean blog theme for Zola."
template = "theme.html"
date = 2025-01-03T23:14:08-07:00

[taxonomies]
theme-tags = []

[extra]
created = 2025-01-03T23:14:08-07:00
updated = 2025-01-03T23:14:08-07:00
repository = "https://github.com/lukehsiao/zola-pickles.git"
homepage = "https://github.com/lukehsiao/zola-pickles"
minimum_version = "0.19.0"
license = "BlueOak-1.0.0"
demo = "https://zola-pickles.pages.dev/"

[extra.author]
name = "Luke Hsiao"
homepage = "https://luke.hsiao.dev"
+++        

<h1 align="center">
    🥒<br>
    zola-pickes
</h1>
<div align="center">
    <strong>Pickles is a clean, responsive blog theme for <a href="https://www.getzola.org/">Zola</a> based on the <a href="https://github.com/mismith0227/hugo_theme_pickles">Hugo theme</a> with the same name.</strong>
</div>
<br>
<div align="center">
  <a href="https://zola-pickles.pages.dev/">
    <img src="https://img.shields.io/badge/demo-website-forestgreen" alt="demo website"></a>
  <a href="https://github.com/lukehsiao/zola-pickles/blob/main/LICENSE.md">
    <img src="https://img.shields.io/badge/license-BlueOak--1.0.0-blue" alt="License">
  </a>
</div>
<br>

![pickles screenshot](https://github.com/lukehsiao/zola-pickles/blob/main/screenshot.png?raw=true)

## Installation
First download this theme to your `themes` directory:

```bash
$ cd themes
$ git clone https://github.com/lukehsiao/zola-pickles.git
```
and then enable it in your `config.toml`:

```toml
theme = "zola-pickles"
```

The theme requires putting the posts in the root of the `content` folder and to enable pagination, for example in `content/_index.md`.

```
+++
paginate_by = 5
sort_by = "date"
insert_anchor_links = "right"
+++
```

## Reference guides

## Configuration Options

```toml
[extra]
# A line to display underneath the main title
subtitle = "Example subtitle"

# Text to display in the footer of the page
copyright = "Copyright authors year"

# Your Google Analytics ID
analytics = ""

# See below
katex_enable = false

# See below
instantpage_enable = false
```

A full example configuration is included in config.toml.

Note how pickles also expects `title` and `description` to also be set in the Zola configuration.

### KaTeX math formula support

This theme contains math formula support using [KaTeX](https://katex.org/), which can be enabled by setting `katex_enable = true` in the `extra` section of `config.toml`.

After enabling this extension, the `katex` short code can be used in documents:
* `{%/* katex(block=true) */%}\KaTeX{%/* end */%}` to typeset a block of math formulas,
  similar to `$$...$$` in LaTeX

### Figure Shortcode

The figure shortcode is convenient for captioning figures.

```
{%/* figure(link="https://www.example.com/", src="https://www.example.com/img.jpeg", alt="sample alt text") */%}
Your caption here.
{%/* end */%}
```

### Table Shortcode

The table shortcode is convenient for making mobile-friendly tables (centered with overflow scrollbar).

```
{%/* table() */%}
| Item         | Price | # In stock |
| :----------- | ----: | ---------: |
| Juicy Apples |  1.99 |        739 |
| Bananas      |  1.89 |          6 |
{%/* end */%}
```

### Fontawesome

This theme includes fontawesome, so that fontawesome icons can be directly used.

### Instant.page

The theme contains instant.page prefetching. This can be enabled by setting `instantpage_enable = true` in the `extra` section of `config.toml`.

## Showing article summaries

By default, the theme will use the first 280 characters of your post as a summary, if a proper [page summary](https://www.getzola.org/documentation/content/page/#summary) using `<!-- more -->` is not provided.
For more sensible summaries, we recommend using the manual more indicator.

        