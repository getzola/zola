
+++
title = "Toucan"
description = "Inspired from Pelican default theme"
template = "theme.html"
date = 2021-02-18T22:27:50+01:00

[extra]
created = 2021-02-18T22:27:50+01:00
updated = 2021-02-18T22:27:50+01:00
repository = "https://git.42l.fr/HugoTrentesaux/toucan.git"
homepage = "https://git.42l.fr/HugoTrentesaux/toucan"
minimum_version = "0.8.0"
license = "AGPL"
demo = "http://blog.coinduf.eu/"

[extra.author]
name = "Hugo Trentesaux"
homepage = "https://trentesaux.fr/"
+++        

# Toucan

A light theme for Zola adapted from Pelican.

![screenshot](./screenshot.png)

## Installation

You can add the theme as a submodule :

```bash
git submodule add --name toucan https://git.42l.fr/HugoTrentesaux/toucan.git themes/toucan
```

and enable the theme in your `config.toml`

```toml
theme = "toucan"
```

## Usage

Categories will be added to the menu, and all articles from categories with

```toml
transparent = true
```

will be listed in the home page.

You can personalize the following options :

```toml
[extra]
title = "Toucan theme"
title_pic = "/favicon.ico"
description = "Theme for Zola inspired from Pelican website theme"
license = """Content under <a href="https://creativecommons.org/licenses/by-sa/4.0/">CC BY-SA</a> Licence"""
```


        