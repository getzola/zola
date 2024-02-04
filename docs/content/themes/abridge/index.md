
+++
title = "abridge"
description = "A fast and lightweight Zola theme using semantic html, a class-light abridge.css, and No mandatory JS."
template = "theme.html"
date = 2024-01-25T10:41:35+02:00

[extra]
created = 2024-01-25T10:41:35+02:00
updated = 2024-01-25T10:41:35+02:00
repository = "https://github.com/Jieiku/abridge.git"
homepage = "https://github.com/jieiku/abridge"
minimum_version = "0.18.0"
license = "MIT"
demo = "https://abridge.netlify.app/"

[extra.author]
name = "Jake G (jieiku)"
homepage = "https://github.com/jieiku/"
+++        

<div align="center">
<img src="https://raw.githubusercontent.com/Jieiku/abridge/master/abridge.svg"/>

# Abridge Zola Theme

A fast, lightweight, and modern [Zola](https://getzola.org) theme utilizing [abridge.css](https://github.com/Jieiku/abridge.css) (a class-light semantic HTML CSS Framework). Perfect [Lighthouse](https://pagespeed.web.dev/report?url=abridge.netlify.app), [YellowLabTools](https://yellowlab.tools/), and [Observatory](https://observatory.mozilla.org/analyze/abridge.netlify.app) scores. Here is a [Zola Themes Benchmarks](https://github.com/Jieiku/zola-themes-benchmarks/blob/main/README.md) Page.

![Lighthouse Score](https://raw.githubusercontent.com/Jieiku/abridge/master/content/overview-abridge/lighthouse.png)

Maintenance of this project is made possible by all the <a href="https://github.com/Jieiku/abridge/graphs/contributors">contributors</a> and <a href="https://github.com/sponsors/Jieiku">sponsors</a>. If you'd like to sponsor this project and have your avatar or company logo appear below <a href="https://github.com/sponsors/Jieiku">click here</a>. 💖

<!-- sponsors --><!-- sponsors -->

---

**[View Abridge demo](https://abridge.netlify.app/overview-abridge/)**

**[View Abridge.css demo](https://abridgecss.netlify.app/overview-abridge/)** [[abridge.css framework](https://github.com/Jieiku/abridge.css/tree/master/dist)]

The Abridge.css demo is simply using Abridge theme as a submodule: [config.toml](https://github.com/Jieiku/abridge.css/blob/master/config.toml), [sass/abridge.scss](https://github.com/Jieiku/abridge.css/blob/master/sass/abridge.scss)
</div>

## Features

- [X] Perfect [Lighthouse](https://pagespeed.web.dev/report?url=abridge.netlify.app), [YellowLabTools](https://yellowlab.tools/), and [Observatory](https://observatory.mozilla.org/analyze/abridge.netlify.app) scores.
- [X] [PWA support](https://abridge.netlify.app/overview-abridge/#pwa-progressive-web-app) (Progressive Web Application).
- [X] All JavaScript can be [fully disabled](https://abridge.netlify.app/overview-abridge/#javascript-files).
- [X] Dark, Light, Auto, and Switcher themes. (colors can be customized, css variables)
- [X] Code [syntax highlighting](https://abridge.netlify.app/overview-code-blocks/). (colors can be customized, css variables)
- [X] Numbered code blocks with [line highlighting](https://abridge.netlify.app/overview-code-blocks/#toml).
- [X] Entirely Offline Site by using the PWA **or** by setting `offline = true` in `config.toml` (full search support).
- [X] Multi-language support.
- [X] Search support. (elasticlunr, tinysearch, stork)
- [X] Search Suggestions navigation keys, `/` focus, `arrow` move, `enter` select, `escape` close.
- [X] Search Results Page, type search query then hit `Enter Key` or `click` the search button icon.
- [X] [SEO](https://abridge.netlify.app/overview-abridge/#seo-and-header-tags) support. (Search Engine Optimization)
- [X] [Pagination](https://abridge.netlify.app/overview-abridge/#pagination) with numbered paginator on index.
- [X] Title Based Previous and Next Article links at bottom of Article.
- [X] Table of Contents in page Index (Optional, clickable links)
- [X] Recent Posts Block. (Optional)
- [X] Back to Top button. (uses css only)
- [X] Code Blocks copy button.
- [X] Email link in footer obfuscation. (anti-spam)
- [X] [KaTeX](https://katex.org/) support.
- [X] [Archive page](https://abridge.netlify.app/archive/).
- [x] [Tags](https://abridge.netlify.app/tags/).
- [x] Categories. (similar to Tags, disabled/commented out by default)
- [x] Social icon links in footer.
- [X] Responsive design. (mobile first)
- [X] Video Shortcodes: [Youtube](https://abridge.netlify.app/video-streaming-sites/overview-embed-youtube/), [Vimeo](https://abridge.netlify.app/video-streaming-sites/overview-embed-vimeo/), [Streamable](https://abridge.netlify.app/video-streaming-sites/overview-embed-streamable/).
- [X] Media Shortcodes: [video](https://abridge.netlify.app/overview-rich-content/#video), [img](https://abridge.netlify.app/overview-images/#img-shortcode), [imgswap](https://abridge.netlify.app/overview-images/#imgswap-shortcode), [image](https://abridge.netlify.app/overview-rich-content/#image), [gif](https://abridge.netlify.app/overview-rich-content/#gif), [audio](https://abridge.netlify.app/overview-rich-content/#audio).
- [X] Other Shortcodes: [showdata](https://abridge.netlify.app/overview-showdata/), [katex](https://abridge.netlify.app/overview-math/#usage-1).

**[Complete Documentation is available here](https://abridge.netlify.app/overview-abridge/)**

## Quick Start

This theme requires version 0.18.0 or later of [Zola](https://www.getzola.org/documentation/getting-started/installation/)

```bash
git clone https://github.com/jieiku/abridge.git
cd abridge
zola serve
# open http://127.0.0.1:1111/ in the browser
```

## Installation

The Quick Start shows how to run the theme directly. Next we will use abridge as a theme to a NEW site.

### 1: Create a new zola site

```bash
yes "" | zola init mysite
cd mysite
```

### 2: Install Abridge

Add the theme as a git submodule:

```bash
git init  # if your project is a git repository already, ignore this command
git submodule add https://github.com/jieiku/abridge.git themes/abridge
git submodule update --init --recursive
git submodule update --remote --merge
```

Or clone the theme into your themes directory:

```bash
git clone https://github.com/jieiku/abridge.git themes/abridge
```

### 3: Configuration

Copy some files from the theme directory to your project's root directory:

```bash
rsync themes/abridge/.gitignore .gitignore
rsync themes/abridge/config.toml config.toml
rsync themes/abridge/content/_index.md content/
rsync -r themes/abridge/COPY-TO-ROOT-SASS/* sass/
rsync themes/abridge/netlify.toml netlify.toml
rsync themes/abridge/package_abridge.js package_abridge.js
rsync themes/abridge/package.json package.json
```

- `config.toml` base configuration with all config values.
- `content/_index.md` required to set pagination.
- `COPY-TO-ROOT-SASS/abridge.scss` overrides to customize Abridge variables.
- `netlify.toml` settings to deploy your repo with netlfiy.
- `package_abridge.js` node script to: update cache files list in PWA, minify js, bundle js
- `package.json` to switch between nosearch, elasticlunr, tinysearch, stork.

Uncomment the theme line in your project's root config.toml:

```bash
sed -i 's/^#theme = "abridge"/theme = "abridge"/' config.toml
```

### 4: Add new content

Copy the content from the theme directory to your project or make a new post:

```bash
rsync -r themes/abridge/content .
```

### 5: Run the project

Just run `zola serve` in the root path of the project:

```bash
zola serve
```

Zola will start the dev web server, accessible by default at `http://127.0.0.1:1111`.

Saved changes will live reload in the browser. (press `ctrl+f5`, or while developing set `pwa=false` in `config.toml`)

## Customization

For further customization be sure to [check the docs](https://abridge.netlify.app/overview-abridge/).

## Sponsor

Do you love this theme? Was it useful to you? Please leave a github star, and if you feel inclined to donate you can make a donation to me through [github sponsors](https://github.com/sponsors/Jieiku/).

## Contributing and Philosophy

We'd love your help! Especially with fixes to issues, or improvements to existing features.

The goal is for Abridge to be lightweight, fast, and to work properly even if javascript is disabled or blocked.

The only feature that may be considered a necessity that relies on javascript is the Search.

## License

**Abridge** is distributed under the terms of the [MIT license](https://github.com/jieiku/abridge/blob/master/LICENSE).

        