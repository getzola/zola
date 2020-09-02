
+++
title = "solar-theme-zola"
description = "A port of solar-theme-hugo for zola"
template = "theme.html"
date = 2020-09-02T11:42:41+05:30

[extra]
created = 2020-09-02T11:42:41+05:30
updated = 2020-09-02T11:42:41+05:30
repository = "https://github.com/hulufei/solar-theme-zola.git"
homepage = "https://github.com/hulufei/solar-theme-zola"
minimum_version = "0.4.0"
license = "MIT"
demo = ""

[extra.author]
name = "hulufei"
homepage = "https://github/hulufei"
+++        

# Solar Theme for Zola

Port of [Solar theme for Hugo](https://github.com/bake/solar-theme-hugo) to Zola.

![screenshot](./screenshot.png)

## Installation

First download this theme to your `themes` directory:

```bash
$ cd themes
$ git clone https://github.com/hulufei/solar-theme-zola.git
```
and then enable it in your `config.toml`:

```toml
theme = "solar-theme-zola"
```

Add `title` and `description`:

```toml
title = "Your Blog Title"
description = "Your blog description"
```

## Options

### Color schemes

Set color scheme to (Solarized) `dark` or (Solarized) `light` with `highlight_theme` option:

```toml
highlight_theme = "solarized-dark"
```

### Sidebar menu

Set a field in `extra` with a key of `site_menus`:

```toml
site_menus = [
  { url = "https://github/hulufei/solar-theme-zola", name = "Repository" },
  { url = "rss.xml", name = "RSS" },
]
```
Each link needs to have a `url` and a `name`.

        