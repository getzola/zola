
+++
title = "Linkita"
description = "A clean and elegant blog theme for Zola. Linkita is based on Kita and Hugo-Paper and is multilingual and SEO friendly."
template = "theme.html"
date = 2025-04-05T22:42:22+03:00

[taxonomies]
theme-tags = ['Blog', 'Multilingual', 'Responsive', 'SEO', 'Search']

[extra]
created = 2025-04-05T22:42:22+03:00
updated = 2025-04-05T22:42:22+03:00
repository = "https://codeberg.org/salif/linkita.git"
homepage = "https://codeberg.org/salif/linkita"
minimum_version = "0.19.0"
license = "MIT"
demo = "https://salif.github.io/linkita/en/"

[extra.author]
name = "Salif Mehmed"
homepage = "https://salif.eu"
+++        

# Linkita

A clean and elegant blog theme for [Zola](https://www.getzola.org/). Linkita is based on [Kita](https://github.com/st1020/kita) and [Hugo-Paper](https://github.com/nanxiaobei/hugo-paper) and is multilingual and SEO friendly.

- The source code is available on [Codeberg](https://codeberg.org/salif/linkita) and mirrored on [GitHub](https://github.com/salif/linkita).
- For discussion, join the [Matrix chat room](https://matrix.to/#/#linkita:mozilla.org).
- Open bug reports and feature requests on [Codeberg](https://codeberg.org/salif/linkita/issues).
- See [demo source code](https://codeberg.org/salif/linkita-demo) and screenshots for [light mode](https://codeberg.org/salif/linkita/src/branch/linkita/screenshot.png) and [dark mode](https://codeberg.org/salif/linkita/src/branch/linkita/screenshot.dark.png).
- Live preview in:
  - [English](https://salif.github.io/linkita/en/), [Bulgarian](https://salif.github.io/linkita/), [Esperanto](https://salif.github.io/linkita/eo/).
  - [Chinese](https://salif.github.io/linkita/zh/), [Arabic](https://salif.github.io/linkita/ar/), [Turkish](https://salif.github.io/linkita/tr/), [Globasa](https://salif.github.io/linkita/gb/).

## Features

### Kita features

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

### Linkita features

- Multilingual support
- Search support (elasticlunr_javascript)
- Improved search engine optimization
- Improved configurability
- Author profiles
- Projects shortcode
- Keyboard shortcuts

## Installing

1. Add this theme as a submodule.

```sh
git submodule add https://codeberg.org/salif/linkita.git themes/linkita
```

Alternatively, clone the repository: `git clone https://codeberg.org/salif/linkita.git themes/linkita`

2. Set `linkita` as your theme in your `config.toml` file.

```toml ,name=config.toml
theme = "linkita"
```

## Managing versions

To update the theme, run:

```sh
git submodule update --remote themes/linkita
```

Check the [changelog](https://codeberg.org/salif/linkita/src/branch/linkita/CHANGELOG.md)
for all versions after the one you are using; there may be breaking changes that require manual involvement.

## Usage

All variables are optional.
Set the ones you need.

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

### Other extra frontmatter variables

```toml ,name=frontmatter
[extra]
# See `config.extra.page_info`.
# (type: array of strings; default value: config.extra.page_info;)
page_info = []
# See `config.extra.page_summary_on_paginator`.
# (type: boolean; default value: config.extra.page_summary_on_paginator;)
page_summary_on_paginator = true
# See `config.extra.toc`.
# (type: boolean or object; default value: config.extra.toc;)
toc = true
[extra.cover]
# Width of the cover image in pixels.
# (type: number; default value: uses `get_image_metadata()`;)
width =
# Height of the cover image in pixels.
# (type: number; default value: uses `get_image_metadata()`;)
height =

# Open Graph frontmatter variables
[extra.open_graph]
# When the article is out of date after. e.g. `2024-02-29`.
# (type: datetime; no default value;)
expiration_time =
# Describes the tier status for an article. e.g. `free`, `locked`, or `metered`.
# (type: string; no default value;)
content_tier = ""
# Defines the location to target for the article. e.g. `["county:COUNTY"]` or `["city:CITY,COUNTY"]`.
# (type: array of strings; no default value;)
locations = []
# A high-level section name. e.g. `Technology`.
# (type: string; no default value;)
section = ""
# Indicates whether the article is an opinion piece or not. e.g. `true` or `false`.
# (type: boolean; no default value;)
opinion =
# The URL for the audio.
# (type: string; no default value;)
audio = ""
# MIME type of the audio. e.g. `audio/vnd.facebook.bridge`, `audio/mpeg`.
# (type: string; no default value;)
audio_type = ""
# The URL for the video.
# (type: string; no default value;)
video = ""
# MIME type of the video. e.g. `application/x-shockwave-flash`, `video/mp4`.
# (type: string; no default value;)
video_type = ""
# Width of the video in pixels.
# (type: number; no default value;)
video_width =
# Height of the video in pixels.
# (type: number; no default value;)
video_height =
# Set only if different from canonical page URL.
# (type: string; default value: current_url;)
url = ""

# Sitemap frontmatter variables
[extra.sitemap]
# Set only if different from `page.updated`.
# (type: string; default value: page.updated;)
updated =
# Valid values are `always`, `hourly`, `daily`, `weekly`, `monthly`, `yearly`, `never`.
# (type: string; no default value;)
changefreq =
# Valid values range from 0.0 to 1.0. The default priority of a page is 0.5.
# (type: string; no default value;)
priority =
```

### Home page profile

Create `content/_index.md` file in your blog and set `extra.profile` to your username:

```toml ,name=content/_index.md
+++
# title = ""
# description = ""
sort_by = "date"
paginate_by = 5
[extra]
profile = "your_username"
+++
```

Do it for each language in your blog.
For French, the file name is `content/_index.fr.md`.

See Profiles for more.

### Non-post pages

The default page template `page.html` is for blog posts.
For pages that are not blog posts, create a `pages` directory and use the `pages.html` template.

Create `content/pages/_index.md` file in your blog:

```toml ,name=content/pages/_index.md
+++
render = false
page_template = "pages.html"
+++
```

#### About you page

Create `content/pages/about.md` file in your blog:

```toml ,name=content/pages/about.md
+++
title = "About me"
# description = ""
# path = "about"
+++

## Hello, world!
```

#### Archive page

Create `content/pages/archive.md` file in your blog:

```toml ,name=content/pages/archive.md
+++
title = "Archive"
# description = ""
# path = "archive"
template = "archive.html"
[extra]
section = "_index.md"
+++
```

#### Projects page

Create `content/pages/projects/index.md` file in your blog:

```toml ,name=content/pages/projects/index.md
+++
title = "My Projects"
# description = ""
# path = "projects"
+++
```

Include the following shortcode too: \{\{ projects(path="data.toml", format="toml") \}\}

Create `content/pages/projects/data.toml` file in your blog:

```toml ,name=content/pages/projects/data.toml
[[project]]
name = "lorem"
desc = "Lorem ipsum dolor sit."
tags = ["lorem", "ipsum"]
links = [
    { name = "homepage", url = "https://example.com" },
    { name = "source", url = "https://example.com" },
]
```

### Page authors

Choose one of the following options or skip if you don't know what you're doing:

#### Option A: Using `page.authors`

You don't need to set `page.authors` in the frontmatter if you are the only author of the post.

Otherwise, set `page.authors`:

```toml ,name=frontmatter
+++
authors = ["author_username"]
+++
```

#### Option B: Using Taxonomies

Useful if the blog has a team of several authors.
If you choose this option you should set taxonomies in each page.

```toml ,name=frontmatter
+++
[taxonomies]
authors = ["author_username", "author2_username"]
+++
```

### Inject support

You can easily use inject to add new features to your side without modifying the theme itself.

To use inject, you need to add some HTML files to the `templates/injects` directory.

The available inject points are: `head`, `head_end`, `header_nav`, `body_start`, `body_end`, `page_start`, `page_end`, `footer`.

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

## Configuring

Copy and paste the examples into your `config.toml` file
and comment out the variables you don't use instead of setting empty values.
All variables are optional.

```toml ,name=config.toml
# The URL the site will be built for
base_url = "https://example.com"

# The site theme to use.
theme = "linkita"

# The default language.
default_language = "en"

# The default author for pages.
# See `extra.profiles`.
# (type: string;)
author = "your_username"

# The site title. Will be in all page titles.
# (type: string;)
title = ""

# The site description. Used in feeds by default.
# (type: string;)
description = ""

# Automatically generate a feed.
# (type: boolean;)
generate_feeds = true

# The filenames to use for the feeds.
# (type: array of strings;)
feed_filenames = ["atom.xml"] # or ["rss.xml"]

# Build a search index from the pages and section content
# for `default_language`. (type: boolean;)
build_search_index = true
```

Zola has built-in support for taxonomies.
Taxonomies on Linkita with translated names are `tags`, `categories`, and `authors`.

```toml ,name=config.toml
[[taxonomies]]
name = "categories"
feed = true
paginate_by = 5

[[taxonomies]]
name = "tags"
feed = true
paginate_by = 5

[[taxonomies]]
name = "authors"
feed = true
paginate_by = 5
```

Add more languages by replacing `fr` from the example with the language code:

```toml ,name=config.toml
[languages.fr]
title = "Site title in French"
description = "Site description in French"
generate_feeds = true
feed_filenames = ["atom.xml"] # or ["rss.xml"]
build_search_index = true
taxonomies = [
    { name = "tags", feed = true, paginate_by = 5 }
]
```

### General config

```toml ,name=config.toml
[extra]
# Enable KaTeX math formula support globally.
# (type: boolean; default value: `false`;)
math = false

# Enable Mermaid support globally.
# (type: boolean; default value: `false`;)
mermaid = false

# Enable comments globally.
# (type: boolean; default value: `false`;)
comment = false

# Title separator.
# (type: string; default value: ` | `;)
title_separator = " | "

# The top menu. See `extra.menus`.
# (type: string; no default value;)
header_menu_name = "menu_name"

# If you disable default favicons, you can use
# the inject support to set your own favicons.
# (type: boolean; default value: `false`;)
disable_default_favicon = false

# If you want to reimplement the JS code
# yourself, set to true and use the inject support.
# (type: boolean; default value: `false`;)
disable_javascript = false

# (type: boolean; default value: `false`;)
use_cdn = false

# You can reorder the strings, remove them, or replace them.
# For example, you can replace `site_title` with `home_button`.
# (type: array of strings; default value: `["site_title", "theme_button", "search_button", "translations_button"]`;)
# header_buttons = []

# Valid values:
#  `date`, `date_on_page`, `date_on_paginator`,
#  `date_updated, `date_updated_on_page, `date_updated_on_paginator`,
#  `reading_time, `reading_time_on_page, `reading_time_on_paginator`,
#  `word_count`, `word_count_on_page`, `word_count_on_paginator`,
#  `authors`, `authors_on_page`, `authors_on_paginator`,
#  `tags`, `tags_on_page`, `tags_on_paginator`.
# (type: array of strings; default value: `["date", "date_updated_on_page", "reading_time", "authors"]`;)
# page_info = []

# Prioritize summary over description.
# (type: boolean; default value: `false`;)
# page_summary_on_paginator = true

# Enable table of contents on all pages.
# If not set, toc is enabled only on post pages.
# If set to false, toc is disabled on all pages.
# (type: boolean or object;)
# toc = true

# Reverse the order of prev and next post links.
# (type: boolean; default value: `false`;)
# invert_page_navigation = false
```

### Style config

```toml ,name=config.toml
[extra.style]
# The custom background color. (type: string;)
bg_color = "#f4f4f5"
# The custom background color in dark mode. (type: string;)
bg_dark_color = "#18181b"

# Enable header blur. (type: boolean;)
header_blur = false

# The custom header color, only available
# when `header_blur` is false. (type: string;)
header_color = "#e4e4e7"
# The custom header color in dark mode, only available
# when `header_blur` is false. (type: string;)
header_dark_color = "#27272a"
```

### Menus

```toml ,name=config.toml
[extra.menus]
menu_name = [
  { url = "$BASE_URL/pages/archive/", name = "Archive" },
  { url = "$BASE_URL/categories", name = "Categories" },
  { url = "$BASE_URL/tags/", name = "Tags" },
  { url = "$BASE_URL/pages/about/", name = "About" },
]

# Example multilingual menu.
multilingual_menu_name = [
  { url = "$BASE_URL/pages/about/", names = { en = "About", fr = "About in French" } },
  { url = "$BASE_URL/pages/projects/", names = { en = "Projects", fr = "Projects in French" } },
  { url = "$BASE_URL/pages/archive/", names = { en = "Archive", fr = "Archive in French" } },
  { url = "$BASE_URL/categories/", names = { en = "Categories", fr = "Categories in French" } },
  { url = "$BASE_URL/tags/", names = { en = "Tags", fr = "Tags in French" } },
  { url = "$BASE_URL/authors/", names = { en = "Authors", fr = "Authors in French" } },
]
```

To use a menu, set `extra.header_menu_name`.

`$BASE_URL` in `url` will be automatically replaced with the language specific base url.
You can use `names_i18n` instead of `names`, see the `static/i18n.json` file,
set `names_i18n` to a `common_` key.

### Profiles

```toml ,name=config.toml
# Replace `your_username` with your username.
[extra.profiles.your_username]
# The URL of avatar.
# (type: string; no default value;)
avatar_url = "icons/github.svg"

# A description of what is in the avatar.
# (type: string; no default value;)
avatar_alt = ""

# Invert avatar color in dark mode.
# (type: boolean; default value: `false`;)
avatar_invert = false

# Profile name.
# (type: string; default value: the username;)
name = ""

# Profile bio.
# (type: string; supports markdown; no default value;)
bio = ""

# Social icons.
# `name` should be the file name of `static/icons/*.svg` or the icon name of https://simpleicons.org/
# `url` supports `$BASE_URL`.
# Other: `urls`, `title`, `titles`.
# (type: array of tables; no default value;)
social = [
    { name = "bluesky", url = "https://bsky.app/profile/username" },
    { name = "github", url = "https://github.com/username" },
    { name = "email", url = "mailto:example@example.com" },
    { name = "rss", url = "$BASE_URL/atom.xml" },
]
```

### Profile translations

```toml ,name=config.toml
# For French. Replace `your_username` with your username.
[extra.profiles.your_username.languages.fr]
# A description of what is in the avatar.
# (type: string; default avatar: extra.profiles.your_username.avatar_alt;)
avatar_alt = ""

# Profile name.
# (type: string; default value: extra.profiles.your_username.name;)
name = ""

# Profile bio.
# (type: string; supports markdown; default value: extra.profiles.your_username.bio;)
bio = ""

# Social icons.
# (type: array of tables; default value: extra.profiles.your_username.social;)
social = []
```

### Open Graph for profiles

See [the Open Graph protocol](https://ogp.me/).

```toml ,name=config.toml
# Replace `your_username` with your username.
[extra.profiles.your_username.open_graph]
# The URL of social image. (type: string; no default value;)
image = ""
# A description of what is in the social image. (type: string; default value: "";)
image_alt = ""
# Your first name. (type: string; no default value;)
first_name = ""
# Your last name. (type: string; no default value;)
last_name = ""
# Your username. (type: string; no default value;)
username = ""
# (type: string; no default value;)
gender = "" # "female" or "male"

# Set if you have a Fediverse account. (type: table; no default value;)
#  handle - Your Fediverse handle. (type: string; no default value;)
#  domain - Your Fediverse instance. (type: string; no default value;)
#  url - Your Fediverse account URL. (type: string; optional;)
# Example for @me@mastodon.social:
# fediverse_creator = { handle = "me", domain = "mastodon.social" }
fediverse_creator = { handle = "", domain = "" }
```

`fb_app_id` and `fb_admins` are only allowed in the default author's profile.
In addition, `image` and `image_alt` of the profile will be used as a
fallback open graph image for all pages.

```toml ,name=config.toml
# Replace `your_username` with your username.
[extra.profiles.your_username.open_graph]
# (type: string; no default value;)
fb_app_id = "Your fb app ID"
# (type: array of strings; no default value;)
fb_admins = ["YOUR_USER_ID"]
```

#### Open Graph translations

```toml ,name=config.toml
# For French. Replace `your_username` with your username.
[extra.profiles.your_username.open_graph.languages.fr]
# A description of what is in the social image.
# (type: string; default value: extra.profiles.your_username.open_graph.image_alt;)
image_alt = ""
```

### The page footer

```toml ,name=config.toml
[extra.footer]
# Replace with the correct year.
# (type: number; default value: current year;)
since = 2025

# Replace with the url of the license you want.
# (type: string; no default value; supports `$BASE_URL`;)
license_url = "https://creativecommons.org/licenses/by-sa/4.0/deed"

# Replace `Your Name` with your name and `CC BY-SA 4.0` with the name of the license you want
copyright = "&copy; $YEAR Your Name &vert; [CC BY-SA 4.0]($LICENSE_URL)"

# (type: string; no default value; supports `$BASE_URL`;)
# privacy_policy_url = "$BASE_URL/privacy-policy/"

# (type: string; no default value; supports `$BASE_URL`;)
# terms_of_service_url = "$BASE_URL/terms-of-service/"

# (type: string; no default value; supports `$BASE_URL`;)
# search_page_url = "$BASE_URL/search/"
```

Currently `privacy_policy_url`, `terms_of_service_url`, and `search_page_url` are not shown.

Option `copyright` supports Markdown and:
- `$BASE_URL`
- `$YEAR` (uses `since`)
- `$LICENSE_URL` (uses `license_url`)

### Language specific options

For date format, see [chrono docs](https://docs.rs/chrono/0.4/chrono/format/strftime/index.html).

```toml ,name=config.toml
# For English
[extra.languages.en]
# (type: string; no default value;)
locale = "en_US"

# (type: string; default value: `%F`;)
date_format = "%x"

# (type: string; default value: `%m-%d`;)
date_format_archive = "%m-%d"

# (type: string; default value: extra.header_menu_name;)
# header_menu_name = "menu_name"

# (type: array of strings; default value: extra.header_buttons;)
# header_buttons = []

# To set a different `lang` attribute of the document.
# You can set IETF tag for artificial languages, e.g. `art-x-code`.
# (type: string;)
# language_code = ""

# To use a different interface language, e.g. English. (type: string;)
# i18n_code = "en"

# Set a description for taxonomy pages.
[extra.languages.en.taxonomy_descriptions]
categories = "A map of all categories on this site. Start exploring!"
tags = "A map of all tags on this site. Start exploring!"
authors = "A map of all authors on this site. Start exploring!"

# Set a description for term pages.
# `$NAME` will be automatically replaced.
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
date_format_archive = "%m-%d"
```

### Web analytics

#### GoatCounter

Set only if you use [GoatCounter](https://www.goatcounter.com/).

```toml ,name=config.toml
[extra.goatcounter]
# (type: string; no default value;)
endpoint = "https://MYCODE.goatcounter.com/count"
# (type: string; no default value;)
# noscript_prefix = ""
```

To enable [pixel](https://www.goatcounter.com/help/pixel), set `noscript_prefix` to an empty string.
If your `base_url` includes a subpath, set `noscript_prefix` to the subpath without a trailing slash.

#### Vercel Analytics

Set only if you use [Vercel Web Analytics](https://vercel.com/docs/analytics).

```toml ,name=config.toml
[extra.vercel_analytics]
# (type: string; no default value;)
src = "/_vercel/insights/script.js"
```

#### Prevent tracking own pageviews

Open a page of your site, adding `#disable-analytics` to the page address.
Do this once for each browser and device.
For example, open <http://127.0.0.1:1111/#disable-analytics>.

### Comments

See [giscus.app](https://giscus.app/).
Only available when `extra.comment` in the frontmatter or `extra.comment` in the config is set to `true`.

```toml ,name=config.toml
[extra.giscus]
# (type: string; no default value;)
repo = ""
# (type: string; no default value;)
repo_id = ""
# (type: string; no default value;)
category = ""
# (type: string; no default value;)
category_id = ""
# (type: string; default value: `pathname`;)
mapping = "pathname"
# (type: number; default value: `1`;)
strict = 1
# (type: number; default value: `0`;)
reactions_enabled = 0
# (type: number; default value: `0`;)
emit_metadata = 0
# (type: string; default value: `top`;)
input_position = "top"
# (type: string; default value: `light`;)
theme = "light"
# (type: string; default value: `en`;)
lang = "en"
# (type: string; default value: `lazy`;)
loading = "lazy"
```

## License

See the [MIT License](https://codeberg.org/salif/linkita/src/branch/linkita/LICENSE) file.

## Contributing

Pull requests are welcome on [Codeberg](https://codeberg.org/salif/linkita) and [Github](https://github.com/salif/linkita).

If you notice even the slightest ambiguity or bug in this repo,
report it IMMEDIATELY, before it breeds and takes over the entire project!

## Sites using Linkita

- [Zola Themes Collection](https://github.com/salif/zola-themes-collection)
- [salif.eu](https://github.com/salif/personal-web-page): Personal website
- [Rratic's blog](https://github.com/Rratic/rratic.github.io): Personal website

If your blog uses Linkita and is open source, feel free to create a pull request to add it to this list.

        