
+++
title = "Linkita"
description = "A clean and elegant blog theme for Zola. Linkita is based on Kita and Hugo-Paper and is multilingual and SEO friendly."
template = "theme.html"
date = 2024-12-16T13:54:28+02:00

[taxonomies]
theme-tags = ['Blog', 'Multilingual', 'Responsive', 'SEO', 'Search']

[extra]
created = 2024-12-16T13:54:28+02:00
updated = 2024-12-16T13:54:28+02:00
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
- Live preview in [English](https://salif.github.io/linkita/en/), [Bulgarian](https://salif.github.io/linkita/), [Esperanto](https://salif.github.io/linkita/eo/). See [demo source code](https://codeberg.org/salif/linkita-demo).
- Screenshots for [light mode](https://codeberg.org/salif/linkita/src/branch/linkita/screenshot.png), [dark mode](https://codeberg.org/salif/linkita/src/branch/linkita/screenshot.dark.png).
- For discussion, join the [Matrix chat room](https://matrix.to/#/#linkita:mozilla.org).

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

- i18n
- Improved SEO
- Author profiles
- Search (elasticlunr_javascript)
- Keyboard shortcuts

## Installing

1. Add this theme as a submodule:

```sh
git submodule add https://codeberg.org/salif/linkita.git themes/linkita
```

Alternatively, clone the repository: `git clone https://codeberg.org/salif/linkita.git themes/linkita`.

2. Set `linkita` as your theme in your `config.toml` file.

```toml
theme = "linkita"
```

3. Optionally, you can switch from the `linkita` branch to the latest release:

```sh
cd themes/linkita
npm run switch-to-latest
```

## Updating

```sh
git submodule update --merge --remote themes/linkita
# cd themes/linkita
# npm run switch-to-latest
```

## Usage

### TOML frontmatter

```toml
+++
title = ""
description = ""
# date = 
# updated = 
[taxonomies]
categories = []
tags = []
authors = []
[extra]
# comment = true
# math = true
# mermaid = true
# page_info = []
[extra.cover]
# image = ""
# alt = ""
+++
```

### YAML frontmatter

```yaml
---
title: ""
description: ""
date: 
# updated: 
taxonomies:
  categories:
  tags:
  authors:
extra:
  comment: false
  math: false
  mermaid: false
  cover:
    image: ""
    alt: ""
---
```

### Open Graph frontmatter options

```toml
[extra.open_graph]
# MIME type of the cover image. e.g. `image/jpeg`, `image/gif`, `image/png`
cover_type = ""
# Width of the cover image in pixels
cover_width =
# Height of the cover image in pixels
cover_height =
# When the article is out of date after. e.g. `2024-02-29`
expiration_time =
# Describes the tier status for an article. e.g. `free`, `locked`, or `metered`
content_tier = ""
# Defines the location to target for the article. e.g. `["county:COUNTY"]` or `["city:CITY,COUNTY"]`
locations = []
# A high-level section name. e.g. `Technology`
section = ""
# Tag words associated with this article
tags = [""]
# Indicates whether the article is an opinion piece or not. e.g. `true` or `false`
opinion =
# The URL for the audio
audio = ""
# MIME type of the audio. e.g. `audio/vnd.facebook.bridge`, `audio/mpeg`
audio_type = ""
# The URL for the video
video = ""
# MIME type of the video. e.g. `application/x-shockwave-flash`, `video/mp4`
video_type = ""
# Width of the video in pixels
video_width =
# Height of the video in pixels
video_height =
# Set only if different from canonical page URL
url = ""
```

### Sitemap frontmatter options

```toml
[extra.sitemap]
# Set only if different from `page.updated`
updated =
# Valid values are `always`, `hourly`, `daily`, `weekly`, `monthly`, `yearly`, `never`
changefreq =
# Valid values range from 0.0 to 1.0. The default priority of a page is 0.5
priority =
```

### Home page profile

Create `content/_index.md` file in your blog and set `extra.profile` to your username:

```toml
+++
sort_by = "date"
paginate_by = 5
[extra]
profile = "your_username"
+++
```

Do it for each language in your blog, for example for French, the file name is `content/_index.fr.md`.

### Profiles for authors

You should add `extra.profiles.author_username` table in your `config.toml` file for each author.
Replace `author_username` with author's username.

### Authors

#### Option 1: Using `page.authors`

You don't need to set `page.authors` in the frontmatter if you are the only author.

Otherwise, set `page.authors`:

```toml
+++
authors = ["author_username", "author2_username"]
+++
```

#### Option 2: Using Taxonomies

If you choose this option you should set taxonomies in each post.

Examples:

**If the blog is your personal blog**:

```toml
+++
[taxonomies]
authors = ["your_username"]
+++
```

**If the blog has a team of multiple authors**:

```toml
+++
[taxonomies]
authors = ["author_username"]
# or:
# authors = ["author_username", "author2_username"]
+++
```

### Archive page

```toml
+++
title = "Archive"
template = "archive.html"
[extra]
section = "_index.md"
+++
```

### Inject support

You can easily use inject to add new features to your side without modifying the theme itself.

To use inject, you need to add some HTML files to the `templates/injects` directory.

The available inject points are: `head`, `header_nav`, `body_start`, `body_end`, `page_start`, `page_end`, `footer`.

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
| Skip to main      | <kbd>Alt</kbd>+<kbd>\-</kbd> |

## Configuring

Copy and paste the examples into your `config.toml` file
and comment out the options you don't use instead of setting empty values.

| key                | type    |
| ------------------ | ------- |
| default_language   | string  |
| author             | string  |
| title              | string  |
| description        | string  |
| generate_feeds     | boolean |
| feed_filenames     | array of strings |
| build_search_index | boolean |
| extra              | table   |

Taxonomies with translated names are `tags`, `categories`, and `authors`.

```toml
# The default language
default_language = "en"

# The default author for pages
author = "your_username"

# The site title
title = ""

# The site description
description = ""

# Automatically generated feed 
generate_feeds = true

# The filenames to use for the feeds
feed_filenames = ["atom.xml"] # or ["rss.xml"]

# Enable search
build_search_index = true
```

```toml
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

Add more languages ​​by replacing `fr` from the example with the language code.

```toml
[languages.fr]
title = "Site title in French"
description = "Site description in French"
generate_feeds = true
feed_filenames = ["atom.xml"] # or ["rss.xml"]

[[languages.fr.taxonomies]]
name = "authors"
feed = true
paginate_by = 5
```

| key                           | type    |
| ----------------------------- | ------- |
| extra.math                    | boolean |
| extra.mermaid                 | boolean |
| extra.comment                 | boolean |
| extra.title_separator         | string  |
| extra.header_menu_name        | string  |
| extra.header_buttons          | array of strings |
| extra.page_info               | array of strings |
| extra.disable_default_favicon | boolean |
| extra.disable_javascript      | boolean |

Tables: `extra.style`, `extra.menus`, `extra.profiles`, `extra.footer`, `extra.languages`, `extra.goatcounter`, `extra.giscus`.

The table below lists valid `extra.page_info` values.
Default value is `["date", "date_updated_on_page", "reading_time", "authors"]`.

| on both        | only on page           | only on paginator           |
| -------------- | ---------------------- | --------------------------- |
| `date`         | `date_on_page`         | `date_on_paginator`         |
| `date_updated` | `date_updated_on_page` | `date_updated_on_paginator` |
| `reading_time` | `reading_time_on_page` | `reading_time_on_paginator` |
| `word_count`   | `word_count_on_page`   | `word_count_on_paginator`   |
| `authors`      | `authors_on_page`      | `authors_on_paginator`      |
| `tags`         | `tags_on_page`         | `tags_on_paginator`         |

Default `extra.header_buttons` value is `["site_title", "theme_button", "search_button", "translations_button"]`.
You can replace `site_title` with `home_button` if you want.

```toml
[extra]
# Enable KaTeX math formula support globally
math = false

# Enable Mermaid support globally 
mermaid = false

# Enable comments globally
comment = false

# Title separator
title_separator = " | "

# The top menu. See `extra.menus`
header_menu_name = "menu_name"

# header_buttons = []
# page_info = []
# disable_default_favicon = true
# disable_javascript = false
```

### Style config

| key | type | default value |
| --- | --- | --- |
| extra.style.bg_color | string | `"#f4f4f5"` |
| extra.style.bg_dark_color | string | `"#18181b"` |
| extra.style.header_blur | boolean | false |
| extra.style.header_color | string | `"#e4e4e7"` |
| extra.style.header_dark_color | string | `"#27272a"` |

```toml
[extra.style]
# The custom background color
bg_color = "#f4f4f5"

# The custom background color in dark mode
bg_dark_color = "#18181b"

# Enable header blur
header_blur = false

# The custom header color, only available when `header_blur` is false
header_color = "#e4e4e7"

# The custom header color in dark mode, only available when `header_blur` is false
header_dark_color = "#27272a"
```

### Menus

| key                                       | type   |
| ----------------------------------------- | ------ |
| extra.menus[menu_name].menu[].url         | string |
| extra.menus[menu_name].menu[].name        | string |
| extra.menus[menu_name].menu[].names       | table  |
| extra.menus[menu_name].menu[].names[lang] | string |
| extra.menus[menu_name].menu[].names_i18n  | string |

`$BASE_URL` in `.url` will be automatically replaced with the language specific base url.
You can use `names_i18n` instead of `names[lang]`, see the `static/i18n.json` file,
set `names_i18n` to a `common_` key.

```toml
[[extra.menus.menu_name]]
url = "$BASE_URL/projects/"
# name = "Projects"
[extra.menus.menu_name.names]
en = "Projects"
# fr = "Projects in French"

[[extra.menus.menu_name]]
url = "$BASE_URL/archive/"
# name = "Archive"
[extra.menus.menu_name.names]
en = "Archive"
# fr = "Archive in French"

[[extra.menus.menu_name]]
url = "$BASE_URL/tags/"
# name = "Tags"
[extra.menus.menu_name.names]
en = "Tags"
# fr = "Tags in French"

[[extra.menus.menu_name]]
url = "$BASE_URL/about/"
# name = "About"
[extra.menus.menu_name.names]
en = "About"
# fr = "About in French"
```

### Profiles

| key                                    | type    |
| -------------------------------------- | ------- |
| extra.profiles[username].avatar_url    | string  |
| extra.profiles[username].avatar_alt    | string  |
| extra.profiles[username].avatar_invert | boolean |
| extra.profiles[username].name          | string  |
| extra.profiles[username].bio           | string  |
| extra.profiles[username].email         | string  |
| extra.profiles[username].url           | string  |
| extra.profiles[username].languages     | table   |
| extra.profiles[username].social        | array of tables |
| extra.profiles[username].open_graph    | table   |

```toml
[extra.profiles.your_username]
# The URL of avatar
avatar_url = "icons/github.svg"

# A description of what is in the avatar
avatar_alt = ""

# Invert avatar color in dark mode
avatar_invert = false

# Profile name for all languages
name = ""

# Profile bio for all languages. Supports Markdown.
bio = ""

# Profile email
# email = ""

# Profile website
# url = ""
```

### Profile translations

| key                                                 | type   |
| --------------------------------------------------- | ------ |
| extra.profiles[username].languages[lang].name       | string |
| extra.profiles[username].languages[lang].bio        | string |
| extra.profiles[username].languages[lang].url        | string |
| extra.profiles[username].languages[lang].avatar_alt | string |

```toml
[extra.profiles.your_username.languages.fr]
# Profile name in French
name = ""

# Profile bio in French
bio = ""
```

### Social icons

| key                                    | type   |
| -------------------------------------- | ------ |
| extra.profiles[username].social[].name | string |
| extra.profiles[username].social[].url  | string |

The `name` should be the file name of `static/icons/*.svg` or the icon name of
[simpleicons.org](https://simpleicons.org/). The `url` supports `$BASE_URL`.

```toml
[[extra.profiles.your_username.social]]
name = "github"
url = "https://github.com/username"

[[extra.profiles.your_username.social]]
name = "bluesky"
url = "https://bsky.app/profile/username"

[[extra.profiles.your_username.social]]
name = "rss"
url = "$BASE_URL/atom.xml"
```

### Open Graph

| key                                            | type   |
| ---------------------------------------------- | ------ |
| extra.profiles[username].open_graph.image      | string |
| extra.profiles[username].open_graph.image_alt  | string |
| extra.profiles[username].open_graph.first_name | string |
| extra.profiles[username].open_graph.last_name  | string |
| extra.profiles[username].open_graph.username   | string |
| extra.profiles[username].open_graph.gender     | string |
| extra.profiles[username].open_graph.fb_app_id  | string |
| extra.profiles[username].open_graph.fb_admins  | array of strings     |
| extra.profiles[username].open_graph.fediverse_creator        | table  |
| extra.profiles[username].open_graph.fediverse_creator.handle | string |
| extra.profiles[username].open_graph.fediverse_creator.domain | string |
| extra.profiles[username].open_graph.fediverse_creator.url    | string |
| extra.profiles[username].open_graph.languages[lang]          | table  |

See [the Open Graph protocol](https://ogp.me/).

```toml
[extra.profiles.your_username.open_graph]
# The URL of social image
image = ""

# A description of what is in the social image
image_alt = ""

first_name = "Your first name"
last_name = "Your last name"
username = "Your username"
gender = "female" # or "male"

# Set if you have a Fediverse account. Example for @user@mastodon.social:
[extra.profiles.your_username.open_graph.fediverse_creator]
# Your Fediverse handle
# handle = "user"
# Your Fediverse instance
# domain = "mastodon.social"
# Your Fediverse account URL
# url = ""

# [extra.profiles.your_username.open_graph.languages.fr]
# A description in French of what is in the social image
# image_alt = ""
```

`fb_app_id` and `fb_admins` are only allowed in the `config.author`'s profile.
In addition, `image` and `image_alt` of the profile will be used as a
fallback open graph image for all pages.

```toml
[extra.profiles.your_username.open_graph]
fb_app_id = "Your fb app ID"
fb_admins = ["YOUR_USER_ID"]
# image = ""
# image_alt = ""
```

### The page footer

| key                               | type   |
| --------------------------------- | ------ |
| extra.footer.since                | number |
| extra.footer.copyright            | string |
| extra.footer.license_url          | string |
| extra.footer.privacy_policy_url   | string |
| extra.footer.terms_of_service_url | string |
| extra.footer.search_page_url      | string |

Currently `privacy_policy_url`, `terms_of_service_url`, and `search_page_url` are not shown.

`$BASE_URL` is supported in the `_url` options.

Option `copyright` supports Markdown and:
- `$BASE_URL`
- `$YEAR` (uses `since`)
- `$LICENSE_URL` (uses `license_url`)

```toml
[extra.footer]
# Replace with the correct year
since = 2024
# Replace with the url of the license you want
license_url = "https://creativecommons.org/licenses/by-sa/4.0/deed"
# Replace `Your Name` with your name and `CC BY-SA 4.0` with the name of the license you want
copyright = "&copy; $YEAR Your Name &vert; [CC BY-SA 4.0]($LICENSE_URL)"
# privacy_policy_url = "$BASE_URL/privacy-policy/"
# terms_of_service_url = "$BASE_URL/terms-of-service/"
# search_page_url = "$BASE_URL/search/"
```

### Locale and Date format

| key                                       | type   | default value |
| ----------------------------------------- | ------ | ------------- |
| extra.languages[lang].locale              | string |               |
| extra.languages[lang].date_format         | string | `%F`          |
| extra.languages[lang].date_format_archive | string | `%m-%d`       |
| extra.languages[lang].header_menu_name    | string |               |
| extra.languages[lang].header_buttons      | array of strings |     |
| extra.languages[lang].art_x_lang          | string |               |

For date format, see [chrono docs](https://docs.rs/chrono/0.4/chrono/format/strftime/index.html).

```toml
[extra.languages.en]
locale = "en_US"
date_format = "%x"
date_format_archive = "%m-%d"

[extra.languages.fr]
locale = "fr_FR"
date_format = "%x"
date_format_archive = "%m-%d"
```

### Web analytics

| key                        | type   |
| -------------------------- | ------ |
| extra.goatcounter.endpoint | string |
| extra.goatcounter.src      | string |

Set only if you use [GoatCounter](https://www.goatcounter.com/).

```toml
[extra.goatcounter]
endpoint = "https://MYCODE.goatcounter.com/count"
src = "//gc.zgo.at/count.js"
```

### Comments

| key                            | type   | default value |
| ------------------------------ | ------ | ------------- |
| extra.giscus.repo              | string |               |
| extra.giscus.repo_id           | string |               |
| extra.giscus.category          | string |               |
| extra.giscus.category_id       | string |               |
| extra.giscus.mapping           | string | `pathname`    |
| extra.giscus.strict            | number | `1`           |
| extra.giscus.reactions_enabled | number | `0`           |
| extra.giscus.emit_metadata     | number | `0`           |
| extra.giscus.input_position    | string | `top`         |
| extra.giscus.theme             | string | `light`       |
| extra.giscus.lang              | string | `en`          |
| extra.giscus.loading           | string | `lazy`        |

See [giscus.app](https://giscus.app/).
Only available when `extra.comment` in the frontmatter or `extra.comment` in the config is set to `true`.

```toml
[extra.giscus]
repo = ""
repo_id = ""
category = ""
category_id = ""
mapping = "pathname"
strict = 1
reactions_enabled = 0
emit_metadata = 0
input_position = "top"
theme = "light"
lang = "en"
loading = "lazy"
```

## License

See the [MIT License](https://codeberg.org/salif/linkita/src/branch/linkita/LICENSE) file.

## Contributing

Pull requests are welcome on [Codeberg](https://codeberg.org/salif/linkita) and [Github](https://github.com/salif/linkita).
Open bug reports and feature requests on [Codeberg](https://codeberg.org/salif/linkita/issues).

## Blogs using this theme

- [Zola Themes Collection](https://salif.github.io/zola-themes-collection/)
- [salif.eu](https://salif.eu): Personal website

If you use Linkita, feel free to create a pull request to add your site to this list.

See also [Google results](https://www.google.com/search?q=%22Powered+by+Zola+and+Linkita%22+-site%3Ahttps%3A%2F%2Fsalif.github.io%2Flinkita%2F)
and [Bing results](https://www.bing.com/search?q=%22Powered+by+Zola+and+Linkita%22+-site%3Ahttps%3A%2F%2Fsalif.github.io%2Flinkita%2F).

        