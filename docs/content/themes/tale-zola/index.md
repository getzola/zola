
+++
title = "tale-zola"
description = "Tala-Zola is a minimal Zola theme helping you to build a nice and seo-ready blog."
template = "theme.html"
date = 2024-01-09T08:33:22+01:00

[extra]
created = 2024-01-09T08:33:22+01:00
updated = 2024-01-09T08:33:22+01:00
repository = "https://github.com/aaranxu/tale-zola.git"
homepage = "https://github.com/aaranxu/tale-zola"
minimum_version = "0.13.0"
license = "MIT"
demo = "https://tale-zola.netlify.app/"

[extra.author]
name = "Aaran Xu"
homepage = "https://github.com/aaranxu"
+++        

# Tale-Zola Theme


Tala-Zola is a minimal [Zola](https://www.getzola.org) theme helping you to
build a light and seo-ready blog, and you can customise any information of the
blog without having to modify the codes of the template. Tala-Zola is a port of
the Jekyll theme [Tale](https://github.com/chesterhow/tale).


## Demo

[Live Preview](https://tale-zola.netlify.app/).

## Requirements

Before using the theme, you need to install the [Zola](https://www.getzola.org/documentation/getting-started/installation/) ≥ 0.13.0.

## Quick Start

```bash
git clone git@github.com:aaranxu/tale-zola.git
cd tale-zola
zola serve
# open http://127.0.0.1:1111/ in the browser
```

## Installation

Just earlier we showed you how to run the theme directly. Now we start to
install the theme in an existing site step by step.

### Step 1: Create a new zola site

```bash
zola init blog
```

### Step 2: Install Tale-Zola

Download this theme to your themes directory:

```bash
cd blog/themes
git clone git@github.com:aaranxu/tale-zola.git
```

Or install as a submodule:

```bash
cd blog
git init  # if your project is a git repository already, ignore this command
git submodule add git@github.com:aaranxu/tale-zola.git themes/tale-zola
```

### Step 3: Configuration

Enable the theme in your `config.toml` in the site derectory:

```toml
theme = "tale-zola"
```

Or copy the `config.toml.example` from the theme directory to your project's
root directory:

```bash
cp themes/tale-zola/config.toml.example config.toml
```

### Step 4: Add new content

Add an `_index.md` file to your `content` directory with some lines as bellows.

```text
+++
sort_by = "date"
paginate_by = 5
+++
```

Add a blog article file with a filename `first-post.md` (or other filenames) and
input some content in it.

```text
+++
title = "First Post"
date = 2021-05-01T18:18:18+00:00

[taxonomies]
tags = ["Post"]

[extra]
author = "Your Name"
+++

This is my first post.
```


Or you can just copy the content from the theme directory to your project:

```bash
cp -r themes/tale-zola/content .
```

### Step 5: Run the project

Just run `zola serve` in the root path of the project:

```bash
zola serve
```

Tale-Zola will start the Zola development web server accessible by default at
`http://127.0.0.1:1111`. Saved changes will live reload in the browser.

## Customisation

You can customize your configurations, templates and content for yourself. Look
at the `config.toml`, `theme.toml` and templates files in this repo for an idea.

In most cases you only need to modify the content in the `config.toml` file to
custom your blog, including different expressions in your speaking language.

### Necessary Configurations

Add some information for your blog.

```toml
title = "You Blog Title"
description = "The description of your blog."
```

Set the tags for the site.

```toml
taxonomies = [
  {name = "tags"},
]
```

Add menus and footer information for your blog.

```
# Menu items
[[extra.menu]]
name = "Posts"
url = "/"

[[extra.menu]]
name = "Tags"
url = "tags"

[[extra.menu]]
name = "About"
url = "about"

[extra.footer]
start_year = "2020"  # start year of the site
end_year = "2021"    # end year of the site
info = "The information on the footer."
```

#### Option Configurations

Add your name as the author name for the blog globally.

```toml
[extra]
author = "Your Name"
```

Use Google Analytics. Add your own Google Analytics ID.

```toml
[extra]
google_analytics = "UA—XXXXXXXX-X"
```

Whether to use Disqus globally and set to your disqus id name.
And you can enable the disqus on per post page with `extra.disqus` option

```toml
[extra]
disqus = false
disqus_id = ""
```

Code syntax highlighting. See also [syntax highlighting](https://www.getzola.org/documentation/getting-started/configuration/#syntax-highlighting).

```toml
[markdown]
highlight_code = true
highlight_theme = "base16-ocean-light"
```

Use KaTeX to support the math notation

```toml
[extra]
katex = true
```

> Note: You can also add the `katex` option on per mardown file of the page or section.

Set date format in the site

```toml
[extra]
timeformat = "%B %e, %Y" # e.g. June 14, 2021, and this is the default format
```

SEO settings, like Open Graph + Twitter Cards

```toml
[extra.seo]
# this image will be used as fallback if a page has no image of its own
image = "tale.png"
image_height = 50
image_width = 50
og_locale = "en_US"

  [extra.seo.twitter]
  site = "twitter_accout"
  creator = "twitter_accout"

  [extra.seo.facebook]
  admins = "facebook_accout"
  publisher = "facebook_accout"
```

Change the words in your speaking language.

```toml
[extra.expressions]
home = "Home"              # The homepage's name
pinned = "Pinned"          # On the header of the post list
written_by = "Written by"  # Like: Written by Aaran Xu
on = "on"                  # Like: on May 3, 2021
top = "Top"                # Go to the top of the page in the post
tags = "Tags"              # In the page of Tags

# disqus comments block
disqus_discussion = "Discussion and feedback"

# The contents of the 404 page
p404 = "404: Page not found"
p404_info = "Oops! We can't seem to find the page you are looking for."
p404_back_home_start = "Let's"
p404_back_home_with_link = "head back home"
p404_back_home_end = "."
```

### Custom CSS styles

Just add your own styles to `sass/_custom.scss` file.

## Reporting Issues

We use GitHub Issues as the official bug tracker for the **Tale-Zola**. Please
search [existing issues](https://github.com/aaranxu/tale-zola/issues). It’s
possible someone has already reported the same problem.

If your problem or idea is not addressed yet, [open a new issue](https://github.com/aaranxu/tale-zola/issues/new).

## Contributing

We'd love your help! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) to learn
about the kinds of contributions we're looking for.

## License

Tale-Zola is distributed under the terms of the
[MIT license](LICENSE).

        