
+++
title = "Albatros"
description = "A feature rich theme originally made for Duniter website."
template = "theme.html"
date = 2024-01-09T08:33:22+01:00

[extra]
created = 2024-01-09T08:33:22+01:00
updated = 2024-01-09T08:33:22+01:00
repository = "https://git.42l.fr/HugoTrentesaux/Albatros.git"
homepage = "https://git.42l.fr/HugoTrentesaux/Albatros"
minimum_version = "0.16.0"
license = "AGPL"
demo = "https://albatros.coinduf.eu/"

[extra.author]
name = "Hugo Trentesuas"
homepage = "https://trentesaux.fr/"
+++        

# Albatros theme for Zola

This theme was made for [Duniter](https://duniter.fr/) website. It was then abstracted and turned into **Albatros**.

![screenshot](./screenshot.png)

## Installation

Add the theme as a git submodule:

```bash
git submodule add --name albatros https://git.42l.fr/HugoTrentesaux/albatros.git themes/albatros
```

and enable the theme in your `config.toml`

theme = "albatros"

## Features

It has a lot of feature that I could not find time to document yet. Most of the available customization is in `theme.toml`/`extra` section and `sass/_albatros.sass` file (e.g. for colors).

See:

- https://duniter.fr/
- https://duniter.org/

for reference.

### Landing pages

You are encouraged to provide custom landing pages that you can write in `template/custom`.
The theme will take care of the rest (pages organised as wiki with breadcrumb).

### Authors

Each author must have a card in `content/team` folder.

## Support

I'll provide support on demand on [Zola forum](https://zola.discourse.group/) by documenting the theme step by step.
        