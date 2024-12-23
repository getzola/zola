
+++
title = "radion"
description = "A sleek, modern blog theme."
template = "theme.html"
date = 2024-12-17T17:19:14-08:00

[taxonomies]
theme-tags = []

[extra]
created = 2024-12-17T17:19:14-08:00
updated = 2024-12-17T17:19:14-08:00
repository = "https://github.com/micahkepe/radion.git"
homepage = "https://github.com/micahkepe/radion"
minimum_version = "0.19.2"
license = "MIT"
demo = "https://micahkepe.com/radion/"

[extra.author]
name = "Micah Kepe"
homepage = "https://micahkepe.com"
+++        

# radion

A sleek, modern blog theme for [Zola](https://www.getzola.org/). See the live 
site demo [here](https://micahkepe.com/radion/).

> **radion** 
> noun 
> 1. (*physics*) A scalar field in higher-dimensional spacetimes
>

<details open>
<summary>Dark theme</summary>

![radion dark theme screenshot](screenshot.png)
</details>

<details>
<summary>Light theme</summary>

![radion light theme screenshot](screenshot-light.png)
</details>

## Features

- [x] Code Snippet Clipboards
  - [x] Line(s)-specific highlighting
- [x] Latex Support
- [x] Light/Dark mode support
- [x] Search functionality
- [x] Table of Contents option

## Contents and Configuration Guide

- Installation
- Options
  - Top menu
  - Title
  - Author
  - GitHub
  - Code Snippets
  - LaTex Support
  - Searchbar
  - Light and Dark Modes
  - Table of Contents
- Acknowledgements

## Installation

First download this theme to your `themes` directory:

```bash
cd themes
git clone https://github.com/micahkepe/radion
```

and then enable it in your `config.toml`:

```toml
theme = "radion"
```

This theme requires your index section (`content/_index.md`) to be paginated to work:

```toml
paginate_by = 5
```

The posts should therefore be in directly under the `content` folder.

The theme requires tags and categories taxonomies to be enabled in your
`config.toml`:

```toml
taxonomies = [
    # You can enable/disable RSS
    {name = "categories", feed = true},
    {name = "tags", feed = true},
]
```

If you want to paginate taxonomies pages, you will need to overwrite the
templates as it only works for non-paginated taxonomies by default.

## Options

### Top-menu

Set a field in `extra` with a key of `radion_menu`:

```toml
radion_menu = [
    {url = "$BASE_URL", name = "Home"},
    {url = "$BASE_URL/categories", name = "Categories"},
    {url = "$BASE_URL/tags", name = "Tags"},
    {url = "https://google.com", name = "Google"},
]
```

If you put `$BASE_URL` in a url, it will automatically be replaced by the actual
site URL.

### Title

The site title is shown on the homepage. As it might be different from the
`<title>` element that the `title` field in the config represents, you can set
the `radion_title` instead.

### Author

You can set this on a per page basis or in the config file.

`config.toml`:

```toml
[extra]
author = "John Smith"
```

In a page (wrap this in +++):

```toml
title = "..."
date = 1970-01-01

[extra]
author = "John Smith"
```

### GitHub

To enable a GitHub reference link in the header, set the following in your
`config.toml`:

```toml
[extra]
github = "https://github.com/your-github-link"
```

### Code Snippets

Syntax Highlighting:

```toml
[markdown]
# Whether to do syntax highlighting
# Theme can be customized by setting the `highlight_theme` variable to a theme supported by Zola
highlight_code = true

highlight_theme = "one-dark"
```

Enhanced Codeblocks (Clipboard Support and Language Tags)

```toml
[extra]
codeblock = true
```

### LaTex Support

To enable LaTeX support with MathJax, set the following in your `config.toml`:

```toml
[extra]
latex = true
```

### Searchbar

To enable a searchbar at the top of the page navigation, set the following in
your `config.toml`:

```toml
build_search_index = true

[search]
index_format = "elasticlunr_json"

[extra]
enable_search = true
```

### Light and Dark Modes

To set the color theme of the site, set the following in your `config.toml`:

```toml
[extra]
theme = "toggle" # options: {light, dark, auto, toggle}
```

There are four options for the `theme` field:

- `light`: Always light mode
- `dark`: Always dark mode
- `auto`: Automatically switch between light and dark mode based on the user's
  system preferences
- `toggle`: Allow the user to toggle between light and dark mode

### Table of Contents

To enable a table of contents on a page, add the following to the front matter
of the page:

```toml
[extra]
toc = true
```

## Acknowledgements

Lots of inspiration and code snippets taken from these awesome Zola themes:

- [`after-dark`](https://github.com/getzola/after-dark) by
  [Vincent Prouillet](https://www.vincentprouillet.com/)

- [`apollo`](https://github.com/not-matthias/apollo/tree/main) by
  [not-matthias](https://github.com/not-matthias)

- [`redux`](https://github.com/SeniorMars/redux) by
  [SeniorMars](https://github.com/SeniorMars).

        