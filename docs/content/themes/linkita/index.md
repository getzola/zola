
+++
title = "Linkita"
description = "A clean and elegant blog theme for Zola. Linkita is based on Kita and Hugo-Paper and is multilingual and SEO friendly."
template = "theme.html"
date = 2025-12-13T22:51:05+02:00

[taxonomies]
theme-tags = ['Blog', 'Multilingual', 'Responsive', 'SEO', 'Search']

[extra]
created = 2025-12-13T22:51:05+02:00
updated = 2025-12-13T22:51:05+02:00
repository = "https://codeberg.org/salif/linkita.git"
homepage = "https://codeberg.org/salif/linkita"
minimum_version = "0.19.0"
license = "MIT"
demo = "https://salif.github.io/linkita/"

[extra.author]
name = "Salif Mehmed"
homepage = "https://salif.eu"
+++        

# Linkita

A clean and elegant blog theme for [Zola](https://www.getzola.org/).
Linkita is based on [Kita](https://github.com/st1020/kita)
and [Hugo-Paper](https://github.com/nanxiaobei/hugo-paper) and is multilingual and SEO friendly.

- The source code is available on [Codeberg](https://codeberg.org/salif/linkita)
  and mirrored on [GitHub](https://github.com/salif/linkita).
- Open bug reports and feature requests on [Codeberg](https://codeberg.org/salif/linkita/issues).
- See the [quickstart repository](https://github.com/salif/linkita-start).
- See a [live preview](https://salif.github.io/linkita/) and
  [its source code](https://codeberg.org/salif/linkita-demo).

## Features

- Easy to use and modify
- No preset limits
- Inject support
- Dark mode
- Responsive design
- Social icons
- Taxonomies support
- Projects page
- Archive page
- Table of Content
- Admonition shortcode
- SEO friendly
- Comments using [Giscus](https://giscus.app/)
- Mathematical notations using [KaTeX](https://katex.org/)
- Diagrams and charts using [Mermaid](https://mermaid.js.org/)
- Multilingual support
- Search support (elasticlunr_javascript)
- Improved search engine optimization
- Improved configurability
- Author profiles
- Projects shortcode
- Keyboard shortcuts
- Relative URLs support

## Installation

The fastest way to create a new site is to use the
[linkita-start template](https://github.com/salif/linkita-start).
This gives you a complete blog setup with all the essential configuration ready to go.

### Manual installation

0. Create a new Zola site if you haven't already:

```sh
zola init myblog
cd myblog
git init
```

1. Add the theme as a git submodule:

```sh
git submodule add -b linkita https://codeberg.org/salif/linkita.git themes/linkita
```

If you don't want to use git submodules, you can clone the repository instead:

`git clone https://codeberg.org/salif/linkita.git themes/linkita`

2. Enable the theme in your `config.toml` file:

```toml ,name=config.toml
theme = "linkita"
```

Place it near the `base_url` variable, not under `[extra]`.

## Managing versions

To update the theme, run:

```sh
git submodule update --remote themes/linkita
```

Check the [changelog](https://codeberg.org/salif/linkita/src/branch/linkita/CHANGELOG.md)
for all versions after the one you are using.
There may be breaking changes that require manual involvement.

## Usage

Linkita uses the following front matter variables.
All variables are optional.
Set the ones you need.

### YAML frontmatter

```yaml ,name=frontmatter
---
title: ""
description: ""
date: 
updated: 
taxonomies:
  categories:
  tags:
extra:
  comment: false
  math: false
  mermaid: false
  cover:
    image: ""
    alt: ""
---
```

### TOML frontmatter

```toml ,name=frontmatter
+++
title = ""
description = ""
# The date of the post
date = 2025-12-30
# The last updated date of the post
updated = 2025-12-31

[taxonomies]
categories = []
tags = []

[extra]
# Enable comments
comment = false
# Enable KaTeX support
math = false
# Enable Mermaid support
mermaid = false
[extra.cover]
# Path to the cover image
image = ""
# A description of the cover image
alt = ""
+++
```

### Other extra front matter variables

Linkita supports [more extra variables, listed here](https://salif.github.io/linkita/extra-frontmatter/).

### Pages and Posts

#### Home page

Create a `content/_index.md` file and set `extra.profile` to your username:

```toml ,name=content/_index.md
+++
title = ""
description = ""
sort_by = "date"
paginate_by = 4
[extra]
profile = "your_username"
+++
```

Do it for each language in your site.
For French, the file name is `content/_index.fr.md`.

See Profiles for more.

#### Posts

In the `content` directory, create a subdirectory named `blog` or another name of your choice.

Create a `content/blog/_index.md` file:

```toml ,name=content/blog/_index.md
+++
title = "Archive"
description = ""
template = "archive.html"
transparent = true
[extra]
date_format = "%m-%d"
+++
```

Create a `content/blog/hello.md` file:

```md ,name=content/blog/hello.md
+++
title = "Title"
date = 2025-12-30
+++

Summary <!-- more -->

## Hello, world!
```

#### Pages

The default page template `page.html` is for blog posts.
For pages that are not blog posts you can use the `pages.html` template.

In the `content` directory, create a subdirectory named `pages` and
create a `content/pages/_index.md` file:

```toml ,name=content/pages/_index.md
+++
render = false
page_template = "pages.html"
+++
```

Create a `content/pages/about.md` file:

```md ,name=content/pages/about.md
+++
title = "About me"
description = ""
path = "about"
+++

## Hello, world!
```

If you want, you can also create [a page for your projects](https://salif.github.io/linkita/shortcodes/#projects).

### Setting page authors

Choose one of the following options or skip if you don't know what you're doing:

#### Option A: Using `page.authors` and `config.author`

The default author for posts is set using the `author` variable in the `config.toml` file.

You don't need to set `authors` in the front matter if the default author is the only author of the post.
Otherwise, set `authors`:

```toml ,name=frontmatter
+++
authors = ["author_username"]
+++
```

#### Option B: Using Taxonomies

Useful if the blog has a team of several authors.
If you choose this option you should set taxonomies in each post.

```toml ,name=frontmatter
+++
[taxonomies]
authors = ["author_username", "author2_username"]
+++
```

### Inject support

You can easily use inject to add new features to your side without modifying the theme itself.

To use inject, you need to add some HTML files to the `templates/injects` directory.

The available inject points are: `head.html`, `head_end.html`, `header_nav.html`,
`body_start.html`, `body_end.html`, `page_start.html`, `page_end.html`, `footer.html`.

For example, you can add JavaScript files and CSS stylesheets in the `templates/injects/head_end.html` file.

### Keyboard shortcuts

| Action            | Shortcut                     |
| ----------------- | ---------------------------- |
| Home              | <kbd>Alt</kbd>+<kbd>\!</kbd> |
| Search            | <kbd>Alt</kbd>+<kbd>\/</kbd> |
| Toggle menu       | <kbd>Alt</kbd>+<kbd>\+</kbd> |
| Toggle dark mode  | <kbd>Alt</kbd>+<kbd>\$</kbd> |
| Go to prev page   | <kbd>Alt</kbd>+<kbd>\,</kbd> |
| Go to next page   | <kbd>Alt</kbd>+<kbd>\.</kbd> |
| Table of contents | <kbd>Alt</kbd>+<kbd>\=</kbd> |
| Skip to footer    | <kbd>Alt</kbd>+<kbd>\_</kbd> |

## Configuration

Copy and paste the examples into your `config.toml` file
and comment out the variables you don't use instead of setting empty values.
All variables are optional.

```toml ,name=config.toml
# The URL the site will be built for.
base_url = "https://example.com"

# The site theme to use.
theme = "linkita"

# The default language.
# "en" is for English.
default_language = "en"

# The default author for pages.
# See "extra.profiles".
author = "your_username"

# The site title.
# Will be in all page titles.
title = ""

# The site description.
# Used in feeds by default.
description = ""

# Automatically generate a feed.
# Default value: false
generate_feeds = true

# The filenames to use for the feeds.
# e.g. ["rss.xml"]
feed_filenames = ["atom.xml"]

# Build a search index from the pages and section content
# for "default_language".
# build_search_index = true
```

Zola has built-in support for taxonomies.
Linkita has special support for taxonomies named `tags`, `categories`, and `authors`.

```toml ,name=config.toml
[[taxonomies]]
name = "categories"
feed = true
paginate_by = 4

[[taxonomies]]
name = "tags"
feed = true
paginate_by = 4

[[taxonomies]]
name = "authors"
feed = true
paginate_by = 4
```

You can add more languages by replacing `fr` from the following example with the language code:

```toml ,name=config.toml
[languages.fr]
title = "Site title in French"
description = "Site description in French"
generate_feeds = true
feed_filenames = ["atom.xml"] # or ["rss.xml"]
build_search_index = true
taxonomies = [
    { name = "tags", feed = true, paginate_by = 4 }
]
```

By default, taxonomy pages are sorted in descending order by the number of posts.
You can customize this behavior using the `extra.taxonomy_sorting` variable:

```toml ,name=config.toml
[extra.taxonomy_sorting]
# "ascending_frequency", "descending_frequency" or "name"
# Default value: "descending_frequency"
categories = "name"
tags = "descending_frequency"
authors = "name"
```

### General config

```toml ,name=config.toml
[extra]
# Enable KaTeX math formula support globally.
# Default value: false
math = false

# Enable Mermaid support globally.
# Default value: false
mermaid = false

# Enable comments globally.
# Default value: false
comment = false

# Title separator.
title_separator = " | "

# The top menu.
# See "extra.menus".
header_menu_name = "menu_name"

# See "extra.menus".
menu_i18n = false

# If you disable default favicons, you can use
# the inject support to set your own favicons.
# Default value: false
disable_default_favicon = false

# If you want to implement the JS code
# yourself, set to true and use the inject support.
# Default value: false
disable_javascript = false

# Use a CDN for libraries like KaTeX.
# Default value: false
use_cdn = false

# Use relative urls.
# It doesn't apply for content yet.
# Default value: false
relative_urls = false

# If you want to view the site without a webserver
# set this and "relative_urls" to true.
# Default value: false
ugly_urls = false

# Prioritize summary over description.
# Default value: false
page_summary_on_paginator = false

# Reverse the order of prev and next post links.
# Default value: false
invert_page_navigation = false

# Header buttons.
# Valid values:
#  "site_title", "home_button", "theme_select", "theme_button",
#  "search_button", "translations_select", "translations_button".
# Can be set to an empty array.
# header_buttons = ["site_title", "theme_select", "search_button", "translations_select"]

# What info is shown on post pages.
# Valid "when" values:
#  "date", "date_updated", "reading_time", "word_count", "authors", "tags", "".
# The "prepend" and "append" are used when the value of "when" is defined for the page.
# e.g. [{when="", prepend="Page Info: "},{when="date",prepend="Published on "},{when="authors",prepend="By "}]
# page_info = [{ when="date" }, { when="date_updated", prepend="(", append=")" }, { when="reading_time" }]

# What info is shown for posts on paginators.
# page_info_on_paginator = [{ when="date" }, { when="reading_time" }]

# Enable table of contents on all pages.
# If not set, toc is enabled only on posts.
# If set to false, toc is disabled on all pages.
# Type: boolean or object
# toc = true
```

### Style config

```toml ,name=config.toml
[extra.style]
# The custom background color.
bg_color = "#f4f4f5"
# The custom background color in dark mode.
bg_dark_color = "#18181b"

# Enable header blur.
header_blur = false

# The custom header color, only available
# when "header_blur" is false.
header_color = "#e4e4e7"
# The custom header color in dark mode, only available
# when "header_blur" is false.
header_dark_color = "#27272a"
```

### Menus

```toml ,name=config.toml
[extra.menus]
menu_name = [
  { url = "$BASE_URL/blog/", name = "Archive" },
  { url = "$BASE_URL/categories/", name = "Categories" },
  { url = "$BASE_URL/tags/", name = "Tags" },
  { url = "$BASE_URL/about/", name = "About" },
]

# Example multilingual menu using `names`.
multilingual_menu_name = [
  { url = "$BASE_URL/about/", names = { en = "About", fr = "About in French" } },
  { url = "$BASE_URL/projects/", names = { en = "Projects", fr = "Projects in French" } },
  { url = "$BASE_URL/blog/", names = { en = "Archive", fr = "Archive in French" } },
  { url = "$BASE_URL/categories/", names = { en = "Categories", fr = "Categories in French" } },
  { url = "$BASE_URL/tags/", names = { en = "Tags", fr = "Tags in French" } },
  { url = "$BASE_URL/authors/", names = { en = "Authors", fr = "Authors in French" } },
]
```

To use a menu, set `extra.header_menu_name`.

`$BASE_URL` in `url` will be automatically replaced with the language specific base url.
You can use [Internal links](https://www.getzola.org/documentation/content/linking/#internal-links)
instead of `$BASE_URL`.

If your site is multilingual, you can choose one of the following options or combine them:
1. Set `names` as in the `multilingual_menu_name` example.
2. Set `extra.menu_i18n` to `true` in order to use the `static/i18n/menu.json` file.
3. Set `header_menu_name` for each language using the language specific options.

### Profiles

```toml ,name=config.toml
# Replace "your_username" with your username.
[extra.profiles.your_username]

# The URL of avatar.
# e.g. "icons/github.svg"
avatar_url = ""

# A description of what is in the avatar.
avatar_alt = ""

# Invert avatar color in dark mode.
# Default value: false
avatar_invert = false

# Profile name.
# Default value: the username
name = ""

# Profile bio.
# Supports Markdown.
bio = ""

# Social icons.
# "name" should be the file name of "static/icons/*.svg" or
# the icon name of https://simpleicons.org/
# "url" supports "$BASE_URL".
# Other variables: "urls", "title", "titles".
social = [
    { name = "bluesky", url = "https://bsky.app/profile/username" },
    { name = "github", url = "https://github.com/username" },
    { name = "email", url = "mailto:example@example.com" },
    { name = "rss", url = "$BASE_URL/atom.xml" },
]
```

### Profile translations

Skip if your site is not multilingual.

```toml ,name=config.toml
# For French. Replace "your_username" with your username.
[extra.profiles.your_username.languages.fr]

# A description of what is in the avatar.
avatar_alt = ""

# Profile name.
name = ""

# Profile bio.
# Supports Markdown.
bio = ""

# Social icons.
social = []
```

### Open Graph for profiles

See [the Open Graph protocol](https://ogp.me/).

```toml ,name=config.toml
# Replace "your_username" with your username.
[extra.profiles.your_username.open_graph]

# The URL of social image.
image = ""

# A description of what is in the social image.
# Default value: ""
image_alt = ""

# Your first name. No default value.
first_name = ""
# Your last name. No default value.
last_name = ""
# Your username. No default value.
username = ""
# e.g. "female" or "male". No default value.
gender = ""

# fb_app_id = "Your fb app ID"
# fb_admins = ["YOUR_USER_ID"]

# Set if you have a Fediverse account.
#  handle – Your Fediverse handle.
#  domain – Your Fediverse instance.
#  url – Your Fediverse account URL. Optional.
# Example for @me@mastodon.social:
# { handle = "me", domain = "mastodon.social" }
fediverse_creator = { handle = "", domain = "" }
```

`image` and `image_alt` of the default author's profile will be used
as a fallback open graph image for all pages.

#### Open Graph translations

Skip if your site is not multilingual.

```toml ,name=config.toml
# For French. Replace "your_username" with your username.
[extra.profiles.your_username.open_graph.languages.fr]
# A description of what is in the social image.
image_alt = ""
```

### The page footer

```toml ,name=config.toml
[extra.footer]
# Replace with the correct year.
# Default value: the current year
since = 2025

# Replace with the URL of the license you want.
# No default value. Supports "$BASE_URL".
license_url = "https://creativecommons.org/licenses/by-sa/4.0/deed"

# Replace "Your Name" with your name and "CC BY-SA 4.0" with the name of the license you want.
copyright = "&copy; $YEAR Your Name &vert; [CC BY-SA 4.0]($LICENSE_URL)"

# Not used yet.
# Supports "$BASE_URL".
# privacy_policy_url = "$BASE_URL/privacy-policy/"

# Not used yet.
# Supports "$BASE_URL".
# terms_of_service_url = "$BASE_URL/terms-of-service/"

# Not used yet.
# Supports "$BASE_URL".
# search_page_url = "$BASE_URL/search/"
```

The `copyright` variable supports Markdown,
`$BASE_URL`, `$YEAR` (uses `since`), and `$LICENSE_URL` (uses `license_url`).

### Language specific options

For date format, see [chrono docs](https://docs.rs/chrono/0.4/chrono/format/strftime/index.html).

```toml ,name=config.toml
# For English
[extra.languages.en]

# No default value.
locale = "en_US"

# Default value: "%F"
date_format = "%x"

# Default value: extra.page_info
# page_info = []

# Default value: extra.page_info_on_paginator
# page_info_on_paginator = []

# Default value: extra.header_menu_name
# header_menu_name = "menu_name"

# Default value: extra.header_buttons
# header_buttons = []

# To set a different lang attribute of the document.
# Also changes the interface language. e.g. "en-GB".
# hreflang = ""

# To format numbers for a different locale. e.g. "en-GB".
# Useful if lang is not a valid locale.
# num_format = ""

# Set a description for taxonomy pages.
[extra.languages.en.taxonomy_descriptions]
categories = "A map of all categories on this site. Start exploring!"
tags = "A map of all tags on this site. Start exploring!"
authors = "A map of all authors on this site. Start exploring!"

# Set a description for term pages.
# "$NAME" will be automatically replaced.
[extra.languages.en.term_descriptions]
categories = "Browse articles related to $NAME. Start exploring!"
tags = "Browse articles related to $NAME. Start exploring!"
authors = "Browse articles written by $NAME. Start exploring!"
```

```toml ,name=config.toml
# For French
[extra.languages.fr]
locale = "fr_FR"
date_format = "%x"
```

### Web analytics

#### GoatCounter

Set only if you use [GoatCounter](https://www.goatcounter.com/).

```toml ,name=config.toml
[extra.goatcounter]
# No default value.
endpoint = "https://MYCODE.goatcounter.com/count"

# To enable tracking pixel, set to an empty string.
# If your "base_url" includes a subpath, set to
# the subpath without a trailing slash.
# noscript_prefix = ""
```

#### Vercel Analytics

Set only if you use [Vercel Web Analytics](https://vercel.com/docs/analytics).

```toml ,name=config.toml
[extra.vercel_analytics]
# No default value.
src = "/_vercel/insights/script.js"
```

#### Prevent tracking own pageviews

Open a page of your site, adding `#disable-analytics` to the page address.
Do this once for each browser and device.
For example, open <http://127.0.0.1:1111/#disable-analytics>.

### Comments

See [giscus.app](https://giscus.app/).
Only available when `extra.comment` in the front matter or `extra.comment` in the config is set to `true`.

```toml ,name=config.toml
[extra.giscus]
# No default value.
repo = ""
# No default value.
repo_id = ""
# No default value.
category = ""
# No default value.
category_id = ""
# Default value: "pathname"
mapping = "pathname"
# Default value: 1
strict = 1
# Default value: 0
reactions_enabled = 0
# Default value: 0
emit_metadata = 0
# Default value: "top"
input_position = "top"
# Default value: "preferred_color_scheme"
theme = "preferred_color_scheme"
# Default value: "en"
lang = "en"
# Default value: "lazy"
loading = "lazy"
```

## Contributing

This project is under the [MIT License](https://codeberg.org/salif/linkita/src/branch/linkita/LICENSE).

Pull requests are welcome on [Codeberg](https://codeberg.org/salif/linkita) and [Github](https://github.com/salif/linkita).

### Localization

Feel free to contribute new translations or improve existing ones.

To add a new language, create a file `LANG_CODE.json` in the `static/i18n` directory.
See [ISO 639-1](https://localizely.com/iso-639-1-list/) codes.

```sh
echo '{}' > static/i18n/LANG_CODE.json
node ./static/i18n/sync.js
```

See also the `static/i18n/menu.json` file and
the [demo repository](https://codeberg.org/salif/linkita-demo).

Live preview is available in following languages:

[Arabic](https://salif.github.io/linkita/ar/),
[Bulgarian](https://salif.github.io/linkita/bg/),
[Czech](https://salif.github.io/linkita/cs/),
[Esperanto](https://salif.github.io/linkita/eo/),
[Spanish](https://salif.github.io/linkita/es/),
[Finnish](https://salif.github.io/linkita/fi/),
[French](https://salif.github.io/linkita/fr/),
[Globasa](https://salif.github.io/linkita/gb/),
[Japanese](https://salif.github.io/linkita/ja/),
[Korean](https://salif.github.io/linkita/ko/),
[English](https://salif.github.io/linkita/),
[Turkish](https://salif.github.io/linkita/tr/),
[Chinese](https://salif.github.io/linkita/zh/).

## Sites using Linkita

- [Zola Themes Collection](https://github.com/salif/zola-themes-collection)
- [Salif's site](https://codeberg.org/salif/personal-web-page)
- [Rratic's blog](https://github.com/Rratic/rratic.github.io)

If your blog is using Linkita and is open source, feel free to create a pull request to add it to this list.

## Other Zola themes

Linkita is one of several Zola themes I maintain.
Other themes include:

- [Tukan](https://codeberg.org/salif/tukan) – Fork of zola-blog-theme, inspired from Toucan theme.
- [Trankwilo](https://codeberg.org/salif/trankwilo) – Fork of the Goyo theme, for documentation.

        