
+++
title = "dose"
description = "a small blog theme"
template = "theme.html"
date = 2021-12-02T23:22:24+01:00

[extra]
created = 2021-12-02T23:22:24+01:00
updated = 2021-12-02T23:22:24+01:00
repository = "https://github.com/oltdaniel/dose.git"
homepage = "https://github.com/oltd/dose"
minimum_version = "0.13.0"
license = "MIT"
demo = "https://oltd.github.io/dose"

[extra.author]
name = "oltd"
homepage = "https://oltd.dev"
+++        

# dose

![](screenshot.png?raw=true)

## Installation

First download this theme to your `themes` directory:

```bash
cd themes
git clone https://github.com/oltd/dose.git
```

and then enable it in your `config.toml`:

```toml
theme = "dose"
```

The following taxonomies are enabled:

```toml
taxonomies = [
    {name = "tags"},
]
```

And the theme uses the following extras:

```toml
[extra]
social_media = [
    {name = "GitHub", url = "https://github.com/oltd"},
    {name = "Twitter", url = "https://twitter.com/@oltd_maker"},
]
```

The description of yourself with your image, you can modify by using a template. Just create a new
file `myblog/parts/me.html`:

```html
<img src="https://via.placeholder.com/50" height="50px" width="50px">
<p>Hi, this is me. I write about microcontrollers, programming and cloud software. ...</p>
```

If you want to have all pages sorted by their date, please create `myblog/content/_index.md`:
```
+++
sort_by = "date"
+++
```

### About

#### Inspired
I created this theme mainly for my personal website. You are free to use it or modify it. It is inspired by the [`no-style-please`](https://riggraz.dev/no-style-please/) jekyll theme.

#### Typography

This theme uses no special font, just the browsers default monospace font. Yes, this can mean that the website could be rendered differently, but users can freely choose their webfont.

#### Darkmode

This theme supports dark and light mode. Currently this will be only switched based on the users preffered system theme. But a manual switch will follow in the future in the footer (see the todo).

| light | dark |
|-|-|
| ![](screenshot-light.png) | ![](screenshot-dark.png) |

#### Size

We need about `2kB` extra stuff aside from images and raw html. This is divided up to `1.7kB CSS` and `~300B JS`.

#### Syntax Highlighting

As I didn't want to invest any time in creating an own syntax color schema for this theme, I suggest to use `visual-studio-dark`, which is the same one used in the demo page.

### TODO

- [ ] introduce sass variables for colors
- [ ] dark/light switch with javascript and store in browser session

## License

![GitHub](https://img.shields.io/github/license/oltd/dose)

        