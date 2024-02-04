
+++
title = "tilde"
description = "Simple theme to match the dracula tilde css"
template = "theme.html"
date = 2024-01-25T10:41:35+02:00

[extra]
created = 2024-01-25T10:41:35+02:00
updated = 2024-01-25T10:41:35+02:00
repository = "https://git.sr.ht/~savoy/tilde"
homepage = "https://git.sr.ht/~savoy/tilde"
minimum_version = "0.4.0"
license = "GPLv3"
demo = "https://savoy.srht.site/blog-demo"

[extra.author]
name = "savoy"
homepage = "https://tilde.team/~savoy/"
+++        

# tilde

Lightweight and minimal blog theme for the [Zola](https://www.getzola.org/)
static site generator.

Live demo is available here:
[https://savoy.srht.site/blog-demo](https://savoy.srht.site/blog-demo)

![](screen_index.png)

![](screen_post.png)

## Installation

[Theme documentation](https://www.getzola.org/documentation/themes/installing-and-using-themes/)

Clone this repository into your site's `themes` directory or add it as a
submodule:

```bash
# Clone into themes
$ git clone https://git.sr.ht/~savoy/tilde themes/tilde
# Add as a submodule
$ git submodule add https://git.sr.ht/~savoy/tilde themes/tilde
```

## Configuration

This theme offers the following config options:

```toml
[extra]

homepage = "" # author homepage
subtitle = "" # blog subtitle
git_source = "" # blog source code
author = "" # author name
email = "" # author email
license = "" # blog license
```

        