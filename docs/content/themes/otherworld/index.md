
+++
title = "otherworld"
description = "Vaporwave aesthetic theme"
template = "theme.html"
date = 2024-07-01T05:58:26Z

[extra]
created = 2024-07-01T05:58:26Z
updated = 2024-07-01T05:58:26Z
repository = "https://git.blek.codes/blek/otherworld.git"
homepage = "https://git.blek.codes/blek/otherworld"
minimum_version = "0.1.0"
license = "GPL-3-only"
demo = "https://world.blek.codes"

[extra.author]
name = "blek!"
homepage = "https://blek.codes"
+++        

<p align='center'>
    <img src='banner.webp'>
</p>

<h1 align='center'>
    otherworld - a zola theme
</h1>

you can see the demo [here](https://world.blek.codes)

## how to use

### prerequisities
1. a linux system. you can use windows for that, but this guide centers itself on linux based systems.
2. you need to have these programs installed: `git` and `zola`
3. some creativity, html and scss skills

### steps
#### 1. clone the repo
(aka download the theme)

lets assume that your website's directory name in `daftpunk`. it will appear in commands a few times, and you should replace it with your website's name.

```sh
$ git clone git@git.blek.codes:blek/otherworld.git daftpunk
$ cd daftpunk
```

#### 2. open an another terminal
in the same directory, run

```sh
$ zola serve
```

#### 3. edit files in the `content` directory...

...as per [zola docs](https://www.getzola.org/documentation/getting-started/overview)

## how to disable loading
go to `content/index.md`, and in the `+++` blocks, set `extra.noload` to `true`.

like this:
```toml
+++
title = "Welcome"

[extra]
noload = true
+++
```

        