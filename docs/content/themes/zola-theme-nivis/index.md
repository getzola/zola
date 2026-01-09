
+++
title = "nivis"
description = "A clean zola theme for bloggers"
template = "theme.html"
date = 2026-01-06T00:53:27+08:00

[taxonomies]
theme-tags = ['clean']

[extra]
created = 2026-01-06T00:53:27+08:00
updated = 2026-01-06T00:53:27+08:00
repository = "https://github.com/Resorie/zola-theme-nivis.git"
homepage = "https://github.com/Resorie/zola-theme-nivis"
minimum_version = "0.21.0"
license = "MIT"
demo = "https://resorie.github.io/blog/"

[extra.author]
name = "Resory"
homepage = "https://resorie.github.io/blog/"
+++        

Nivis: A clean zola theme for bloggers.

![screenshot](screenshot.png)

Live demo: [Example Site](https://resorie.github.io/zola-theme-nivis/) | [My Blog](https://resorie.github.io/blog/).

This theme is inspired by (and derived from) themes [Float](https://float-theme.netlify.app/) and [anatole](https://longfangsong.github.io/). Check out these two wonderful themes as well! :smile:

## Features :star:

- Clean & Minimalist Design
- Elegant Typography
- Responsive Layout
- Dark/Light Mode Support

## Getting Started :rocket:

Use `git submodule` to add the theme to your site:
```bash
git submodule add -b master --depth=1 https://github.com/Resorie/zola-theme-nivis.git themes/nivis/
git submodule update --init --recursive
```

Then, change your theme config in `config.toml`:
```toml
theme = "nivis"
```

Start your site by copying the example content into your site folder:
```bash
cp -r themes/nivis/content content
```

Move on to the [example site](https://resorie.github.io/zola-theme-nivis/) for more info. Enjoy it! :kissing_heart:

## Known Bugs :poop:

- [ ] When switching light/dark mode, some elements blink
        