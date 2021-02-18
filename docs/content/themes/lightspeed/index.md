
+++
title = "lightspeed"
description = "Zola theme with a perfect Lighthouse score"
template = "theme.html"
date = 2021-02-18T22:27:50+01:00

[extra]
created = 2021-02-18T22:27:50+01:00
updated = 2021-02-18T22:27:50+01:00
repository = "https://github.com/carpetscheme/lightspeed"
homepage = "https://github.com/carpetscheme/lightspeed"
minimum_version = "0.10.0"
license = "MIT"
demo = "https://quirky-perlman-34d0da.netlify.com/"

[extra.author]
name = "El Carpet"
homepage = "https://github.com/carpetscheme"
+++        

# Light Speed

An insanely fast and performance-based Zola theme, ported from [Light Speed Jekyll](https://github.com/bradleytaunt/lightspeed).

Some fun facts about the theme:

* Perfect score on Google's Lighthouse audit
* Only ~700 bytes of CSS
* No JavaScript
* Now with SEO!

Demo: [quirky-perlman-34d0da.netlify.com](https://quirky-perlman-34d0da.netlify.com)

-----

## Contents

- [Installation](#installation)
- [Options](#options)
  - [Title](#title)
  - [Footer menu](#footer-menu)
  - [SEO](#seo)
  - [Footer text](#footer-text)
  - [Sass](#Sass)
- [Original](#original)
- [License](#license)

## Installation
First download this theme to your `themes` directory:

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
    {url = "https://google.com", name = "Google"},
]
```

If you put `$BASE_URL` in a url, it will automatically be replaced by the actual
site URL.

Create pages such as `$BASE_URL/about` by placing them in a subfolder of the content directory, and specifying the path in the frontmatter:

```toml
path = "about"
```

### SEO

Most SEO tags are populated by the page metadata, but you can set the `author` and for the `og:image` tag provide the path to an image:

```toml
[extra]

author = "Grant Green"
ogimage = "Greenery.png"
```

### Footer-text

By default the footer provides links to Zola and Netlify, and a tagline of "Maintained with :heart: for the web".
To disable any of those parts, and/or add a custom tagline of your own, the following options are available:

```toml
[extra]

zola = true
netlify = false
maintained_with_love = false
footer_tagline = "What if everything is an illusion and nothing exists? In that case, I definitely overpaid for my carpet."
```

### Sass

Styles are compiled from sass and imported inline to the header :zap:

You can overide the styles by enabling sass compilation in the config:

```toml
compile_sass = true
```

...and placing a replacement `style.scss` file in your sass folder.

## Original
This template is based on the Jekyll template [Light Speed Jekyll](https://github.com/bradleytaunt/lightspeed) by Bradley Taunt.

## License

Open sourced under the [MIT license](LICENSE.md).

This project is open source except for example articles found in `content`.


        