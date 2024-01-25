
+++
title = "dose"
description = "a small blog theme"
template = "theme.html"
date = 2024-01-25T10:41:35+02:00

[extra]
created = 2024-01-25T10:41:35+02:00
updated = 2024-01-25T10:41:35+02:00
repository = "https://github.com/oltdaniel/dose.git"
homepage = "https://github.com/oltdaniel/dose"
minimum_version = "0.14.0"
license = "MIT"
demo = "https://oltdaniel.github.io/dose"

[extra.author]
name = "oltdaniel"
homepage = "https://oltdaniel.eu"
+++        

# dose

![](screenshot.png?raw=true)

## Installation

First install the theme into the `themes` directory with one of these options:

```bash
# If you work with git: 
git submodule add https://github.com/oltdaniel/dose.git themes/dose
# or just do a download:
git clone https://github.com/oltdaniel/dose.git themes/dose
```

and then enable it in your `config.toml`:

```toml
theme = "dose"
```

You can enable the following taxonomies:

```toml
taxonomies = [
    { name = "tags", feed = true },
]
```

And the theme uses the following extras:

```toml
[extra]
social_media = [
    { name = "GitHub", url = "https://github.com/oltdaniel" },
    { name = "Twitter", url = "https://twitter.com/@twitter" },
    { name = "Mastodon", url = "https://mastodon.social/@Mastodon", rel = "me" }
]
default_theme = "dark" # or "light"
```

The description of yourself with your image, you can modify by using a template. Just create a new
file `myblog/templates/parts/me.html`:

```html
<img src="https://via.placeholder.com/50" height="50px" width="50px">
<p>Hi, this is me. I write about microcontrollers, programming and cloud software. ...</p>
```

If you want to have all pages sorted by their date, please create `myblog/content/_index.md`:
```
+++
sort_by = "date"
+++
```

### About

#### Inspired
I created this theme mainly for my personal website. You are free to use it or modify it. It is inspired by the [`no-style-please`](https://riggraz.dev/no-style-please/) jekyll theme.

#### Typography

This theme uses no special font, just the browsers default monospace font. Yes, this can mean that the website could be rendered differently, but users can freely choose their webfont.

#### Darkmode

This theme supports dark and light mode. Currently this will be only switched based on the users preffered system theme. But a manual switch will follow in the future in the footer (see the todo).

| light | dark |
|-|-|
| ![](screenshot-light.png) | ![](screenshot-dark.png) |

#### Size

The JavaScript has been moved into the page itself to allow minification. Together this results in the following sizes for the `index.html`:
- `~ 3kB` JavaScript
- `~ 3kB` CSS
- `~ 17kB` Profile Image
- `~5kB - ~3kB = ~2kB` HTML

Which results in a total loading size of `3kB + 3kB + 17kB + 2kB = 25kB`.

#### Syntax Highlighting

As I didn't want to invest any time in creating an own syntax color schema for this theme, I suggest to use `visual-studio-dark`, which is the same one used in the demo page.

#### Customization

You can create your own version of this theme, by simply changing the sass variables in `sass/style.scss` to match your taste.

```scss
/**
 * Variables
 */
$base-background: white;
$text-color: black;
$article-tag: green;
$lang-tag: red;
$link-color: blue;
$target-color: yellow;
$separator-decoration: "//////";
```

## License & Contributors

![GitHub](https://img.shields.io/github/license/oltdaniel/dose)

This project was created by [Daniel Oltmanns](https://github.com/oltdaniel) and has been imporved by these [contributors](https://github.com/oltdaniel/dose/graphs/contributors).
        