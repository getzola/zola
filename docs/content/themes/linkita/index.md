
+++
title = "Linkita"
description = "A clean and elegant blog theme for Zola. Linkita is based on Kita and Hugo-Paper and is multilingual and SEO friendly."
template = "theme.html"
date = 2024-11-04T05:59:13Z

[extra]
created = 2024-11-04T05:59:13Z
updated = 2024-11-04T05:59:13Z
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

## Demo

- [English](https://salif.github.io/linkita/en/)
- [Bulgarian](https://salif.github.io/linkita/)
- [Esperanto](https://salif.github.io/linkita/eo/)

### Screenshots

| Light mode | Dark mode |
| :---: | :---: |
| ![Screenshot](https://codeberg.org/salif/linkita/raw/branch/linkita/screenshot.png) | ![Screenshot - Dark mode](https://codeberg.org/salif/linkita/raw/branch/linkita/screenshot.dark.png) |

## Kita features

- Easy to use and modify
- No preset limits (This theme does not limit your content directory structure, taxonomy names, etc. It's applicable to all zola sites.)
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

## Linkita features

- i18n
- Author profiles
- Improved SEO

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

3. It is recommended to switch from the `linkita` branch to the latest release:

```sh
cd themes/linkita
npm run switch-to-latest
```

Alternatively, use this command: `./justfile switch-to-latest`.

## Updating

```sh
git submodule update --remote themes/linkita
cd themes/linkita
npm run switch-to-latest
```

## Usage

### TOML front matter

```toml
+++
title = ""
description = ""
# date = 
# updated = 
[taxonomies]
tags = []
authors = []
[extra]
# comment = true
# math = true
# mermaid = true
[extra.cover]
# image = ""
# alt = ""
[extra.open_graph]
+++
```

### YAML front matter

```yaml
---
title: ""
description: ""
date: 
# updated: 
taxonomies:
  tags:
  authors:
    - your_username
extra:
  comment: false
  math: false
  mermaid: false
  cover:
    image: ""
    alt: ""
  open_graph:
---
```

### Open Graph options for pages

| key | type | example | comment |
| --- | --- | --- | --- |
| `cover_type` | string | `image/jpeg`, `image/gif`, `image/png` | MIME type of the cover image |
| `cover_width` | string |  | Width of the cover image in pixels |
| `cover_height` | string |  | Height of the cover image in pixels |
| `expiration_time` | string | `"2024-02-29"` | When the article is out of date after |
| `content_tier` | string | `"free"`, `"locked"`, or `"metered"` | Describes the tier status for an article |
| `locations` | array of strings | `["county:COUNTY"]` or `["city:CITY,COUNTY"]` | Defines the location to target for the article |
| `section` | string |  | A high-level section name. E.g. Technology |
| `tags` | array of strings |  | Tag words associated with this article. |
| `opinion` | string | `"true"` or `"false"` | Indicates whether the article is an opinion piece or not |
| `audio` | string |  | The URL for the audio |
| `audio_type` | string | `audio/vnd.facebook.bridge`, `audio/mpeg` | MIME type of the audio |
| `video` | string |  | The URL for the video |
| `video_type` | string | `application/x-shockwave-flash`, `video/mp4` | MIME type of the video |
| `video_width` | string |  | Width of the video in pixels |
| `video_height` | string |  | Height of the video in pixels |
| `url` | string |  | Set only if different from canonical page URL |

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

#### Option 1: Using Taxonomies

If you choose this option you should set taxonomies in each post.

Examples:

**If the blog is your personal blog**:

```toml
[taxonomies]
authors = ["your_username"]
# or:
# authors = ["your_username", "contributor_username"]
# or:
# authors = ["main_author_of_the_post", "your_username"]
```

**If the blog has a team of multiple authors**:

```toml
[taxonomies]
authors = ["author_username"]
# or:
# authors = ["author_username", "author2_username", etc.]
```

#### Option 2: Using `page.authors`

*TODO*

### Inject support

You can easily use inject to add new features to your side without modifying the theme itself.

To use inject, you need to add some HTML files to the `templates/injects` directory.

The available inject points are: `head`, `header_nav`, `body_start`, `body_end`, `page_start`, `page_end`, `footer`.

For example, to load a custom script, you can add a `templates/injects/head.html` file:

```html
<script src="js-file-path-or-cdn-url.js"></script>
```

## Configuring

Copy and paste the examples into your `config.toml` file
and comment out the options you don't use instead of setting empty values.

All configuration options used by this theme are listed in tables.

| key | type |
| --- | --- |
| `default_language` | string |
| `author` | string |
| `title` | string |
| `description` | string |
| `generate_feeds` | boolean |
| `feed_filenames` | array of strings |
| `extra` | table |

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
```

```toml
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
name = "tags"
feed = true
paginate_by = 5

[[languages.fr.taxonomies]]
name = "authors"
feed = true
paginate_by = 5
```

| key | type | comment |
| --- | --- | --- |
| `extra.math` | boolean | Enable KaTeX math formula support globally |
| `extra.mermaid` | boolean | Enable Mermaid support globally |
| `extra.comment` | boolean | Enable comment support globally |
| `extra.title_separator` | string | Title Separator |
| `extra.page_info` | array of strings | Show page date, reading time, author names |
| `extra.style` | table | The theme style config |
| `extra.profiles` | table | Author profiles |
| `extra.menu` | array of tables | The top menu |
| `extra.footer` | table | The page footer options |
| `extra.locales` | table | Locale codes and date formats |
| `extra.goatcounter` | table | Enable web analytics |
| `extra.giscus` | table | The giscus comment options |

Strings in `extra.page_info` that are not one of the following, will be displayed directly in the UI:
`"date"`, `"date_updated"`, `"reading_time"`, `"word_count"`, `"authors"`.
Default `extra.page_info` value is `["date", "date_updated", "reading_time", "authors"]`.

```toml
[extra]
math = false
mermaid = false
comment = false
title_separator = " | "
# page_info = ["date", "date_updated", "reading_time", "word_count", "authors"]
```

### Style config (`extra.style`)

| key | type | default value | comment |
| --- | --- | --- | --- |
| `extra.style.bg_color` | string | `"#f4f4f5"` | The custom background color |
| `extra.style.bg_dark_color` | string | `"#18181b"` | The custom background color in dark mode |
| `extra.style.header_blur` | boolean |  | Enable header blur |
| `extra.style.header_color` | string | `"#e4e4e7"` | The custom header color, only available when `header_blur` is false |
| `extra.style.header_dark_color` | string | `"#27272a"` | The custom header color in dark mode, only available when `header_blur` is false |

```toml
[extra.style]
bg_color = "#f4f4f5"
bg_dark_color = "#18181b"
header_blur = false
header_color = "#e4e4e7"
header_dark_color = "#27272a"
```

### Profiles (`extra.profiles`)

| key | type | comment |
| --- | --- | --- |
| `extra.profiles[username]` | table |  |
| `extra.profiles[username].avatar_url` | string | The URL of avatar |
| `extra.profiles[username].avatar_alt` | string | A description of what is in the avatar |
| `extra.profiles[username].avatar_invert` | boolean | Invert color in dark mode |
| `extra.profiles[username].name` | string | Profile name for all languages |
| `extra.profiles[username].bio` | string | Profile bio for all languages |
| `extra.profiles[username].email` | string | Profile email |
| `extra.profiles[username].url` | string | Profile website |
| `extra.profiles[username].translations` | table | Profile name and bio translations |
| `extra.profiles[username].social` | array of tables | The social icons below the profile |
| `extra.profiles[username].open_graph` | table | Open Graph |

```toml
[extra.profiles.your_username]
avatar_url = "icons/github.svg"
avatar_alt = ""
avatar_invert = true
# name = ""
# bio = ""
```

### Profile translations (`extra.profiles[username].translations`)

| key | type |
| --- | --- |
| `extra.profiles[username].translations[lang]` | table |
| `extra.profiles[username].translations[lang].name` | string |
| `extra.profiles[username].translations[lang].bio` | string |
| `extra.profiles[username].translations[lang].url` | string |
| `extra.profiles[username].translations[lang].avatar_alt` | string |

```toml
[extra.profiles.your_username.translations.fr]
name = "Profile name in French"
bio = "Profile bio in French"
```

### Social icons `extra.profiles[username].social`

| key | type |
| --- | --- |
| `extra.profiles[username].social[].name` | string |
| `extra.profiles[username].social[].url` | string |

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

### Open Graph (`extra.profiles[username].open_graph`)

| key | type | comment |
| --- | --- | --- |
| `extra.profiles[username].open_graph.image` | string | The URL of social image |
| `extra.profiles[username].open_graph.image_alt` | string | A description of what is in the social image |
| `extra.profiles[username].open_graph.first_name` | string | A name normally given to an individual by a parent or self-chosen |
| `extra.profiles[username].open_graph.last_name` | string | A name inherited from a family or marriage and by which the individual is commonly known |
| `extra.profiles[username].open_graph.username` | string | A short unique string to identify them |
| `extra.profiles[username].open_graph.gender` | string | Their gender |
| `extra.profiles[username].open_graph.fb_app_id` | string | Set fb:app_id |
| `extra.profiles[username].open_graph.fb_admins` | array of strings | Set fb:admins |
| `extra.profiles[username].open_graph.fediverse_creator` | table | Set if you a Fediverse account |
| `extra.profiles[username].open_graph.fediverse_creator.handle` | string | Your Fediverse handle |
| `extra.profiles[username].open_graph.fediverse_creator.domain` | string | Your Fediverse instance |
| `extra.profiles[username].open_graph.fediverse_creator.url` | string | Your Fediverse account URL |
| `extra.profiles[username].open_graph.translations[lang].image_alt` | string | A description of what is in the social image |

See [the Open Graph protocol](https://ogp.me/).

```toml
[extra.profiles.your_username.open_graph]
# image = ""
# image_alt = ""
first_name = "Your first name"
last_name = "Your last name"
username = "Your username"
gender = "female" # or "male"

[extra.profiles.your_username.open_graph.fediverse_creator]
# Example for "@user@mastodon.social"
handle = "user"
domain = "mastodon.social"
```

`fb_app_id` and `fb_admins` are only allowed in the `config.author`'s profile.
In addition, `image` and `image_alt` of the profile will be used as a
fallback open graph image for all pages.

```toml
[extra.profiles.default_author.open_graph]
fb_app_id = "Your fb app ID"
fb_admins = ["YOUR_USER_ID"]
# image = ""
# image_alt = ""
```

### The top menu (`extra.menu`)

| key | type |
| --- | --- |
| `extra.menu[].url` | string |
| `extra.menu[].name` | string |
| `extra.menu[].names` | table |
| `extra.menu[].names[lang]` | string |
| `extra.menu[].names_i18n` | string |

`$BASE_URL` will be automatically translated into the language specific base url.
You can use `names_i18n` instead of `names[lang]`, see the `static/i18n.json` file,
set `names_i18n` to a `common_` key.

```toml
[[extra.menu]]
url = "$BASE_URL/projects/"
# name = "Projects"
[extra.menu.names]
en = "Projects"
# fr = "Projects in French"

[[extra.menu]]
url = "$BASE_URL/archive/"
# name = "Archive"
[extra.menu.names]
en = "Archive"
# fr = "Archive in French"

[[extra.menu]]
url = "$BASE_URL/tags/"
# name = "Tags"
[extra.menu.names]
en = "Tags"
# fr = "Tags in French"

[[extra.menu]]
url = "$BASE_URL/about/"
# name = "About"
[extra.menu.names]
en = "About"
# fr = "About in French"
```

### The page footer (`extra.footer`)

| key | type |
| --- | --- |
| `extra.footer.author_name` | string |
| `extra.footer.since` | number |
| `extra.footer.license_name` | string |
| `extra.footer.license_url` | string |
| `extra.footer.privacy_policy_url` | string |
| `extra.footer.terms_of_service_url` | string |
| `extra.footer.search_page_url` | string |

Currently `privacy_policy_url`, `terms_of_service_url`, and `search_page_url` are only used in `<head>`.

`$BASE_URL` is supported in the `_url` options.

```toml
[extra.footer]
since = 2024
license_name = "CC BY-SA 4.0"
license_url = "https://creativecommons.org/licenses/by-sa/4.0/deed"
# privacy_policy_url = "$BASE_URL/privacy-policy/"
# terms_of_service_url = "$BASE_URL/terms-of-service/"
# search_page_url = "$BASE_URL/search/"
```

### Locale and Date format (`extra.locales`)

| key | type | default value |
| --- | --- | --- |
| `extra.locales[lang].locale` | string |  |
| `extra.locales[lang].date_format` | string | `%F` |
| `extra.locales[lang].date_format_archive` | string | `%m-%d` |

For date format, see [chrono docs](https://docs.rs/chrono/0.4/chrono/format/strftime/index.html).

```toml
[extra.locales.en]
locale = "en_US"
date_format = "%x"
date_format_archive = "%m-%d"

[extra.locales.fr]
locale = "fr_FR"
date_format = "%x"
date_format_archive = "%m-%d"
```

### Web analytics (`extra.goatcounter`)

| key | type |
| --- | --- |
| `extra.goatcounter.endpoint` | string |
| `extra.goatcounter.src` | string |

Set only if you use [GoatCounter](https://www.goatcounter.com/).

```toml
[extra.goatcounter]
endpoint = "https://MYCODE.goatcounter.com/count"
src = "//gc.zgo.at/count.js"
```

### Comments (`extra.giscus`)

| key | type | default value |
| --- | --- | --- |
| `extra.giscus.repo` | string |  |
| `extra.giscus.repo_id` | string |  |
| `extra.giscus.category` | string |  |
| `extra.giscus.category_id` | string |  |
| `extra.giscus.mapping` | string | `pathname` |
| `extra.giscus.strict` | number | `1` |
| `extra.giscus.reactions_enabled` | number | `0` |
| `extra.giscus.emit_metadata` | number | `0` |
| `extra.giscus.input_position` | string | `top` |
| `extra.giscus.theme` | string | `light` |
| `extra.giscus.lang` | string | `en` |
| `extra.giscus.loading` | string | `lazy` |

Only available when `extra.comment` in the frontmatter or `extra.comment` in the config is set to `true`. See [giscus.app](https://giscus.app/).

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
Open *bug reports* and *feature requests* on [Codeberg](https://codeberg.org/salif/linkita/issues).

If you want to add new translations or correct existing ones, please find another person who speaks the language to confirm your translations are good, by adding a comment or review on your pull request.

## Blogs using this theme

- [salif.eu](https://salif.eu): Personal website

If you use Linkita, feel free to create a pull request to add your site to this list.

        