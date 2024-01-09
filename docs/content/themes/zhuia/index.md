
+++
title = "Zhuia"
description = "An elegant but still playful theme for Zola."
template = "theme.html"
date = 2024-01-09T08:33:22+01:00

[extra]
created = 2024-01-09T08:33:22+01:00
updated = 2024-01-09T08:33:22+01:00
repository = "https://github.com/gicrisf/zhuia.git"
homepage = "https://github.com/gicrisf/zhuia"
minimum_version = "0.15.0"
license = "MIT"
demo = "https://zhuia.netlify.app"

[extra.author]
name = "Giovanni Crisalfi"
homepage = "https://github.com/gicrisf"
+++        

# Zhuia

![logo-zhuia](static/logo.png)

An elegant but still playful theme for [Zola](https://getzola.org/) powered by [Spectre.css](https://picturepan2.github.io/spectre/).

It is especially optimized for mobile navigation (optionally without JavaScript, if you don't like fancy stuff).

**DEMO**: [https://zhuia.netlify.app/](https://zhuia.netlify.app/)

## Contents

- Installation
- Features
- Options
  - Title
  - SEO
  - Menu
  - Social
  - Footer
- Name
- Genesis
- Donate
- License

## Installation

First download this theme to your `themes` directory:

```bash
$ cd themes
$ git clone https://github.com/gicrisf/zhuia.git
```
and then enable it in your `config.toml`:

```toml
theme = "zhuia"
```

Posts should be placed directly in the `content` folder.

To sort the post index by date, enable sort in your index section `content/_index.md`:

```toml
sort_by = "date"
```

## Features
- [x] Lightweight and minimal
- [x] Spectre CSS classes to manage content. [Look at the docs](https://picturepan2.github.io/spectre/)
- [x] Responsive for mobile support (with full-page mobile menu)
- [x] SCSS based CSS source files for easy customization
- [x] HTML based sidebar widget
- [x] Author card sidebar widget with customizable avatar
- [ ] Multi-author support
- [x] Optional twitter sidebar widget
- [x] Feed RSS/Atom
- [x] Open Graph and Twitter Cards support
- [x] Social buttons with icons
- [x] Deploy via Netlify (config already included)
- [x] Tags AND categories
- [x] Granular image optimization for a really faster loading on mobile
- [x] Pagination
- [x] Easily extendable menu
- [ ] Inter-page pagination
- [x] Optional NoJs
- [x] Hamburger animation
- [ ] Comments
- [ ] Related posts (not sure about this)
- [ ] Search bar
- [x] Math rendering
- [x] Other shortcodes
- [ ] Multilanguage support
- [ ] Dark mode
- [ ] Table of Contents
- [ ] Image + text title option

## Options

### Title

Set a title and description in the config to appear in the site header and on the RSS feed:

```toml
title = "Der Prozess"
description = "a novel written by Franz Kafka in 1914"
```

### SEO

Most SEO tags are populated by the page metadata, but you can set the `author` and for the `og:image` tag provide the path to an image:

```toml
[extra]

author = "Timothy Morton"
og_image = "Hyperobjects.png"
```

### Menu
You can choose between two modes:
- With a small script for an elegant overlay menu
- Without any scripts at all (it just your show menu underneath)

![mobile menus](screenshot-mobile-menus.png)

### Social

Set a field in `extra` with a key of `footer_links`:

```toml
[extra]

# Freely comment out or delete every field
social_links = [
    {url = "https://t.me/yourname", name = "telegram"},
    {url = "https://twitter.com/gicrisf", name = "twitter"},
    {url = "https://github.com/gicrisf", name = "github"},
    # {url = "", name = "facebook"},
    # {url = "", name = "instagram"},
    # {url = "", name = "bookstack"},
    # {url = "", name = "dokuwiki"},
]
```

![social buttons](social-buttons.png)

The theme automatically picks up the right icons.
We can expand the support to other social, for sure: make a PR or open an enhancement issue to ask a new implementation.

### Footer

You can add your own copyright or whatever to the footer with a through a simple option on the config file:

```toml
[extra]

footer_tagline = "What if everything is an illusion and nothing exists? In that case, I definitely overpaid for my carpet."
```

## Name

The name arise from two parts:
- The generator, Zola, gives the "Z";
- An extinct species of New Zealand wattlebird, the huia, provide the second part.

The theme is built on **Spectre** CSS framework, so I found reasonable evoking a **spectral species**.

## Genesis

This theme is based on a Pelican theme I originally made for my blog, which was in turn based on the 
Grav theme [Quark](https://github.com/getgrav/grav-theme-quark).

## Donate
Did you liked this theme? Make a donation and support new features!

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/V7V425BFU)

## License

Open sourced under the [MIT license](LICENSE.md).

        