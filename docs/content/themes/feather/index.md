
+++
title = "feather"
description = "A fuzzy blog theme"
template = "theme.html"
date = 2024-07-01T05:58:26Z

[extra]
created = 2024-07-01T05:58:26Z
updated = 2024-07-01T05:58:26Z
repository = "https://github.com/piedoom/feather.git"
homepage = "https://github.com/piedoom/feather"
minimum_version = "0.11.0"
license = "MIT"
demo = "http://feather.doomy.org/"

[extra.author]
name = "doomy"
homepage = "https://doomy.org"
+++        

# feather
A lightweight blog theme for [Zola](https://www.getzola.org/) (and to my knowledge the first of now
many themes created specifically for Zola).

# [Live demo 🔗](https://feather.doomy.org/)

[![screenshot](screenshot.png)](https://feather.doomy.org/)

## Features

- Fully responsive
- Designed for legibility
- All JS is non-critical and fails gracefully

## Options
Zola allows themes to [define `[extra]` variables](https://www.getzola.org/documentation/getting-started/configuration/)
in the config. Here's a full list of theme variables with example values and comments.

```toml
# Regular variables you might want to set...
title = "My site" # Otherwise, this will read "Home" in the nav

[extra]
# Specify a theme
# Default: unset
#
# by default, feather enables light and dark mode
# (and switching when javascript is enabled.)
# However, if you prefer to only allow one mode,
# set this to "dark" or "light".
feather_theme = "dark"

# Quickly insert into `<head>`
# Default: unset
feather_head = "<script>alert()</script>"

# Add Disqus comments
# Default: unset
#
# Adds comments to pages by providing your
# disqus domain. Comments will not appear on
# index pages, etc.
feather_disqus_domain = "mysite-com"

# Hide the nav bottom border/background image
# Default: false
feather_hide_nav_image = true
```

# Usage
Using feather is easy.  Install [Zola](https://www.getzola.org/) and follow
[the guide for creating a site and using a theme](https://www.getzola.org/documentation/themes/installing-and-using-themes/).  Then,
add `theme = "feather"` to your `config.toml` file.

If you intend to publish your site to GitHub Pages, please check out [this
tutorial](https://www.getzola.org/documentation/deployment/github-pages/).

You can specify `tags` taxonomies .

# Developing & Contributing
Because feather comes with example content, you can run the theme just like any Zola
blog with `zola serve`.

        