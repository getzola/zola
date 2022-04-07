
+++
title = "karzok"
description = "A theme for your documentation. Fast and secure"
template = "theme.html"
date = 2022-04-06T19:00:39-07:00

[extra]
created = 2022-04-06T19:00:39-07:00
updated = 2022-04-06T19:00:39-07:00
repository = "https://github.com/kogeletey/karzok.git"
homepage = "https://github.com/kogeletey/karzok"
minimum_version = "0.0.14"
license = "Apache-2.0"
demo = "https://fmatch.org/karzok"

[extra.author]
name = "Konrad Geletey"
homepage = ""
+++        

<p align="center">
  <a href="https://builds.sr.ht/~kogeletey/karzok"><img src="https://builds.sr.ht/~kogeletey/karzok.svg"  alt="builds.sr.ht status" /></a>
  <a href="https://design.penpot.app/#/view/b4a9c170-5cb6-11ec-826f-e949c75b760d?page-id=ef8611e1-7c24-11eb-89c7-03f8ac143bbf&index=0&share-id=d81024f0-5cb7-11ec-826f-e949c75b760d"><img alt="designed with penpot" src="https://badgen.net/badge/designed%20with/penpot/31EFB8" /></a>
</p>

# Karzok

A theme for your documentation. Fast and secure

![screenshot](./screenshot.png)

## Demo

[Fmatch Karzok](https://fmatch.org/karzok)

## Requirements

Karzok uses npm,zola to dependency managment,rendering, scripts and plugins.

### Install

1. [Zola](https://www.getzola.org/documentation/getting-started/installation/)
2. [Node.js](https://nodejs.org/)

for your platform.

### Optional

1. [yj](https://github.com/sclevine/yj)
   > for transfer toml file in yaml
2. [docker](https://docs.docker.com/engine/install/)
   > for packaging container
3. [rsync](https://rsync.samba.org/)
   > A better copy and move

## Get Started

### 1. Create a new zola site

```zsh
zola init zola_site
```

### 2. Download this theme to you themes directory:

```zsh
git clone https://git.sr.ht/~kogeletey/karzok zola_site/themes
```

or install as submodule:

```zsh
cd zola_site
git init # if your project is a git repository already, ignore this command
git submodule add https://git.sr.ht/~kogeletey/karzok zola_site/themes
```

### 3. Configuration. Open in favorite editor `config.toml`

```toml
base_url = "https://karzok.example.net" # set-up for production
theme = "karzok"
```

See more in [Karzok Configuration](#configuration)

### 4. Added new content

```zsh
    cp ./themes/content/_index.md content/_index.md
    # a template will appear with which you can quickly start writing
    # cp ./themes/content/tmpl.md content/filename.md
```

how you can give freedom to your creativity

### 5. Run the project

i. development enviroment

1. Install node dependencies needed to work

```zsh
npm run gen # don't use npm install before that
```

2. Just run `zola serve` in the root path of the project

```zsh
zola serve
```

Open in favorite browser [http://127.0.0.1:1111](http://127.0.0.1:1111). Saved
changes live reolad.

ii. production enviroment

- with docker

1. Build docker image

```zsh
docker build .
```

or if installed docker-compose

```zsh
docker-compose build
```

2. Run containers

```zsh
docker start -d -p 80:80 container_id
```

or if installed docker-compose

```zsh
docker-compose up -d
```

Open in favorite browser [https://localhost](http://localhost)

## Configuration

## options under the `[extra]`

1. `math` - rendering math formulas throught [katex](https://katex.org)
2. `favicon` - set path to favicon icon import(default `favicon`)
3. `localcdn`- if you want to store all assets on your domain, then enable this
   setting
4. `cdnurl` - you can customize your url to store assets,default use
   [jsdelivr](https://www.jsdelivr.com)
5. `show_word_count` - allowing you to show number of words
6. `show_reading_time`- allowing you to show reading time
7. `children`- for header nesting to work
8. `[[extra.menu]]` - the main navigation on the site
9. `[[extra.header]]` - the header navigantion for the site

### Templates

All pages are extend to the base.html, and you can customize them as need.

## License

This program is Free Software: You can use, study share and improve it at your
will. Specifically you can redistribute and/or modify it under the terms of the
[Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)

# Contribute

Make sure to read the [Code of Conduct](/meta/code-of-conduct)

## Find bugs and come up with features

On the [todo.sr.ht](https://todo.sr.ht/~kogeletey/karzok) or
[github issues](https://github.com/kogeletey/karzok/issues)

## Improve Code

The Karzok is stored in the repository at
[sr.ht](https://sr.ht/~kogeletey/karzok) and mirror
[github](https://github.com/kogeletey/karzok)

> Thank you so much for any help

        