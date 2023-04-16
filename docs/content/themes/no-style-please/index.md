
+++
title = "no style, please!"
description = "A (nearly) no-CSS, fast, minimalist Zola theme"
template = "theme.html"
date = 2023-04-16T21:40:29+02:00

[extra]
created = 2023-04-16T21:40:29+02:00
updated = 2023-04-16T21:40:29+02:00
repository = "https://gitlab.com/4bcx/no-style-please.git"
homepage = "https://gitlab.com/4bcx/no-style-please"
minimum_version = "0.4.0"
license = "MIT"
demo = "https://4bcx.gitlab.io/no-style-please"

[extra.author]
name = "Ahmed Alaa"
homepage = "https://4b.cx"
+++        

# no style, please!

A (nearly) no-CSS, fast, minimalist [Zola](https://www.getzola.org/) theme.
Ported from from [riggraz](https://riggraz.dev/)'s [no style, please! Jekyll theme](https://riggraz.dev/no-style-please/), and I use it for [my site](https://4b.cx/)

![screenshot](./screenshot.png)

## Installation

First download this theme to your `themes` directory:

```bash
cd themes
git clone https://gitlab.com/4bcx/no-style-please.git
```

and then enable it in your `config.toml`:

```toml
theme = "no-style-please"
```

## Options

### Pages list in homepage

To enable listing of pages in homepage add the following in `content\_index.md` frontmatter

```toml
[exta]
list_pages = false
```

### Extra data

- `author` can be set in both main config and in pages metadata
- `image` variable can be used in pages to add an image to HTML `<meta>` tags
- Same for `logo` in main config, except this one is also used as the site icon

### Horizontal rule shortcode `hr()`

Adds the option to insert text in the thematic break

```html
{{/* hr(data_content="footnotes") */}}
```

is rendered

![thematic break screenshot](./hr_footnotes.png)

### Invertable image `iimg()`

Images are not inverted in darkmode by default. To add an invertable image use the following

```html
{{/* iimg(src="logo.png", alt="alt text") */}}
```

In light mode

![image in light mode](./iimg_light.png)

In dark mode

![image in dark mode](./iimg_dark.png)

## TODO

- [ ] Add RTL support
- [ ] Write proper test pages

## License

The theme is available as open source under the terms of the [MIT License](https://opensource.org/licenses/MIT).

        