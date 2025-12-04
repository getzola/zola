
+++
title = "Cela"
description = "A minimalist documentation/blog theme."
template = "theme.html"
date = 2025-11-22T23:52:32+08:00

[taxonomies]
theme-tags = ['blog', 'documentation', 'lightweight', 'minimal', 'responsive', 'search']

[extra]
created = 2025-11-22T23:52:32+08:00
updated = 2025-11-22T23:52:32+08:00
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
  <a href="https://edwardzcn-decade.github.io/cela"><img src="https://img.shields.io/badge/Cela-f8f8f8?style=for-the-badge">
  <a href="https://www.getzola.org"><img src="https://img.shields.io/badge/Zola-f8f8f8?style=for-the-badge&logo=zola&logoColor=black"></a>
</p>

*Cela* is an ongoing simple and lightweight theme for Zola.<br>If you like it, Please give a 🌟 on Github :)<br>The style sheet of Cela is adapted from [Catppuccin](https://github.com/catppuccin/catppuccin).

![screenshot](screenshot.png)

---

## Theme Features

+ [x] Catppuccin color theme support
+ [x] Light/Dark mode support
+ [x] Flexible Mathjax support
+ [x] Blog post RSS feeds
+ [x] Blog search
+ [x] Robot tools
+ [ ] Blog post archive (indexing by year)
+ [ ] Internationalization support (i18n directory and translation)

### Tags, Categories, and Taxonomies

Cela provides hexo/hugo categories and tags compatible with zola style taxonomies.

In toml front matter

```toml
[taxonomies]
tags = ["Rust", "Zola"]
categories = ["Programming"]
```

or you can also use yaml front matter

```yaml
taxonomies:
  tags: ["Rust", "Zola"]
  categories: ["Programming"]
```

Zola taxonomies are more powerful for structuring or indexing your content. For now, Cela native provides tags, categories taxonomies. You can add more taxonomies in `config.toml` file. See [zola taxonomies](https://www.getzola.org/documentation/content/taxonomies/) for more information.



## Quick Start

If you just want to know how to install the theme, just skip to Theme Installation.

### Zola Installation

```bash
# Choose one of the following according to your OS
# macOS
brew install zola
# Alpine Linux
apk add zola
# Arch Linux
pacman -S zola
# Through docker
docker pull ghcr.io/getzola/zola:v0.19.1
```

### Create a zola site

Creates your first zola site (myblog) using the following command. Hints, if `myblog` already exists but only contains hidden files (like `.git`), Zola will alswo populate the site.

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
```

or modify the `.gitmodules` file directly

```toml
[submodule "themes/cela"]
	path = themes/cela
	url = https://github.com/edwardzcn-decade/cela
```

Init your submodule, update and sync.

```bash
git submodule update --init --force --recursive
git submodule sync
```

Then change the theme in your `config.toml` file.

```toml
theme = "cela"
```

TODO

#### By Download Releases

TODO

## 👐 Contributing

> [!NOTE]
>
> If you find this project helpful and would like to support its development, see our [CONTRIBUTING](CONTRIBUTING.md) and [CODE_OF_CONDUCT](CODE_OF_CONDUCT.md) guidelines.

## LICENSE

MIT

        