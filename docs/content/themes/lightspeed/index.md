
+++
title = "lightspeed"
description = "Zola theme with a perfect Lighthouse score"
template = "theme.html"
date = 2025-07-05T15:37:03+02:00

[taxonomies]
theme-tags = []

[extra]
created = 2025-07-05T15:37:03+02:00
updated = 2025-07-05T15:37:03+02:00
repository = "https://github.com/carpetscheme/lightspeed.git"
homepage = "https://github.com/carpetscheme/lightspeed"
minimum_version = "0.20.0"
license = "MIT"
demo = "https://carpetscheme.github.io/lightspeed"

[extra.author]
name = "carpetscheme"
homepage = "https://github.com/carpetscheme"
+++        

# Light Speed

A small Zola theme, ported from [Light Speed Jekyll](https://github.com/bradleytaunt/lightspeed).

* Perfect score on Google's Lighthouse audit
* Only ~700 bytes of CSS
* No JavaScript

Demo: [carpetscheme.github.io/lightspeed](https://carpetscheme.github.io/lightspeed)

-----

## Contents

- Installation
- Options
  - Title
  - Footer menu
  - Sass
- Original
- License

## Installation
Download this theme to your `themes` directory:

```bash
$ cd themes
$ git clone https://github.com/carpetscheme/lightspeed.git
```
and then enable it in your `config.toml`:

```toml
theme = "lightspeed"
```

Posts should be placed directly in the `content` folder.

To sort the post index by date, enable sort in your index section `content/_index.md`:

```toml
sort_by = "date"
```

## Options

### Title
Set a title and description in the config to appear in the site header:

```toml
title = "Different strokes"
description = "for different folks"

```

### Footer-menu
Set a field in `extra` with a key of `footer_links`:

```toml
[extra]

footer_links = [
    {url = "$BASE_URL/about", name = "About"},
    {url = "$BASE_URL/atom.xml", name = "RSS"},
    {url = "https://example.com", name = "Example"},
]
```

Create pages such as `$BASE_URL/about` by placing them in a subfolder of the content directory, and specifying the path in the frontmatter:

```toml
path = "about"
```

The footer credit to Zola and Lightspeed can be disabled with the `footer_credits` option.

### Sass

Styles are compiled from sass and imported inline to the header.

You can overide the styles by enabling sass compilation in the config:

```toml
compile_sass = true
```

and placing a replacement `style.scss` file in your sass folder.

## Original
This template is based on the Jekyll template [Light Speed Jekyll](https://github.com/bradleytaunt/lightspeed) by Bradley Taunt.

## License

Open sourced under the [MIT license](LICENSE.md).

This project is open source except for example articles found in `content`.


        