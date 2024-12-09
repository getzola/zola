
+++
title = "Kita"
description = "Kita is a clean, elegant and simple blog theme for Zola."
template = "theme.html"
date = 2024-10-07T20:10:13+08:00

[taxonomies]
theme-tags = []

[extra]
created = 2024-10-07T20:10:13+08:00
updated = 2024-10-07T20:10:13+08:00
repository = "https://github.com/st1020/kita.git"
homepage = "https://github.com/st1020/kita"
minimum_version = "0.17.0"
license = "MIT"
demo = "https://st1020.github.io/kita/"

[extra.author]
name = "st1020"
homepage = "https://st1020.com"
+++        

# Kita

Kita is a clean, elegant and simple blog theme for Zola.

This theme is based on Hugo theme [hugo-paper](https://github.com/nanxiaobei/hugo-paper) with some features added.

[Demo](https://st1020.github.io/kita/)

![Screenshot](https://raw.githubusercontent.com/st1020/kita/main/screenshot.png)

## Features

- Easy to use and modify
- No preset limits (This theme does not limit your content directory structure, taxonomy names, etc. It's applicable to all zola sites.)
- Inject support
- Dark mode
- Responsive design
- Social icons
- Taxonomies support
- Projects page
- Archive page
- Table of Content
- Admonition shortcode
- SEO friendly
- Comments using [Giscus](https://giscus.app/)
- Mathematical notations using [KaTeX](https://katex.org/)
- Diagrams and charts using [Mermaid](https://mermaid.js.org/)

## Installation

The easiest way to install this theme is to clone this repository in the themes directory:

```sh
git clone https://github.com/st1020/kita.git themes/kita
```

Or to use it as a submodule:

```sh
git submodule add https://github.com/st1020/kita.git themes/kita
```

Then set `kita` as your theme in `config.toml`.

```toml
theme = "kita"
```

## Configuration

See the `extra` section in [config.toml](https://github.com/st1020/kita/blob/main/config.toml) as a example.

## Inject support

You can easily use inject to add new features to your side without modifying the theme itself.

To use inject, you need to add some HTML files to the `templates/injects` directory.

The available inject points are: `head`, `header_nav`, `body_start`, `body_end`, `page_start`, `page_end`, `footer`, `page_info`.

For example, to load a custom script, you can add a `templates/injects/head.html` file:

```html
<script src="js-file-path-or-cdn-url.js"></script>
```

## License

[MIT License](https://github.com/st1020/kita/blob/main/LICENSE)

Copyright (c) 2023-present, st1020

        