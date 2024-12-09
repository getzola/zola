
+++
title = "feather"
description = "A fuzzy blog theme"
template = "theme.html"
date = 2024-09-12T20:47:48-04:00

[taxonomies]
theme-tags = []

[extra]
created = 2024-09-12T20:47:48-04:00
updated = 2024-09-12T20:47:48-04:00
repository = "https://github.com/piedoom/feather.git"
homepage = "https://github.com/piedoom/feather"
minimum_version = "0.19.0"
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

[extra.feather]
# Specify a specific theme to use, or use system prefs
# If set, the theme switcher button is hidden
theme = "light"
head = "<script></script>" # add anything to the head
hide_nav_image = false # hide the navigation image
disqus_id = "my-site-com" # site domain if you want disqus comments
cusdis_id = "12312-31231123-123123123" # cusdis id if you use their comment service
social =  { url = "https://mastodon.social/@doomy", display = "@doomy@mastodon.social" } # generic social to show on pages
timezone = "America/New_York" # timezone to calculate article post times

[extra.feather.analytics]
goatcounter_id = "mydomain-com" # privacy-focused analytics https://www.goatcounter.com
```

Per post, these options are available:

```toml
[extra.feather.opengraph]
image = "my_image.jpg" # Assumes asset colocation
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

        