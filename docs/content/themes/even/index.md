
+++
title = "even"
description = "A robust, elegant dark theme"
template = "theme.html"
date = 2018-01-25T18:44:44+01:00

[extra]
created = 2018-08-04T13:14:45+02:00
updated = 2018-01-25T18:44:44+01:00
repository = "https://github.com/getzola/even"
homepage = "https://github.com/getzola/even"
minimum_version = "0.5.0"
license = "MIT"
demo = "https://zola-even.netlify.com"

[extra.author]
name = "Vincent Prouillet"
homepage = "https://www.vincentprouillet.com"
+++        

# Even
Even is a clean, responsive theme based on the Hugo theme with the same name featuring categories, tags and pagination.

![even screenshot](https://github.com/getzola/even/blob/master/screenshot.png?raw=true)

## Contents

- [Installation](#installation)
- [Options](#options)
  - [Top menu](#top-menu)
  - [Title](#title)

## Installation
First download this theme to your `themes` directory:

```bash
$ cd themes
$ git clone https://github.com/getzola/even.git
```
and then enable it in your `config.toml`:

```toml
theme = "even"
```

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

It also requires to put the posts in the root of the `content` folder and to enable pagination, for example in `content/_index.md`:

```
+++
paginate_by = 5
sort_by = "date"
+++
```

## Options

### Top-menu
Set a field in `extra` with a key of `even_menu`:

```toml
# This is the default menu
even_menu = [
    {url = "$BASE_URL", name = "Home"},
    {url = "$BASE_URL/categories", name = "Categories"},
    {url = "$BASE_URL/tags", name = "Tags"},
    {url = "$BASE_URL/about", name = "About"},
]
```

If you put `$BASE_URL` in a url, it will automatically be replaced by the actual
site URL.

### Title
The site title is shown on the header. As it might be different from the `<title>`
element that the `title` field in the config represents, you can set the `even_title`
instead.

        