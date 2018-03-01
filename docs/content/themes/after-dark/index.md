
+++
title = "after-dark"
description = "A robust, elegant dark theme"
template = "theme.html"
date = 2017-11-07T17:39:37+01:00

[extra]
created = 2018-02-22T19:13:36+01:00
updated = 2017-11-07T17:39:37+01:00
repository = "https://github.com/Keats/after-dark"
homepage = "https://github.com/Keats/after-dark"
minimum_version = "0.2"
license = "MIT"

[extra.author]
name = "Vincent Prouillet"
homepage = "https://vincent.is"
+++        

# after-dark

![after-dark screenshot](https://github.com/Keats/after-dark/blob/master/screenshot.png?raw=true)

## Contents

- [Installation](#installation)
- [Options](#options)
  - [Top menu](#top-menu)
  - [Title](#title)

## Installation
First download this theme to your `themes` directory:

```bash
$ cd themes
$ git clone https://github.com/Keats/after-dark.git
```
and then enable it in your `config.toml`:

```toml
theme = "after-dark"
```

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
This template is based on the Hugo template https://github.com/comfusion/after-dark

        