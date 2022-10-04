
+++
title = "DeepThought"
description = "A simple blog theme focused on writing powered by Bulma and Zola."
template = "theme.html"
date = 2022-10-04T04:04:47-05:00

[extra]
created = 2022-10-04T04:04:47-05:00
updated = 2022-10-04T04:04:47-05:00
repository = "https://github.com/RatanShreshtha/DeepThought.git"
homepage = "https://github.com/RatanShreshtha/DeepThought"
minimum_version = "0.14.1"
license = "MIT"
demo = "https://deepthought-theme.netlify.app/"

[extra.author]
name = "Ratan Kulshreshtha"
homepage = "https://ratanshreshtha.dev"
+++        

<p align="center">
  <a href="https://github.com/RatanShreshtha/DeepThought">
    <img src="static/images/avatar.png" alt="Logo" width="80" height="80">
  </a>

  <h3 align="center">DeepThought</h3>

  <p align="center">
    A simple blog theme focused on writing powered by Bulma and Zola.
    <br />
    <a href="https://deepthought-theme.netlify.app/docs/"><strong>Explore the docs »</strong></a>
    <br />
    <br />
    <a href="https://github.com/RatanShreshtha/DeepThought">Code Repository</a>
    ·
    <a href="https://github.com/RatanShreshtha/DeepThought/issues">Report Bug</a>
    ·
    <a href="https://github.com/RatanShreshtha/DeepThought/issues">Request Feature</a>
  </p>
</p>

<details open="open">
  <summary><h2 style="display: inline-block">Table of Contents</h2></summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#acknowledgements">Acknowledgements</a></li>
  </ol>
</details>

## About The Project

[![DeepThought](./screenshot.png)](https://deepthought-theme.netlify.app/)

> A simple blog theme focused on writing powered by Bulma and Zola.

### Features

- [x] Dark Mode
- [x] Pagination
- [x] Search
- [x] Charts
- [x] Maps
- [x] Diagrams
- [x] Galleria
- [x] Analytics
- [x] Comments
- [x] Categories
- [x] Social Links
- [x] Multilingual Navbar
- [x] Katex

### Built With

- [Zola](https://www.getzola.org/)
- [Bulma](https://bulma.io/)

## Getting Started

To get a local copy up and running follow these simple steps.

### Prerequisites

You need static site generator (SSG) [Zola](https://www.getzola.org/documentation/getting-started/installation/) installed in your machine to use this theme follow their guide on [getting started](https://www.getzola.org/documentation/getting-started/overview/).

### Installation

Follow zola's guide on [installing a theme](https://www.getzola.org/documentation/themes/installing-and-using-themes/).
Make sure to add `theme = "DeepThought"` to your `config.toml`

**Check zola version (only 0.9.0+)**
Just to double-check to make sure you have the right version. It is not supported to use this theme with a version under 0.14.1.

## Usage

### How to serve?

Go into your sites directory and type `zola serve`. You should see your new site at `localhost:1111`.

**NOTE**: you must provide the theme options variables in `config.toml` to serve a functioning site

### Deployment

[Zola](https://www.getzola.org) already has great documentation for deploying to [Netlify](https://www.getzola.org/documentation/deployment/netlify/) or [Github Pages](https://www.getzola.org/documentation/deployment/github-pages/). I won't bore you with a regurgitated explanation.

### Theme Options

```toml
# Enable external libraries
[extra]
katex.enabled = true
katex.auto_render = true

chart.enabled = true
mermaid.enabled = true
galleria.enabled = true

navbar_items = [
 { code = "en", nav_items = [
  { url = "$BASE_URL/", name = "Home" },
  { url = "$BASE_URL/posts", name = "Posts" },
  { url = "$BASE_URL/docs", name = "Docs" },
  { url = "$BASE_URL/tags", name = "Tags" },
  { url = "$BASE_URL/categories", name = "Categories" },
 ]},
]

# Add links to favicon, you can use https://realfavicongenerator.net/ to generate favicon for your site
[extra.favicon]
favicon_16x16 = "/icons/favicon-16x16.png"
favicon_32x32 = "/icons/favicon-32x32.png"
apple_touch_icon = "/icons/apple-touch-icon.png"
safari_pinned_tab = "/icons/safari-pinned-tab.svg"
webmanifest = "/icons/site.webmanifest"

# Author details
[extra.author]
name = "DeepThought"
avatar = "/images/avatar.png"

# Social links
[extra.social]
email = "<email_id>"
facebook = "<facebook_username>"
github = "<github_username>"
gitlab = "<gitlab_username>"
keybase = "<keybase_username>"
linkedin = "<linkedin_username>"
stackoverflow = "<stackoverflow_userid>"
twitter = "<twitter_username>"
instagram = "<instagram_username>"
behance = "<behance_username>"
google_scholar = "<googlescholar_userid>"
orcid = "<orcid_userid>"
mastodon = "<mastadon_username>"


# To add google analytics
[extra.analytics]
google = "<your_gtag>"

# To add disqus comments
[extra.commenting]
disqus = "<your_disqus_shortname>"

# To enable mapbox maps
[extra.mapbox]
enabled = true
access_token = "<your_access_token>"
```

#### Multilingual Navbar

If you want to have a multilingual navbar on your blog, you must add your new code language in the [languages](https://www.getzola.org/documentation/content/multilingual/#configuration) array in the `config.toml` file.

**NOTE**: Don't add you default language to this array

```toml
languages = [
    {code = "fr"},
    {code = "es"},
]
```

And then create and array of nav item for each language:

**NOTE**: Include your default language in this array

```toml
navbar_items = [
 { code = "en", nav_items = [
  { url = "$BASE_URL/", name = "Home" },
  { url = "$BASE_URL/posts", name = "Posts" },
  { url = "$BASE_URL/docs", name = "Docs" },
  { url = "$BASE_URL/tags", name = "Tags" },
  { url = "$BASE_URL/categories", name = "Categories" },
 ]},
 { code = "fr", nav_items = [
  { url = "$BASE_URL/", name = "Connexion" },
 ]},
 { code = "es", nav_items = [
  { url = "$BASE_URL/", name = "Publicationes" },
  { url = "$BASE_URL/", name = "Registrar" },
 ]}
]
```

en:

![DeepThought](./screenshot_navbar_en.png)

fr:

![DeepThought](./screenshot_navbar_fr.png)

es:

![DeepThought](./screenshot_navbar_es.png)

### KaTeX math formula support

This theme contains math formula support using [KaTeX](https://katex.org/),
which can be enabled by setting `katex.enabled = true` in the `extra` section
of `config.toml`.

After enabling this extension, the `katex` short code can be used in documents:

- `{{/* katex(body="\KaTeX") */}}` to typeset a math formula inlined into a text,
  similar to `$...$` in LaTeX
- `{%/* katex(block=true) */%}\KaTeX{%/* end */%}` to typeset a block of math formulas,
  similar to `$$...$$` in LaTeX

#### Automatic rendering without short codes

Optionally, `\\( \KaTeX \\)` / `$ \KaTeX $` inline and `\\[ \KaTeX \\]` / `$$ \KaTeX $$`
block-style automatic rendering is also supported, if enabled in the config
by setting `katex.auto_render = true`.

### Elasticlunr search in other language

Zola use [Elasticlunr.js](https://github.com/weixsong/elasticlunr.js) to add full-text search feature.
To use languages other than en (English), you need to add some javascript files. See the Zola's issue [#1349](https://github.com/getzola/zola/issues/1349).
By placing the `templates/base.html`on your project and using the `other_lang_search_js` block, you can load the required additional javascript files in the right timing.

e.g. `templates/base.html`

```html
{%/* extends "DeepThought/templates/base.html" */%} {%/* block other_lang_search_js */%}
<script src="{{/* get_url(path='js/lunr.stemmer.support.js') */}}"></script>
<script src="{{/* get_url(path='js/tinyseg.js') */}}"></script>
<script src="{{/* get_url(path='js/lunr.' ~ lang ~ '.js') */}}"></script>
<script src="{{/* get_url(path='js/search.js') */}}"></script>
{%/* endblock */%}
```

More detailed explanations are aound in [elasticlunr's documents](https://github.com/weixsong/elasticlunr.js#other-languages-example-in-browser).

## Roadmap

See the [open issues](https://github.com/RatanShreshtha/DeepThought/issues) for a list of proposed features (and known issues).

## Contributing

Contributions are what make the open source community such an amazing place to be learn, inspire, and create. Any contributions you make are **greatly appreciated**.

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

Distributed under the MIT License. See `LICENSE` for more information.

## Contact

Ratan Kulshreshtha - [@RatanShreshtha](https://twitter.com/RatanShreshtha)>

Project Link: [https://github.com/RatanShreshtha/DeepThought](https://github.com/RatanShreshtha/DeepThought)

## Acknowledgements

- [GitHub Emoji Cheat Sheet](https://www.webpagefx.com/tools/emoji-cheat-sheet)
- [Choose an Open Source License](https://choosealicense.com)
- [Slick Carousel](https://kenwheeler.github.io/slick)
- [Font Awesome](https://fontawesome.com)
- [Unsplash](https://unsplash.com/)

        