
+++
title = "zola.386"
description = "Zola port of the BOOTSTRA.386 theme."
template = "theme.html"
date = 2021-02-18T22:27:50+01:00

[extra]
created = 2021-02-18T22:27:50+01:00
updated = 2021-02-18T22:27:50+01:00
repository = "https://github.com/lopes/zola.386"
homepage = "https://github.com/lopes/zola.386"
minimum_version = "0.10.1"
license = "MIT"
demo = "https://zola-386.netlify.com"

[extra.author]
name = "José Lopes"
homepage = "https://github.com/lopes"
+++        

# ZOLA.386

![ZOLA.386 screenshot](https://github.com/lopes/zola.386/blob/master/screenshot.png?raw=true)

## [Live demo](https://zola386.netlify.app/)

ZOLA.386 is a port of the BOOTSTRA.386 theme and was based on:

- [BOOTSTRA.386](https://kristopolous.github.io/BOOTSTRA.386/): main idea, design.
- [HUGO.386](https://themes.gohugo.io/hugo.386/): item placement.
- [Dinkleberg](https://github.com/rust-br/dinkleberg): internal structure and SEO.
- [after-dark](https://github.com/getzola/after-dark): navbar and minor components.

ZOLA.386 is a theme that refers to the 90s, but with cutting edge features to be fast and responsive.


## Installation
The easiest way to install ZOLA.386 is to clone this repository and build your site upon it:

```bash
$ git clone https://github.com/lopes/zola.386
```

Of course you can install it just as another theme for your site, but ZOLA.386 must be added as a module:

```bash
$ cd themes
$ git submodule add https://github.com/lopes/zola.386 
```


## Configuration
Configuration is mainly done in `config.toml` and here I'll describe the main topics.

### Global
`config.toml` starts with the global variables.  All of these items are important, but it is fundamental to create two taxonomies at least:

```toml
taxonomies = [
  {name="categories", rss=true},
  {name="tags", rss=true},
]
```

Remember that all descriptions (`config.description` and `page.description`) are shown on the index page, one at the header and the others through the body.

### Extras
ZOLA.386 comes with a lot of extra variables which eases the creation and maintenance of the site, so it's important to review all of them after installing the theme.

The `zola386_menu` composes the navbar and is created by setting up a `path`, which will be appended to the `base_url` and the `name` will appear on the navbar.

```toml
zola386_menu = [
  {path="/", name="Home"},
  {path="categories", name="Categories"},
  {path="tags", name="Tags"},
  {path="about", name="About"},
]
```

### Social
ZOLA.386 is also prepared to deal with Google Analytics, Disqus, and Twitter --[Open Graph Protocol](https://ogp.me/) is welcome.  This theme is prepared to use the output of [Favicon Generator](https://www.favicon-generator.org/), to do so, you'll just need to download the output of that site and extract in `static/images`. 

As said, Disqus is supportted, but besides setting the username in `config.toml`, you also must to put a `comments = true` extra option on the pages where Disqus will be enabled --this gives you the freedom to enable or disable comments on certain posts.  You can use the extra option `image` on each page, to represent that post.

### Animations
All JavaScript animations can be set at `static/js/zola386.js`.  Basically you can disable all animations, use one or two scans, and change the scan speed.  Personally, I prefer only one scan with a speed factor of 5.

### Language
Under the `label_` variables, you can set names to better localize your site.  Note that you can change the language of a single page, by using `page.extra.lang`, which causes `<html lang="">` to change only on that page.  A theme to provide information for its owner and SEO-friendly.

### Search
Search was implemented according to the [official documentation](https://www.getzola.org/documentation/content/search/).  It uses JavaScript to search on an indexed version of the site based on `search_index.LANG.js`, `elasticlunr.min.js`, and `search.js` --the first two are generated after each build.  If you're running your site in other default language other than English, you **must** change the `search_index.LANG.js` line in `index.html`, setting up `LANG` accordingly.

### Other files
The `content\_index.md` file must be properly configured to provide better experience.  Check out this file for more information.

The 404 page is almost hardcoded, so you must edit it directly.  


## License
This theme is released under the MIT license.  For more information read the [License](https://github.com/lopes/zola.386/blob/master/LICENSE).


[![Netlify Status](https://api.netlify.com/api/v1/badges/5d6f1986-7bf3-40d3-b298-3339288585d4/deploy-status)](https://app.netlify.com/sites/zola386/deploys)

        