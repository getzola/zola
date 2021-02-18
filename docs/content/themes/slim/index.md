
+++
title = "Slim"
description = "Slim is a minimal, clean and beautiful theme for Zola."
template = "theme.html"
date = 2021-02-18T22:27:50+01:00

[extra]
created = 2021-02-18T22:27:50+01:00
updated = 2021-02-18T22:27:50+01:00
repository = "https://github.com/jameshclrk/zola-slim"
homepage = "https://github.com/jameshclrk/zola-slim"
minimum_version = "0.8.0"
license = "MIT"
demo = "https://zola-slim.netlify.com"

[extra.author]
name = "James Clark"
homepage = "https://jamesclark.dev"
+++        

# Slim

Slim is a minimal, clean and beautiful theme for [Zola](http://getzola.org/).

This theme was ported to Zola, the original is available at [zhe/hugo-theme-slim](https://github.com/zhe/hugo-theme-slim). It is excellent, thank you [zhe](https://github.com/zhe)!

![Slim screenshot](https://github.com/jameshclrk/zola-slim/blob/master/screenshot.png)

[Demo](http://zola-slim.netlify.com).

## Installation

```
cd themes
git clone https://github.com/jameshclrk/zola-slim slim
```

See the [official docs](https://www.getzola.org/documentation/themes/installing-and-using-themes/) for more information.

## Configuration
Slim supports a `tags` taxonomy by default. This can be enabled by setting it in your `config.toml`:

```
taxonomies = [
    {name = "tags", paginate_by = 5, rss = true}
]
```

There are a couple of extra options supported:
```
[extra]
# Show a summary of a post in a list
slim_summary = false
# Show the content of a post in a list
slim_content = false
# Links to show at the top of the menu
slim_menu = [
    {url = "$BASE_URL/tags", name = "Tags"}
]
# Links to show at the bottom of the menu
slim_social = [
    {url = "https://github.com/jameshclrk", name = "Github"}
]
```

## License

Open sourced under [MIT license](https://github.com/zhe/hugo-theme-slim/blob/master/LICENSE.md).

        