
+++
title = "Cela"
description = "A minimalist documentation/blog theme."
template = "theme.html"
date = 2025-12-06T11:43:01+08:00

[taxonomies]
theme-tags = ['blog', 'documentation', 'lightweight', 'minimal', 'responsive', 'search']

[extra]
created = 2025-12-06T11:43:01+08:00
updated = 2025-12-06T11:43:01+08:00
repository = "https://github.com/edwardzcn-decade/cela.git"
homepage = "https://github.com/edwardzcn-decade/cela"
minimum_version = "0.19.0"
license = "MIT"
demo = "https://edwardzcn-decade.github.io/cela/"

[extra.author]
name = "Edward Zhang"
homepage = "https://github.com/edwardzcn-decade"
+++        

# Cela

<p align="center">
  <a href="https://edwardzcn-decade.github.io/cela"><img src="https://img.shields.io/badge/Cela-f8f8f8?style=for-the-badge"></a>
  <a href="https://www.getzola.org"><img src="https://img.shields.io/badge/Zola-f8f8f8?style=for-the-badge&logo=zola&logoColor=black"></a>
</p>

*Cela* is a simple, lightweight Zola theme, inspired by [Hugo PaperMod](https://github.com/adityatelange/hugo-PaperMod).

The style sheet is adapted from [Catppuccin](https://github.com/catppuccin/catppuccin).
If you like it, please give it a 🌟 on GitHub. Thanks!

![screenshot](screenshot.png)

---

## Theme Features

+ [x] Catppuccin color theme
+ [x] Light/Dark mode toggle
+ [x] MathJax support
+ [x] Blog RSS feeds
+ [x] Full-text search
+ [x] Robot tools
+ [ ] Blog archive (group by year)
+ [ ] Internationalization (i18n)

### Tags, Categories, and Taxonomies

Cela provides Hexo/Hugo-like `tags` and `categories`, compatible with Zola `taxonomies`. In front matter:

```toml
[taxonomies]
tags = ["Rust", "Zola"]
categories = ["Programming"]
```

or in YAML style:

```yaml
taxonomies:
  tags: ["Rust", "Zola"]
  categories: ["Programming"]
```

Zola `taxonomies` as recommended are more powerful for structuring your contents. See [zola taxonomies](https://www.getzola.org/documentation/content/taxonomies/) for more information.

## Quick Start

If you only need installation of the theme, skip to Theme Installation.

### Zola Installation

```bash
# macOS
brew install zola
# Alpine Linux
apk add zola
# Arch Linux
pacman -S zola
# Docker
docker pull ghcr.io/getzola/zola:v0.19.1
```

### Create a Zola site

Creates your first Zola site.

If `myblog` already exists but only contains hidden files (like `.git`), Zola will alswo populate the site.

```bash
zola init myblog
# or
# populate the current directory
zola init
```

Any choices you make during the initialization can be changed later in the `config.toml` file.


### Theme Installation

#### By Git submodule

```bash
git submodule add https://github.com/edwardzcn-decade/cela themes/cela
git submodule update --init --force --recursive
git submodule sync
```

Then set the `theme` in your `config.toml` file.

```toml
theme = "cela"
```

#### By Download Releases

1. Download the latest release archive from the Cela releases.
2. Unzip to themes/cela in your Zola project.
3. Set `theme` in config.toml.
4. (Optional) Delete unused example content under content/ if you start fresh.

## 👐 Contributing

> [!NOTE]
>
> If you find this project helpful and would like to support its development, see our [CONTRIBUTING](CONTRIBUTING.md) and [CODE_OF_CONDUCT](CODE_OF_CONDUCT.md) guidelines.

## LICENSE

MIT


        