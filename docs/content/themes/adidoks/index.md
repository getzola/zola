
+++
title = "adidoks"
description = "AdiDoks is a Zola theme helping you build modern documentation."
template = "theme.html"
date = 2024-01-09T08:33:22+01:00

[extra]
created = 2024-01-09T08:33:22+01:00
updated = 2024-01-09T08:33:22+01:00
repository = "https://github.com/aaranxu/adidoks.git"
homepage = "https://github.com/aaranxu/adidoks"
minimum_version = "0.15.0"
license = "MIT"
demo = "https://adidoks.netlify.app/"

[extra.author]
name = "Aaran Xu"
homepage = "https://github.com/aaranxu"
+++        

# Zola Theme AdiDoks

AdiDoks is a modern documentation theme, which is a port of the Hugo
theme [Doks](https://github.com/h-enk/doks) for Zola.

## Demo

[Live Preview](https://adidoks.netlify.app/).

## Requirements

Before using the theme, you need to install the [Zola](https://www.getzola.org/documentation/getting-started/installation/) ≥ 0.15.0.

## Quick Start

```bash
git clone https://github.com/aaranxu/adidoks.git
cd adidoks
zola serve
# open http://127.0.0.1:1111/ in the browser
```

Read more from [the document of the AdiDoks](https://adidoks.org/docs/getting-started/introduction/).

## Installation

Just earlier we showed you how to run the theme directly. Now we start to
install the theme in an existing site step by step.

### Step 1: Create a new zola site

```bash
zola init mysite
```

### Step 2: Install AdiDoks

Download this theme to your themes directory:

```bash
cd mysite/themes
git clone https://github.com/aaranxu/adidoks.git
```

Or install as a submodule:

```bash
cd mysite
git init  # if your project is a git repository already, ignore this command
git submodule add https://github.com/aaranxu/adidoks.git themes/adidoks
```

### Step 3: Configuration

Enable the theme in your `config.toml` in the site directory:

```toml
theme = "adidoks"
```

Or copy the `config.toml.example` from the theme directory to your project's
root directory:

```bash
cp themes/adidoks/config.toml.example config.toml
```

### Step 4: Add new content

You can copy the content from the theme directory to your project:

```bash
cp -r themes/adidoks/content .
```

You can modify or add new posts in the `content/blog`, `content/docs` or other
content directories as needed.

### Step 5: Run the project

Just run `zola serve` in the root path of the project:

```bash
zola serve
```

AdiDoks will start the Zola development web server accessible by default at
`http://127.0.0.1:1111`. Saved changes will live reload in the browser.

## Customisation

You can customize your configurations, templates and content for yourself. Look
at the `config.toml`, `theme.toml`, `content` files and templates files in this
repo for an idea.

### Global Configuration

There are some configuration options that you can customize in `config.toml`.

#### Configuration options before `extra` options

Set the authors's taxonomies for the site.

```toml
taxonomies = [
  {name = "authors"},
]
```

Use search function for the content.

```toml
build_search_index = true
```

#### Configuration options under the `extra`

The following options should be under the `[extra]` in `config.toml`

- `language_code` - set HTML file language (default to `en-US`)
- `theme_color` - your site's HTML color (default to `#fff`)
- `title_separator` - the separator to your site title, like `|` and `-` (defaults to `|`)
- `title_addition` - the additon content for the title of the homepage
- `timeformat` - the timeformat for the blog article published date
- `timezone` - the timezone for the blog article published date
- `edit_page` (and `docs_repo`, `repo_branch`) - whether to show the edit page in the github repo for your docs
- `math` (and `library`) - set KaTeX or MathJax library
- `[extra.open]` - Open Graph + Twitter Cards for the site
- `[extra.schema]` - set JSON-LD for the site
- `[[extra.menu.main]]` - the header navigations for the site
- `[[extra.menu.social]]` - the social links on the header of the page
- `[extra.footer]` - the footer content on the left
- `[[extra.footer.nav]]` - the footer navigations on the right

### Templates

All pages are extend to the `base.html`, and you can customize them as need.

### Content

#### Homepage

Go to the *content/_index.md* file to add your own homepage content.

- `[extra]` - the main content of the homepage
- `[[extra.ist]]` - the lists' content of the homepage

#### Sections

Each section includes a `_index.md`, and you can customize it or add your new
section under the `content` folder.

#### Pages

There are mainly three types of pages in the site.

- `blog` - blog article
- `docs` - documentation article
- `authors` - authors page if you need to add some information for a new author

## Reporting Issues

We use GitHub Issues as the official bug tracker for the **AdiDoks**. Please
search [existing issues](https://github.com/aaranxu/adidoks/issues). It’s
possible someone has already reported the same problem.

If your problem or idea is not addressed yet, [open a new issue](https://github.com/aaranxu/adidoks/issues/new).

## Contributing

We'd love your help! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) to learn
about the kinds of contributions we're looking for.

## License

**AdiDoks** is distributed under the terms of the
[MIT license](https://github.com/aaranxu/adidoks/blob/main/LICENSE).

        