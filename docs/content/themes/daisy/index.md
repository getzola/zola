
+++
title = "Daisy"
description = "Beautiful and fast responsive theme based on TailwindCSS and DaisyUI."
template = "theme.html"
date = 2025-08-30T16:04:32+02:00

[taxonomies]
theme-tags = ['multilingual', 'responsive', 'search']

[extra]
created = 2025-08-30T16:04:32+02:00
updated = 2025-08-30T16:04:32+02:00
repository = "https://codeberg.org/winterstein/zola-theme-daisy.git"
homepage = "https://codeberg.org/winterstein/zola-theme-daisy"
minimum_version = "0.19.0"
license = "MIT"
demo = "https://zola-daisy.winterstein.biz"

[extra.author]
name = "Adrian Winterstein"
homepage = "https://www.winterstein.biz"
+++        

# Daisy Theme

> You can find this theme on [Codeberg](https://codeberg.org/winterstein/zola-theme-daisy) and [Github](https://github.com/awinterstein/zola-theme-daisy).

A beautiful and fast [Zola](https://www.getzola.org/) theme build on [TailwindCSS](https://tailwindcss.com) and [DaisyUI](https://daisyui.com) with 37 different color schemes included. See an example of the *autumn* colors here:

![Screenshot](https://codeberg.org/awinterstein/zola-theme-daisy/raw/branch/main/screenshot.png)

The theme is responsive and works very well on mobile devices:

<img src="https://codeberg.org/awinterstein/zola-theme-daisy/raw/branch/main/screenshot-mobile.png" alt="Mobile Screenshot" width="200"/>

## Features

* Responsive design (looks good on desktop and mobile)
* Automatically selected dark / light modes
* 37 color schemes included
* Customizable navbar and footer (with social links)
* Can be used with any Zola taxonomies (e.g., tags, categories)
* Search functionality
* Multi-language support
* Pagination
* Customizable favicon
* Error 404 page

### Styling

The Daisy theme supports all [built-in color themes of DaisyUI](https://daisyui.com/docs/themes/#enable-a-built-in-theme) plus a light and dark color scheme that I created for my own website. The color themes can optionally even be switched at runtime.

![DaisyUI Color Themes](https://codeberg.org/winterstein/zola-theme-daisy/raw/branch/main/daisyui-themes.png)

## Quick Start

The installation of the theme works the same as for other Zola themes. As it is described in the [official documentation](https://www.getzola.org/documentation/themes/installing-and-using-themes/). Hence, it fist needs to be added as a git submodule:

```bash
cd my-zola-website
git submodule add -b main \
    https://codeberg.org/winterstein/zola-theme-daisy.git \
    themes/daisy
```

Please make sure to add it at the path `themes/daisy` in your Zola directory. The translations and the icons won't work if added to a different directory.

As the second step, it can be enabled then in the `config.toml` file of your website:

```toml
theme = "daisy"
```

For starting to create a new Zola website using this theme, the you can also just checkout / fork the [example repository](https://codeberg.org/winterstein/zola-theme-daisy-example) and adapt it to your needs. That repository already contains a structure and configuration for the Zola-based website.

## Configuration

See the following sections for information on the possible configurations for the theme in the your `config.toml` file.

### Color Schemes

Set a light and dark color scheme:

```toml
daisyui_theme_light = "light"
daisyui_theme_dark = "dark"
```

See the `themes` list in the [`theme.toml`](theme.toml) for all possible identifiers. You can also set only a light or a dark color scheme, if you do not want the automatic dark mode switching based on the browser settings of your visitors.

If you want to allow your visitors to change the used color scheme, just set the following variable in the `[extra]` section of your `config.toml`:

```toml
[extra]
enable_theme_switching = true
```

There will be a dropdown in the navbar then, for the visitors to select form the color schemes.

### Languages

To enable support for multiple languages, simply set the default language and add language settings for all your additional languages:

```toml
default_language = "en"

[languages.de]
# title and description in the additional language
title = "Daisy Theme"
description = "Beispiel- und Demoseite des Daisy-Themas für Zola."

# don't forget to enable features like search or feed
# generation for the additional language as well
build_search_index = true
generate_feeds = true

# also any taxonomies of your default language need to
# be defined for the additional language as well
taxonomies = [
    { name = "tags", paginate_by = 2, feed = true },
    { name = "directors", paginate_by = 2, feed = true },
]
```

Taxonomies should have exactly the same (not translated) name in all languages, for the language switching to work best.

You need to create an i18n file containing the translations for all theme variables for all the languages of your website, if they are not included in the theme. Right now, [English](i18n/en.toml), [German](i18n/de.toml) and [Hungarian](i18n/hu.toml) are included. You can create a the directory `i18n` in your website root directory and the language files in there will be picked up by the theme. It would be great, however, if you create a [pull-request](https://codeberg.org/winterstein/zola-theme-daisy/pulls) on the theme repository to add your translations to the theme.

### Search

Integrating a search into your website is as easy as adding the following to your configuration:

```toml
# enable it globally for the default language
build_search_index = true

[search]
# only this format is supported by the theme
index_format = "elasticlunr_json"

# you need to enable search at all your language sections as well
[languages.de]
build_search_index = true
```

As soon as `build_search_index` is enabled, the search indices are created for all languages that have this variable enabled in their section in the `config.toml` and the search bar is shown in the navbar of the website.

Just be aware, that you need to add an [Elasticlunr.js](http://elasticlunr.com/)-compatible [Lunr Languages](https://github.com/weixsong/lunr-languages) file to your `static` directory, if you are using other languages than English and German. See the corresponding repository for the [`min` files](https://github.com/weixsong/lunr-languages/tree/master/min). Feel free to add support for your languages to the theme as well, via a [pull-request](https://codeberg.org/winterstein/zola-theme-daisy/pulls).

### Navbar

Arbitrary links can be added to the footer by defining the following list in the `[extra.navbar]` section:

```toml
[extra.navbar]
links = [
    { url = "blog", i18n_key = "posts" },
    { url = "tags", i18n_key = "tags" },
    { url = "movies", i18n_key = "movies" },
]
```

The value of the `i18n_key` must be in the `i18n` files for your languages (see [en.toml](i18n/en.toml), for example).

### Footer

All three parts of the footer can be adapted: the links, the social icons, and the copyright notice.

#### Links

Arbitrary links can be added to the footer by defining the following list in the `[extra.footer]` section:

```toml
[extra.footer]
links = [
    { url = "about", i18n_key = "about" },
    { url = "sitemap.xml", i18n_key = "sitemap", no_translation = true },
]
```

The value of the `i18n_key` must be in the `i18n` files for your languages (see [en.toml](i18n/en.toml), for example). If the parameter `no_translation` is set to true, than the URL is not adapted to contain the current language code. This is needed for external links or something like the `sitemap.xml` in the example, that is not translated within your website.

#### Social Icons

The social icons in the footer can be adapted by setting any of the following variables:

```toml
[extra.social]
codeberg = ""
github = ""
gitlab = ""
stackoverflow = ""
mastodon = ""
linkedin = ""
instagram = ""
youtube = ""
signal = ""
telegram = ""
email = ""
phone = ""
```

For every non-empty variable, the corresponding icon is shown in the footer.

#### Copyright Notice

The copyright notice in the footer can be set by adding the following variable in the configuration:

```toml
[extra.footer]
notice = "This is my <b>copyright</b> notice."
```

HTML can be used there.

## Customization

The page template can be extended with custom CSS or JavaScript files (or code) by inheriting the template and overwriting the blocks `extra_headers` or `extra_javascript`. The content of `extra_headers` will be added at the end of the `<head>` section of each page, while the content of `extra_javascript` will be added at the end of the `<body>` section of each page.

```html
{%/* extends "daisy/templates/base.html" */%}

{%/* block extra_headers */%}
<!-- add an own stylesheet for example -->
<link rel="stylesheet" href="{{/* get_url(path='my_custom_style.css') */}}">
{%/* endblock */%}

{%/* block extra_javascript */%}
<script>
    /* here is some custom JavaScript code */
</script>
{%/* endblock */%}
```

In most cases, however, you would probably not extent the `base` template, but the more specific templates like `page`, `section`, or `index`. As they are themselves derived from the `base` template you can override the `extra_headers` and `extra_javascript` blocks the same way in those cases.

        