
+++
title = "Zulma"
description = "A zola theme based off bulma.css"
template = "theme.html"
date = 2021-02-18T22:27:50+01:00

[extra]
created = 2021-02-18T22:27:50+01:00
updated = 2021-02-18T22:27:50+01:00
repository = "https://github.com/Worble/Zulma"
homepage = "https://github.com/Worble/Zulma"
minimum_version = "0.6.0"
license = "MIT"
demo = "https://festive-morse-47d46c.netlify.com/"

[extra.author]
name = "Worble"
homepage = ""
+++        

# Zulma

A Bulma theme for Zola. See a live preview [here](https://festive-morse-47d46c.netlify.com/)

![Zulma Screenshot](/screenshot.png)

## Contents

- [Zulma](#zulma)
  - [Contents](#contents)
  - [Installation](#installation)
  - [Javascript](#javascript)
    - [Sources](#sources)
    - [Building](#building)
  - [Options](#options)
    - [Pagination](#pagination)
    - [Taxonomies](#taxonomies)
    - [Menu Links](#menu-links)
    - [Brand](#brand)
    - [Search](#search)
    - [Title](#title)
    - [Theming](#theming)
  - [Original](#original)
  - [Known Bugs](#known-bugs)

## Installation

First download this theme to your `themes` directory:

```bash
cd themes
git clone https://github.com/Worble/Zulma
```

and then enable it in your `config.toml`:

```toml
theme = "Zulma"
```

That's it! No more configuration should be required, however it might look a little basic. Head to the [Options](#options) section to see what you can set for more customizability.

## Javascript

### Sources

All the source javascript files live in `javascript/src`. Following is a list of the javascript files, their purpose, and their sources. All files are prefixed with `zulma_` to avoid any name clashes.

- `zulma_search.js` - Used when a user types into the search box on the navbar (if enabled). Taken from [Zola's site](https://github.com/getzola/zola/blob/6100a43/docs/static/search.js).
- `zulma_navbar.js` - Used for the mobile navbar toggle. Taken from the [bulma template](https://github.com/dansup/bulma-templates/blob/6263eb7/js/bulma.js) at Bulmaswatch
- `zulma_switchcss.js` - Used for swapping themes (if enabled).

### Building

The JavaScript files are transpiled by babel, minified by webpack, sourcemaps are generated and then everything placed in `static/js`. The repo already contains the transpiled and minified files along with their corrosponding sourcemaps so you don't need to do anything to use these. If you would prefer to build it yourself, feel free to inspect the js files and then run the build process (please ensure that you have [node, npm](https://nodejs.org/en/) and optionally [yarn](https://yarnpkg.com/lang/en/) installed):

```bash
cd javascript
yarn
yarn webpack
```

### Github warnings

You may get warnings about vulnerabilities from the JavaScript dependencies. These shouldn't be an issue since we only have dev-dependencies and none of the them reach the end-user, but if you don't want to run the buld process yourself, and to stop Github pestering you about security warnings, feel free to delete the top level `javascript` folder when committing.

## Options

### Pagination

Zulma makes no assumptions about your project. You can freely paginate your content folder or your taxonomies and it will adapt accordingly. For example, editing or creating section (`content/_index.md`) and setting pagination:

```toml
paginate_by = 5
```

This is handled internally, no input is needed from the user.

### Taxonomies

Zulma has 3 taxonomies already set internally: `tags`, `cateogories` and `authors`. Setting of any these three in your config.toml like so:

```toml
taxonomies = [
    {name = "categories"},
    {name = "tags", paginate_by = 5, rss = true},
    {name = "authors", rss = true},
]
```

and setting any of them in a content file:

```toml
[taxonomies]
categories = ["Hello world"]
tags = ["rust", "ssg", "other", "test"]
authors = ["Joe Bloggs"]
```

will cause that metadata to appear on the post, either on the header for the name, or at the bottom for tags and categories, and enable those pages.

Making your own taxonomies is also designed to be as easy as possible. First, add it to your cargo.toml

```toml
taxonomies = [
    {name = "links"},
]
```

and make the corrosponding folder in your templates, in this case: `templates\links`, and the necessary files: `templates\links\list.html` and `templates\links\single.html`

And then for each, just inherit the taxonomy master page for that page. Before rendering the content block, you may optionally set a variable called `title` for the hero to display on that page, otherwise it will use the default for that taxonomy.

In `single.html`:

```jinja
{%/* extends "Zulma/templates/taxonomy_single.html" */%}
```

In `list.html`:

```jinja
{%/* extends "Zulma/templates/taxonomy_list.html" */%}

{%/* block content */%}
{%/* set title = "These are all the Links"*/%}
{{/* super() */}}
{%/* endblock content */%}
```

### Menu Links

In extra, setting `zulma_menu` with a list of items will cause them to render to the top menu bar. It has two paramers, `url` and `name`. These _must_ be set. If you put \$BASE_URL in a url, it will automatically be replaced by the actual site URL. This is the easiest way to allow users to navigate to your taxonomies:

```toml
[extra]
zulma_menu = [
    {url = "$BASE_URL/categories", name = "Categories"},
    {url = "$BASE_URL/tags", name = "Tags"},
    {url = "$BASE_URL/authors", name = "Authors"}
]
```

On mobile, a dropdown burger is rendered using javascript. If the page detects javascript is disabled on the clients machine, it will gracefully degrade to always showing the menu (which isn't pretty, but keeps the site functional).

### Brand

In extra, setting `zulma_brand` will cause a brand image to display in the upper left of the top menu bar. This link will always lead back to the homepage. It has two parameters, `image`(optional) and `text`(required). `image` will set the brand to an image at the location specified, and `text` will provide the alt text for this image. If you put \$BASE_URL in a url, it will automatically be replaced by the actual site URL. If `image` is not set, the brand will simply be the text specified.

```toml
[extra]
zulma_brand = {image = "$BASE_URL/images/bulma.png", text = "Home"}
```

### Search

Zulma provides search built in. So long as `build_search_index` is set to `true` in `config.toml` then a search input will appear on the top navigation bar. This requires javascript to be enabled to function; if the page detects javascript is disabled on the clients machine, it will hide itself.

The search is shamefully stolen from [Zola's site](https://github.com/getzola/zola/blob/master/docs/static/search.js). Thanks, Vincent!

### Title

In extra, setting `zulma_title` will set a hero banner on the index page to appear with that title inside.

```toml
[extra]
zulma_title = "Blog"
```

If you want to get fancy with it, you can set an image behind using sass like so:

```scss
.index .hero-body {
  background-image: url(https://upload.wikimedia.org/wikipedia/commons/thumb/f/f6/Plum_trees_Kitano_Tenmangu.jpg/1200px-Plum_trees_Kitano_Tenmangu.jpg);
  background-position: center;
  background-size: cover;
  background-repeat: no-repeat;
  background-color: rgba(0, 0, 0, 0.6);
  background-blend-mode: overlay;
}
```

This will set the image behind the hero, and darken it so the main text can still be easily read.

### Theming

In extra, setting `zulma_theme` to a valid value will change the current colour scheme to that one. All themes were taken from [Bulmaswatch](https://jenil.github.io/bulmaswatch/). Valid theme values are:

- default
- darkly
- flatly
- pulse
- simplex
- lux
- slate
- solar
- superhero

All valid themes can also be found under the `extra.zulma_themes` variable in the `theme.toml`. Choosing no theme will set default as the theme. Setting an invalid theme value will cause the site to render improperly.

```toml
[extra]
zulma_theme = "darkly"
```

Additionally, in extra, you can also set the `zulma_allow_theme_selection` boolean. Setting this to `true` will allow a menu in the footer to allow users to select their own theme. This option will store their theme choice in their localstorage and apply it on every page, assuming `zulma_allow_theme_selection` is still true. This requires javascript to be enabled to function; if the page detects javascript is disabled on the clients machine, it will hide itself.

Each theme contains the entirety of Bulma, and will weigh in at ~180kb. If you're running on a server severely limited on space, then I'd recommend you delete each theme you're not using, either from the source or from `/public`. Obviously, doing this will cause `zulma_allow_theme_selection` to work improperly, so make sure you either override `extra.zulma_themes` in `config.toml` to only show themes you have left or to not enable this option at all.

```toml
[extra]
zulma_allow_theme_selection = true
```

## Original

This template is based on the [blog template](https://bulmatemplates.github.io/bulma-templates/templates/blog.html) over at [Free Bulma Templates](https://bulmatemplates.github.io/bulma-templates/). All themes were taken from [Bulmaswatch](https://jenil.github.io/bulmaswatch/). The code behind from originally adapted from the [after-dark](https://github.com/getzola/after-dark/blob/master/README.md) zola template.

## Known Bugs

- If user theme swapping is enabled and the user selects a theme different to the default, a slight delay will be introduced in page rendering as the css gets swapped out and in by the javascript. This is particularly pronounced when using the dark theme, since it will flash white before going back to black. This is better than the alternative flashes of unstyled content or old theme, but still annoying. I don't know any way around this, but with browser caching it should be fast enough to not cause serious issues.

        