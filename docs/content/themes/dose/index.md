
+++
title = "dose"
description = "a small blog theme"
template = "theme.html"
date = 2022-10-04T04:04:47-05:00

[extra]
created = 2022-10-04T04:04:47-05:00
updated = 2022-10-04T04:04:47-05:00
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

First download this theme to your `themes` directory:

```bash
cd themes
git clone https://github.com/oltdaniel/dose.git
```

and then enable it in your `config.toml`:

```toml
theme = "dose"
```

The following taxonomies are enabled:

```toml
taxonomies = [
    {name = "tags"},
]
```

And the theme uses the following extras:

```toml
[extra]
social_media = [
    {name = "GitHub", url = "https://github.com/oltdaniel"},
    {name = "Twitter", url = "https://twitter.com/@twitter"},
]
```

The description of yourself with your image, you can modify by using a template. Just create a new
file `myblog/parts/me.html`:

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

We need about `~2.3KiB` extra stuff aside from images and raw html. This is divided up to `~2.1KiB CSS` and `212B JS`.

Test yourself with `zola build 1>/dev/null; echo "scale=2; $(cat public/**/*.{js,css} | wc -c)/1024" | bc -l`.

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

### TODO

- [x] introduce sass variables for colors
- [x] dark/light switch with javascript and store in browser local storage

## License

![GitHub](https://img.shields.io/github/license/oltdaniel/dose)

        