
+++
title = "after-dark"
description = "A robust, elegant dark theme"
template = "theme.html"
date = 2017-11-07T17:39:37+01:00

[extra]
created = 2019-04-06T11:27:43+02:00
updated = 2017-11-07T17:39:37+01:00
repository = "https://github.com/getzola/after-dark"
homepage = "https://github.com/getzola/after-dark"
minimum_version = "0.5.0"
license = "MIT"
demo = "https://zola-after-dark.netlify.com"

[extra.author]
name = "Vincent Prouillet"
homepage = "https://www.vincentprouillet.com"
+++        

# after-dark

![after-dark screenshot](https://github.com/getzola/after-dark/blob/master/screenshot.png?raw=true)

## Contents

- [Installation](#installation)
- [Options](#options)
  - [Top menu](#top-menu)
  - [Title](#title)

## Installation
First download this theme to your `themes` directory:

```bash
$ cd themes
$ git clone https://github.com/getzola/after-dark.git
```
and then enable it in your `config.toml`:

```toml
theme = "after-dark"
```

This theme requires your index section (`content/_index.md`) to be paginated to work:

```toml
paginate_by = 5
```

The posts should therefore be in directly under the `content` folder.

The theme requires tags and categories taxonomies to be enabled in your `config.toml`:

```toml
taxonomies = [
    # You can enable/disable RSS
    {name = "categories", rss = true},
    {name = "tags", rss = true},
]
```
If you want to paginate taxonomies pages, you will need to overwrite the templates
as it only works for non-paginated taxonomies by default.


## Options

### Top-menu
Set a field in `extra` with a key of `after_dark_menu`:

```toml
after_dark_menu = [
    {url = "$BASE_URL", name = "Home"},
    {url = "$BASE_URL/categories", name = "Categories"},
    {url = "$BASE_URL/tags", name = "Tags"},
    {url = "https://google.com", name = "Google"},
]
```

If you put `$BASE_URL` in a url, it will automatically be replaced by the actual
site URL.

### Title
The site title is shown on the homepage. As it might be different from the `<title>`
element that the `title` field in the config represents, you can set the `after_dark_title`
instead.

## Original
This template is based on the Hugo template https://git.habd.as/comfusion/after-dark

        