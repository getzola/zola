# Gutenberg
[![Build Status](https://travis-ci.org/Keats/gutenberg.svg?branch=master)](https://travis-ci.org/Keats/gutenberg)
[![Build status](https://ci.appveyor.com/api/projects/status/h4t9r6h5gom839q0/branch/master?svg=true)](https://ci.appveyor.com/project/Keats/gutenberg/branch/master)

An opinionated static site generator written in Rust.

## Installation
You can get the latest release by going to the [Release page](https://..).
Alternatively, if you have the rust toolchain on your computer, you can also install it
through Cargo: `cargo install gutenberg`.

## Usage

### Creating a new site
Use `gutenberg init <a_directory_name>`. 
This will create a folder with the name given and the base structure of a gutenberg site.

### Working on a site
Use `gutenberg serve` to spin up a server that will automatically live reload any changes to the 
content, templates or static files.

### Building a site
Use `gutenberg build` to generate the site in the `public/` directory.

### Gutenberg terms
Some words are going to be repeated in the docs so let's make sure they are clear.

- Page: a markdown file in the `content` directory that has a name different from `_index.md`
- Section: a group of pages in the `content` directory that has `_index.md` in the same folder

### Configuration
Configuration is using the [TOML](https://github.com/toml-lang/toml) language.
Only 2 parameters are required: `title` and `base_url`.
The other options are:

- `highlight_code`: Whether to highlight all code blocks found in markdown files. Defaults to false
- `highlight_theme`: Which themes to use for code highlighting. Defaults to "base16-ocean-dark"
- `language_code`: The language used in the site. Defaults to "en"
- `generate_rss`: Whether to generate RSS, defaults to false
- `generate_tags_pages`: Whether to generate tags and individual tag pages if some pages have them. Defaults to true
- `generate_categories_pages`: Whether to generate categories and individual category categories if some pages have them. Defaults to true

If you want to add some of your own variables, you will need to put them in the `[extra]` table in `config.toml` or
they will be silently ignored.

### Templates
Templates live in the `templates/` directory.
Only [Tera](https://github.com/Keats/tera) templates are supported.

Each kind of page get their own variables:

// TODO: detail the schema of the variables

- index.html: gets `pages` that contain all pages in the site
- page.html: gets `page` that contains the data for that page 
- section.html: gets `section` that contains the data for pages in it and its subsections
- tags.html: gets `tags`
- tag.html: gets `tag` and `pages`
- categories.html: gets `categories`
- category.html: gets `category` and `pages`

Additionally, all pages get a `config` variable representing the data in `config.toml`.
If you want to know all the data present in a template content, simply put `{{ __tera_context }}`
in the templates and it will print it.

### Static files
Everything in the `static` folder will be copied into the output directory as-is.

### Pages
Pages have to start with a front-matter enclosed in `+++`. Here is a minimal example:

```md
+++
title = "My page"
description = "Some meta info"
+++

A simple page with fixed url
```

A front-matter requires a title, a description and has the following optional variables:

- date: a YYYY-MM-DD or RFC339 formatted date
- slug: what slug to use in the url
- url: this overrides the slug and make this page accessible at `{config.base_url}/{url}`
- tags: an array of strings
- category: only one category is allowed
- draft: whether the post is a draft or not
- template: if you want to change the template used to render that specific page

You can also, like in the config, add your own variables in a `[extra]` table.
The front-matter will be accessible in templates at the `page.meta` field.

By default, the URL of a page will follow the filesystem paths. For example, if you have
a page at `content/posts/python3.md`, it will be available at `{config.base_url}/posts/python3/`.
You can override the slug created from the filename by setting the `slug` variable in the front-matter.

Quite often, a page will have assets and you might want to co-locate them with the markdown file.
Gutenberg supports that pattern out of the box: you can create a folder, put a file named `index.md` and any number of files
along with it that are NOT markdown.
Those assets will be copied in the same folder when building so you can just use a relative path to use them.

A summary is only defined if you put `<!-- more -->` in the content. If present in a page, the summary will be from
the start up to that tag.s

### Sections
Sections represent a group of pages, for example a `tutorials` section of your site.
Sections are only created in Gutenberg when a file named `_index.md` is found in the `content` directory.

This `_index.md` file needs to include a front-matter as well, but won't have content:

```md
+++
title = "Tutorials"
description = ""
+++
```
Both `title` and `description` are mandatory, you can also set the `template` variable to change
which template will be used to render that section.

Sections will also automatically pick up their subsections, allowing you to make some complex pages layout and
table of contents.

### Code highlighting themes
Code highlighting can be turned on by setting `highlight_code = true` in `config.toml`.

When turned on, all text between backticks will be highlighted, like the example below.

    ```rust
    let site = Site::new();
    ```
If the name of the language is not given, it will default to plain-text highlighting.

Gutenberg uses Sublime Text themes for syntax highlighting. It comes with the following theme
built-in:

- base16-ocean-dark
- base16-ocean-light
- gruvbox-dark
- gruvbox-light
- inspired-github
- kronuz
- material-dark
- material-light
- monokai
- solarized-dark
- solarized-light

A gallery containing lots of themes at https://tmtheme-editor.herokuapp.com/#!/editor/theme/Agola%20Dark.
More themes can be easily added to gutenberg, just make a PR with the wanted theme.

## Example sites

- [vincent.is](https://vincent.is): https://gitlab.com/Keats/vincent.is
