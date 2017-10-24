+++
title = "Creating a theme"
weight = 30
+++

Creating is exactly like creating a normal site with Gutenberg, except you
will want to use many [Tera blocks](https://tera.netlify.com/docs/templates/#inheritance) to
allow users to easily modify it.

A theme also need to have a `theme.toml` configuration file with the
following fields, here's the one from a [real template](https://github.com/Keats/hyde):

```toml
name = "hyde"
description = "A classic blog theme"
license = "MIT"
homepage = "https://github.com/Keats/gutenberg-hyde"
# The minimum version of Gutenberg required
min_version = "0.1"

# Any variable there can be overriden in the end user `config.toml`
# You don't need to prefix variables by the theme name but as this will
# be merged with user data, some kind of prefix or nesting is preferable
# Use snake_casing to be consistent with the rest of Gutenberg
[extra]
hyde_sticky = true
hyde_reverse = false
hyde_theme = ""
hyde_links = [
    {url = "https://google.com", name = "Google.com"},
    {url = "https://google.fr", name = "Google.fr"},
]

# The theme author info: you!
[author]
name = "Vincent Prouillet"
homepage = "https://vincent.is"

# If this is porting a theme from another static site engine, provide
# the info of the original author here
[original]
author =  "mdo"
homepage = "http://markdotto.com/"
repo = "https://www.github.com/mdo/hyde"
```

A theme will also need three directories to work:

- `static`: any static files used in this theme
- `templates`: all templates used in this theme
- `sass`: Sass stylesheets for this theme, can be empty

To be featured on this site, the theme will require two more things:

- `screenshot.png`: a screenshot of the theme in action, its size needs to be reasonable
- `README.md`: a thorough README explaining how to use the theme and any other information
of importance

A simple theme you can use as example is [Hyde](https://github.com/Keats/hyde).
