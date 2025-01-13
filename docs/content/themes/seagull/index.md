
+++
title = "Seagull"
description = "A Zola theme."
template = "theme.html"
date = 2025-01-09T11:39:48+01:00

[taxonomies]
theme-tags = []

[extra]
created = 2025-01-09T11:39:48+01:00
updated = 2025-01-09T11:39:48+01:00
repository = "https://git.42l.fr/HugoTrentesaux/seagull.git"
homepage = "https://git.42l.fr/HugoTrentesaux/seagull"
minimum_version = "0.17.0"
license = "AGPL"
demo = "https://seagull.coinduf.eu/"

[extra.author]
name = "Hugo Trentesaux"
homepage = "https://trentesaux.fr/"
+++        

# Seagull

A Zola theme.

![gull](./static/img/gull_rect.svg)

## Installation

Add the theme as a git submodule

```bash
git submodule add --name seagull https://git.42l.fr/HugoTrentesaux/seagull.git themes/seagull
```

Enable the theme in your `config.toml`

```
theme = "seagull"
```

Add a `_variables.sass` file in a `sass` folder

```sh
mkdir sass
touch sass/_variables.sass
```

Add a `_index.md` file in your `content` folder.

## Features

Features can be seen on the demo website: https://seagull.coinduf.eu/.

You can customize the theme with the `/sass/_variables.sass` file.

## Support

I'll provide support on demand on [Zola forum](https://zola.discourse.group/) if you tag [@HugoTrentesaux](https://zola.discourse.group/u/hugotrentesaux/summary)

## Build website

Because of the hack used to allow theme customization, before building seagull website itself, you need to create an empty file

```sh
mkdir ../../sass
touch ../../sass/_variables.sass
```
        