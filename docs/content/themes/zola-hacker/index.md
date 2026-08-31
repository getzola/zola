
+++
title = "zola-hacker"
description = "Hacker is a theme for Zola"
template = "theme.html"
date = 2026-08-29T11:45:50-04:00

[taxonomies]
theme-tags = []

[extra]
created = 2026-08-29T11:45:50-04:00
updated = 2026-08-29T11:45:50-04:00
repository = "https://github.com/en9inerd/zola-hacker"
homepage = "https://github.com/en9inerd/zola-hacker"
minimum_version = "0.19.1"
license = "MIT"
demo = "https://zola-hacker.enginerd.io/"

[extra.author]
name = "Vladimir Loskutov"
homepage = "https://github.com/en9inerd"
+++        

# Hacker theme for Zola

Zola Hacker is a minimalistic theme for Zola, inspired by the [Hacker theme](https://pages-themes.github.io/hacker/) for Jekyll. It is designed for developers who want to write blogs.

## Demo

[Live Preview](https://zola-hacker.enginerd.io/).

## Requirements

Before using the theme, you need to install the [Zola](https://www.getzola.org/documentation/getting-started/installation/) ≥ 0.23.4.

## Quick Start

```bash
git clone git@github.com:en9inerd/zola-hacker.git
cd zola-hacker
zola serve
# open http://127.0.0.1:1111/ in the browser
```

## Installation

Just earlier we showed you how to run the theme directly. Now we start to
install the theme in an existing site step by step.

### Step 1: Create a new zola site

```bash
zola init mysite
```

### Step 2: Install Zola Hacker Theme

Download this theme to your themes directory:

```bash
cd mysite/themes
git clone git@github.com:en9inerd/zola-hacker.git
```

Or install as a submodule:

```bash
cd mysite
git init  # if your project is a git repository already, ignore this command
git submodule add git@github.com:en9inerd/zola-hacker.git themes/hacker
```

### Step 3: Configuration

Enable the theme in your `zola.toml` in the site directory:

```toml
theme = "hacker"
```

Or copy the `zola.toml` from the theme directory to your project's
root directory:

```bash
cp themes/hacker/zola.toml zola.toml
```

### Step 4: Add new content

You can copy the content from the theme directory to your project:

```bash
cp -r themes/hacker/content .
```

You can modify or add new posts in the `content/posts`, `content/pages` or other
content directories as needed.

### Step 5: Run the project

Just run `zola serve` in the root path of the project:

```bash
zola serve
```

This command will start the Zola development web server accessible by default at
`http://127.0.0.1:1111`. Saved changes will live reload in the browser.

## Customization

You can customize your configurations, templates and content for yourself. Look
at the `zola.toml`, `theme.toml`, `content` files and templates files in this
repo for an idea.

### Global Configuration

There are some configuration options that you can customize in `zola.toml`.

#### Configuration options before `extra` options

Set the tags taxonomy in the `zola.toml`:

```toml
taxonomies = [
    { name = "tags" },
]
```

#### Configuration options under the `extra`

The following options should be under the `[extra]` in `zola.toml`

- `language_code` - set HTML file language (default to `en-US`)
- `title_separator` - the separator to your site title, like `|` and `-` (defaults to `|`)
- `logo` - path to the logo image
- `google_analytics` - Google Analytics tracking ID
- `timeformat` - the timeformat for the blog article published date
- `timezone` - the timezone for the blog article published date
- `edit_page` - whether to show the edit page in the github repo for your posts
- `menu` - set the menu items for your site
- `contact_form_script_id` - the script id for the contact form based on [Google Script](https://github.com/en9inerd/learn-to-send-email-via-google-script-html-no-server)
- `[extra.github]` - set the GitHub metadata for your site
- `[extra.giscus]` - set the Giscus settings for your site to enable the comments. Please use own settings from the [Giscus](https://giscus.app/) website
- `[extra.opengraph]` - set the Open Graph metadata for your site
- `[extra.pgp_key]` - set pgp key in the footer for certain pages
- `social_links` - set the social media links in the footer
...

### Templates

All pages extend the `base.html`, and you can customize them as need.

### Components

The theme provides some components to help you write your content. Zola 0.23
replaced shortcodes with [Tera components](https://www.getzola.org/documentation/templates/overview/),
so they are called with the `{{/*<name ... />*/}}` syntax instead of `name()`.

Components are hygienic: they only see the arguments you pass, so `config` (and
`page`, where the component reads front matter) must be passed explicitly. Values
that are not strings are wrapped in braces, for example `size={100}`.

`contact_form`
The `contact_form` component is based on [Google Apps Mail](https://github.com/en9inerd/learn-to-send-email-via-google-script-html-no-server) to send emails without a server.
It depends on `contact_form_script_id` in the `zola.toml`.

```markdown
{{/*<contact_form config />*/}}
```

`cv`
The `cv` component is used to display the CV in the page. Data for CV is stored in yaml format in the `data/cv` directory.

```markdown
{{/*<cv config />*/}}
```

`github_avatar`
The `github_avatar` component is used to display the GitHub avatar image. It depends on `extra.github.username` in the `zola.toml`. Also, you can pass size as an argument.

```markdown
{{/*<github_avatar config size={100} />*/}}
```

`projects`
The `projects` component is used to display repositories from GitHub. It depends on `extra.github.username` in the `zola.toml` and `extra.repo_names` in page front matter to filter the repositories.

```markdown
{{/*<projects page config />*/}}
```

### Migrating from a pre-0.23 version of this theme

Zola 0.23 removed shortcodes and macros, so upgrading is a breaking change:

- Rewrite shortcode calls in your content using the component syntax above.
- Remove `{%/* import "macros.html" as macros */%}`; components are registered
  globally and no longer need importing. Calls such as `macros::seo(...)` become
  `{{/*<seo ... />*/}}`.
- If you override `base.html`, note that the `lang` variable was renamed to
  `html_lang` — it shadowed Zola's own `lang` and broke `get_section`.

## Reporting Issues

We use GitHub Issues as the official bug tracker for the **Zola Hacker Theme**. Please
search [existing issues](https://github.com/en9inerd/zola-hacker/issues). It’s
possible someone has already reported the same problem.

If your problem or idea is not addressed yet, [open a new issue](https://github.com/en9inerd/zola-hacker/issues/new).

## License

**Zola Hacker Theme** is distributed under the terms of the
[MIT license](https://github.com/en9inerd/zola-hacker/blob/master/LICENSE).

        