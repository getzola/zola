
+++
title = "radion"
description = "A sleek, modern blog theme."
template = "theme.html"
date = 2025-12-18T08:48:59-08:00

[taxonomies]
theme-tags = ['SEO', 'search', 'accessible']

[extra]
created = 2025-12-18T08:48:59-08:00
updated = 2025-12-18T08:48:59-08:00
repository = "https://github.com/micahkepe/radion.git"
homepage = "https://github.com/micahkepe/radion"
minimum_version = "0.20.0"
license = "MIT"
demo = "https://micahkepe.com/radion/"

[extra.author]
name = "Micah Kepe"
homepage = "https://micahkepe.com"
+++        

# radion

A sleek, modern blog theme for [Zola](https://www.getzola.org/). See the live
site demo [here](https://micahkepe.com/radion/).

> **radion**
> noun
>
> 1. (_physics_) A scalar field in higher-dimensional spacetimes

<details open>
<summary>Dark theme</summary>

![radion dark theme screenshot](screenshot.png)

</details>

<details>
<summary>Light theme</summary>

![radion light theme screenshot](screenshot-light.png)

</details>

## Features

- [x] Code Snippet Clipboards
  - [x] Line(s)-specific highlighting
- [x] Latex Support
- [x] Light/Dark mode support
- [x] Search functionality
- [x] Table of Contents option
- [x] Footnote support
- [x] Built-in comments option (Giscus)
- [x] Open Graph cover image selection

## Contents and Configuration Guide

- Installation
- Options
  - Top\-menu
  - Title
  - Author Attribution
    - Defining a Global Default Author in config\.toml
    - Defining Author(s) Per\-Page
  - Favicon
  - GitHub
  - Fediverse and Mastodon
  - Code Snippets
    - Syntax Highlighting:
    - Enhanced Codeblocks (Clipboard Support and Language Tags)
  - LaTex Support
  - Searchbar
  - Light and Dark Modes
  - Table of Contents
  - Comments
  - Post Revision History
  - Set Post Open Graph Image (Cover Image)
  - Custom Fonts
    - Font Weights by Provider
    - Examples
- Acknowledgements

## Installation

First download this theme to your `themes` directory:

```bash
cd themes
git clone https://github.com/micahkepe/radion
```

and then enable it in your `config.toml`:

```toml
theme = "radion"
```

This theme requires your index section (`content/_index.md`) to be paginated to
work:

```toml
paginate_by = 5
```

The posts should therefore be in directly under the `content` folder.

The theme requires tags and categories taxonomies to be enabled in your
`config.toml`:

```toml
taxonomies = [
    # You can enable/disable RSS
    { name = "categories", feed = true },
    { name = "tags", feed = true },
]
```

If you want to paginate taxonomies pages, you will need to overwrite the
templates as it only works for non-paginated taxonomies by default.

---

## Options

### Top-menu

Set a field in `extra` with a key of `radion_menu`:

```toml
radion_menu = [
    { url = "$BASE_URL", name = "Home" },
    { url = "$BASE_URL/categories", name = "Categories" },
    { url = "$BASE_URL/tags", name = "Tags" },
    { url = "https://google.com", name = "Google" },
]
```

If you put `$BASE_URL` in a url, it will automatically be replaced by the actual
site URL.

### Title

The site title is shown on the homepage. As it might be different from the
`<title>` element that the `title` field in the config represents, you can set
the `radion_title` instead.

### Author Attribution

You may define the author(s) of a page in either the root `config.toml` file, or
on a per-page basis in the page's frontmatter.

The order of precedence for determining the author shown in a page’s footer is:

1. `page.extra.author` (highest precedence)
2. `page.authors`
3. `config.author` (lowest precedence, default)

#### Defining a Global Default Author in `config.toml`

In `config.toml`:

```toml
author = "John Smith"
```

#### Defining Author(s) Per-Page

At the top of a page in its frontmatter (wrap this in `+++`):

1. Define a single author for the page:

```toml
title = "..."
date = 1970-01-01

[extra]
author = "John Smith"
```

Alternatively, you can define the `page.authors` variable with a single entry:

```toml
title = "..."
date = 1970-01-01
authors = ["John Smith"]
```

2. Define multiple authors for a page:

```toml
title = "..."
date = 1970-01-01
authors = ["John Smith", "Joe Schmoe", "Jane Doe"]
```

> [!NOTE]
> Do not define both `extra.author` and `authors` in the same page unless you
> want `extra.author` to take precedence.

### Favicon

To change the default favicon:

1. Create your own favicon folder with the following site:
   [RealFaviconGenerator](https://realfavicongenerator.net/)
   - Set the 'Favicon path' option to `/icons/favicon/`

2. Unzip the created folder
3. Create a `static/icons/` directory if it does not already exist
4. Place the unzipped `favicon/` directory in `static/icons/`.

By default, favicons are enabled, however, if for some reason you would like to
disable favicons, set the following in your `config.toml`:

```toml
[extra]
favicon = false
```

### GitHub

To enable a GitHub reference link in the header, set the following in your
`config.toml`:

```toml
[extra]
github = "https://github.com/your-github-link"
```

### Fediverse and Mastodon

In your `config.toml` you can set options related to the Fediverse and
explicitly Mastodon.

To enable author attribution, set the `extra.fediverse.creator` option to your
account address. To enable website verification, set the
`extra.fediverse.rel_me` option to a link to your profile.

Set the `extra.mastodon` field to a link to your Mastodon account to show a
Mastodon logo with this link.

```toml
[extra]
fediverse.creator = "@username@my.instance.example.com"
fediverse.rel_me = "https://my.instance.example.com/@username"
mastodon = "https://my.instance.example.com/@username"
```

### Code Snippets

#### Syntax Highlighting:

This theme uses **class-based syntax highlighting** for better security (CSP
compliance) and theme flexibility.

In your `config.toml`:

```toml
[markdown]
highlight_code = true
highlight_theme = "css"  # Required for class-based highlighting

# Specify theme(s) for dark and light modes
highlight_themes_css = [
  { theme = "one-dark", filename = "syntax/syntax-theme-dark.css" },
  { theme = "gruvbox-dark", filename = "syntax/syntax-theme-light.css" },
]
```

For example, the above configuration will use the `one-dark` theme for dark mode
and the `gruvbox-dark` theme for light mode.

##### Choosing Themes

1. Browse available themes at [Zola's syntax highlighting
   docs](https://www.getzola.org/documentation/getting-started/configuration/#syntax-highlighting)
2. Update both entries in `highlight_themes_css` with your preferred themes
3. Run `zola serve` or `zola build` - the CSS files will be automatically
   generated in `static/`

> [!NOTE]
> If you change the syntax themes, delete the `static/syntax` directory to
> ensure that the new Syntect CSS files are properly updated.

##### Migration from Previous Versions

**Breaking Change:** If upgrading from an older version that used inline styles:

1. Change `highlight_theme` from a specific theme name to `"css"`
2. Add the `highlight_themes_css` configuration as shown above
3. Delete any old `syntax-theme-*.css` files from your `static/` folder
4. Run `zola build` to regenerate the CSS files

This change improves security by removing inline styles and enables proper CSP
headers.

#### Enhanced Codeblocks (Clipboard Support and Language Tags)

```toml
[extra]
codeblock = true
```

> [!NOTE]
> Ligatures are disabled by default as defined in the
> [\_theme.scss](./sass/_theme.scss) file.

### LaTex Support

To enable LaTeX support with MathJax, set the following in your `config.toml`:

```toml
[extra]
latex = true
```

### Searchbar

To enable a searchbar at the top of the page navigation, set the following in
your `config.toml`:

```toml
build_search_index = true

[search]
index_format = "elasticlunr_json"

[extra]
enable_search = true
```

### Light and Dark Modes

To set the color theme of the site, set the following in your `config.toml`:

```toml
[extra]
theme = "toggle" # options: {light, dark, auto, toggle}
```

There are four options for the `theme` field:

- `light`: Always light mode
- `dark`: Always dark mode
- `auto`: Automatically switch between light and dark mode based on the user's
  system preferences
- `toggle`: Allow the user to toggle between light and dark mode

### Table of Contents

To enable a table of contents on a page, add the following to the front matter
of the page:

```toml
[extra]
toc = true
```

### Comments

> [!NOTE]
> Giscus comments assumes that you are hosting the blog site via GitHub Pages
> and thus have access to GitHub Discussions.

First, follow the instructions at [giscus.app](https://giscus.app/).
This includes installing the Giscus app and enabling discussions on the
GitHub repository that you host the website code. Additionally, fill in the
repository path in the prompt. Then, from the generated script, fill in the
corresponding values in the `config.toml`:

```toml
[extra]
comments = true  # {true, false}; sets global enabling of comments by default
giscus_repo = "FILL ME IN"
giscus_repo_id = "FILL ME IN"
giscus_data_category_id = "FILL ME IN"
# giscus_data_category = "General" # Default to "General"
```

Comments can be enabled or disabled on a per page basis by editing the page's
front matter. For example, to disable comments on a specific post:

```toml
[extra]
comments = false
```

The `config.toml` value for `comments` takes precedence and priority. For
example, if you globally disable comments in your `config.toml` by setting
`comments = false`, then trying to enabling comments through a page's front
matter will have no effect.

### Post Revision History

To enable revision history links that allow readers to view the commit history
for individual posts, configure the following in your `config.toml`:

```toml
[extra]
# Enable revision history globally
revision_history = true
# Your blog's GitHub repository URL
blog_github_repo_url = "https://github.com/username/repository-name"
```

Revision history can be enabled or disabled on a per-page basis by adding the
following to a page's front matter:

```toml
[extra]
revision_history = true  # or false to disable for this page
```

When enabled, a "(revision history)" link will appear in the page footer that
links directly to the GitHub commit history for that specific content file,
allowing readers to see how the post has evolved over time.

### Set Post Open Graph Image (Cover Image)

[Open Graph](https://ogp.me/) is a standard for embedding rich previews of
content on the Internet. It is used by social media platforms like Facebook,
Twitter, and LinkedIn to display a preview of a page when a user shares the
page on their social media network.

For example, to set the Open Graph image for a post `my-post` to be the page
asset `cover.png`, add the following to the front matter of the post:

1. Make sure the image is located in the page's content directory (i.e.
   `content/my-post/`. For example:

   ```
   content/
   └── my-post/
       ├── index.md
       ├── cover.png        # Your cover image
       └── assets/
           └── other-image.jpg
   ```

   or

   ```
   content/
   └── my-post/
       ├── index.md
       └── assets/
           ├── other-image.jpg
           └── cover.png    # Your cover image
   ```

2. Add the following to the front matter of the post:

```toml
[extra]
cover_image = "cover.png"
```

> [!NOTE]
> The image must be located within the page's content directory and
> `cover_image` expects just the filename of the image (e.g., `"cover.png"`, not
> a path like `"assets/cover.png"`). The first filename match will be used.

### Custom Fonts

Currently three font CDN sites are supported:

1. [Google Font (`"googlefont"`)](https://fonts.google.com/): Fonts from `fonts.google.com`
2. [Fontsource (`"fontsource"`)](https://fontsource.org/): Self-hosted fonts from `fontsource.org`. Uses WOFF2 files.
3. [ZeoSeven Font (`"zeoseven"`)](https://fonts.zeoseven.com/): Fonts from
   `fonts.zeoseven.com`. Requires a `font_id` for URL construction.

To configure, add entries under `[extra]` in your `config.toml`:

| Option          | Type   | Default            | Description                                                                |
| --------------- | ------ | ------------------ | -------------------------------------------------------------------------- |
| `font_cdn`      | String | `"googlefont"`     | Font provider: `"googlefont"`, `"fontsource"`, `"zeoseven"`, or `"custom"` |
| `font_name`     | String | `"JetBrains Mono"` | Font family name (e.g., `"Inter"`, `"Roboto"`)                             |
| `font_weights`  | Array  | (_See below_)      | Weights to load (provider-specific format)                                 |
| `font_display`  | String | `"swap"`           | CSS `font-display` value: `"swap"`, `"block"`, `"auto"`, etc.              |
| `font_id`       | Number | _None_             | **ZeoSeven only**: Numeric ID from font URL                                |
| `font_css_urls` | Array  | _None_             | **Custom only**: Array of CSS URLs for font definitions                    |

#### Font Weights by Provider

| Provider     | Format           | Example      |
| ------------ | ---------------- | ------------ |
| Google Fonts | Array of numbers | `[400, 700]` |
| Fontsource   | Array of strings | `["main"]`   |
| ZeoSeven     | Array of numbers | `[400, 700]` |

#### Examples

```toml
# Google Fonts
[extra]
font_cdn = "googlefont"
font_name = "Inter"
font_weights = [300, 400, 500, 700]
font_display = "swap"

# Fontsource
[extra]
font_cdn = "fontsource"
font_name = "JetBrains Mono"
font_weights = ["main"]

# ZeoSeven
[extra]
font_cdn = "zeoseven"
font_name = "Custom Font"
font_id = 443
font_weights = [400, 700]

# Custom CSS
[extra]
font_cdn = "custom"
font_name = "My Custom Font"
font_css_urls = [
    "https://example.com/fonts/custom-font.css",
    "https://cdn.example.com/typography.css"
]
```

---

## Acknowledgements

Lots of inspiration and code snippets taken from these awesome Zola themes:

- [`after-dark`](https://github.com/getzola/after-dark) by
  [Vincent Prouillet](https://www.vincentprouillet.com/)

- [`apollo`](https://github.com/not-matthias/apollo/tree/main) by
  [not-matthias](https://github.com/not-matthias)

- [`redux`](https://github.com/SeniorMars/redux) by
  [SeniorMars](https://github.com/SeniorMars).

        