
+++
title = "emily_zola_theme"
description = "a KISS theme for Zola"
template = "theme.html"
date = 2024-01-25T10:41:35+02:00

[extra]
created = 2024-01-25T10:41:35+02:00
updated = 2024-01-25T10:41:35+02:00
repository = "https://github.com/kyoheiu/emily_zola_theme.git"
homepage = "https://github.com/kyoheiu/emily_zola_theme"
minimum_version = "0.14.1"
license = "MIT"
demo = "https://emily-zola-theme.netlify.app/"

[extra.author]
name = "Kyohei Uto"
homepage = "https://github.com/kyoheiu"
+++        

# emily_zola_theme

![screenshot01](/static/images/ss01.png)


A KISS theme for Zola (static site generator written in Rust). 

Features:
- simple & clean
- mobile-friendly
- MathJax support

Demo site is [here](https://emily-zola-theme.netlify.app/).

## Usage

```
cd YOUR_SITE_DIRECTORY/themes
git clone https://github.com/kyoheiu/emily_zola_theme.git
```

and set the theme-name in `config.toml` to `emily_zola_theme`.

```
theme = "emily_zola_theme"
```

## example articles

In `YOUR_SITE_DIRECTORY/themes/emily_zola_theme/content`.

## MathJax support

To use MathJax, add the following lines to the front matter in `.md` file. `[extra]` is mandatory:

```
[extra]
math = true
```

## How to customize
In addition to default values, you can customize following parts easily:

- author name (appears in footer)
- header icon (appears in header)
- favicon
- header icon size (default width: 70px)
- number of posts in `index.html` (default 5)

Set your own in `themes/emily_zola_theme/theme.toml`, or to overwrite, copy `[extra]` block, paste it into your `config.toml` and edit.

        