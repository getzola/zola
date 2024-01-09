
+++
title = "nasm-theme"
description = "A robust, elegant blue theme"
template = "theme.html"
date = 2024-01-09T08:33:22+01:00

[extra]
created = 2024-01-09T08:33:22+01:00
updated = 2024-01-09T08:33:22+01:00
repository = "https://github.com/lucasnasm/nasm-theme.git"
homepage = "https://github.com/lucasnasm/nasm-theme"
minimum_version = "0.1.0"
license = "MIT"
demo = "https://lucasnasm.github.io"

[extra.author]
name = "Francisco Lucas"
homepage = "https://lucasnasm.github.io"
+++        

# nasm-theme

## Web
![nasm-theme web](screenshot.png)

## Mobile
![nasm-theme mobile](https://github.com/lucasnasm/nasm-theme/blob/master/screenshot-mobile.png?raw=true)

## Contents

- nasm-theme
  - Web
  - Mobile
  - Contents
  - Fonts
  - Installation
  - Options
    - Disqus
    - Top-menu
    - Title
  - Original
## Fonts
Font Awesome for icons  
Nunito Font
## Installation
First download this theme to your `themes` directory:

```bash
$ git submodule add git@github.com:lucasnasm/nasm-theme.git themes/nasm-theme
```
and then enable it in your `config.toml`:

```toml
theme = "nasm-theme"
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
### Disqus
set a field `extra` with key of `disqus_username`:
```toml
disqus_username = 'username'
```
### Top-menu
Set a field in `extra` with a key of `nasm-theme`:  
Font Awesome default icons
```toml
nasm_menu = [
    {url = "$BASE_URL", name = "Home", fawesome = "fas fa-home"},
    {url = "$BASE_URL/categories", name = "Categories", fawesome = "fas fa-folder-open"},
    {url = "$BASE_URL/tags", name = "Tags", fawesome = "fas fa-tag" },
    {url = "$BASE_URL/about", name = "About", fawesome = "fas fa-user-alt" },

]
```

If you put `$BASE_URL` in a url, it will automatically be replaced by the actual
site URL.

### Title
The site title is shown on the homepage. As it might be different from the `<title>`
element that the `title` field in the config represents, you can set the `nasm_theme_title`
instead.

## Original
This template is based on the Zola template https://github.com/getzola/after-dark  
Thanks

        