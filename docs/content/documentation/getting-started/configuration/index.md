+++
title = "Configuration"
weight = 40
+++

The default configuration is sufficient to get Zola running locally but not more than that.
It follows the philosophy of paying for only what you need, almost everything is turned off by default.

To change the configuration, edit the `config.toml` file.
If you are not familiar with TOML, have a look at [the TOML spec](https://github.com/toml-lang/toml).

⚠️ If you add keys to your `config.toml`, you must pay attention to which TOML section it belongs to. A TOML section starts with a header, e.g. `[search]`, and ends at the next section or EOF.

Here are the current `config.toml` sections:
1. main (unnamed)
2. markdown
3. link_checker
4. slugify
5. search
6. translations
7. languages
8. extra

**Only the `base_url` variable is mandatory**. Everything else is optional. All configuration variables
used by Zola as well as their default values are listed below:

```toml
# The base URL of the site; the only required configuration variable.
base_url = "https://mywebsite.com"

# The site title and description; used in feeds by default.
title = ""
description = ""

# The default language; used in feeds.
default_language = "en"

# The site theme to use.
theme = ""

# For overriding the default output directory `public`, set it to another value (e.g.: "docs")
output_dir = "public"

# Whether dotfiles at the root level of the output directory are preserved when (re)building the site.
# Enabling this also prevents the deletion of the output folder itself on rebuilds.
preserve_dotfiles_in_output = false

# When set to "true", the Sass files in the `sass` directory in the site root are compiled.
# Sass files in theme directories are always compiled.
compile_sass = false

# When set to "true", the generated HTML files are minified.
minify_html = false

# A list of glob patterns specifying asset files to ignore when the content
# directory is processed. Defaults to none, which means that all asset files are
# copied over to the `public` directory.
# Example:
#     ignored_content = ["*.{graphml,xlsx}", "temp.*", "**/build_folder"]
ignored_content = []

# Similar to ignored_content, a list of glob patterns specifying asset files to
# ignore when the static directory is processed. Defaults to none, which means
# that all asset files are copied over to the `public` directory
ignored_static = []

# When set to "true", a feed is automatically generated.
generate_feeds = false

# When set to "all", paginated pages are not a part of the sitemap, default is "none"
exclude_paginated_pages_in_sitemap = "none"

# The filenames to use for the feeds. Used as the template filenames, too.
# Defaults to ["atom.xml"], which has a built-in template that renders an Atom 1.0 feed.
# There is also a built-in template "rss.xml" that renders an RSS 2.0 feed.
feed_filenames = ["atom.xml"]

# The number of articles to include in the feed. All items are included if
# this limit is not set (the default).
# feed_limit = 20

# When set to "true", files in the `static` directory are hard-linked. Useful for large
# static files. Note that for this to work, both `static` and the
# output directory need to be on the same filesystem. Note that the theme's `static`
# files are always copied, regardless of this setting.
hard_link_static = false

# The default author for pages
author =

# The taxonomies to be rendered for the site and their configuration of the default languages
# Example:
#     taxonomies = [
#       {name = "tags", feed = true}, # each tag will have its own feed
#       {name = "tags"}, # you can have taxonomies with the same name in multiple languages
#       {name = "categories", paginate_by = 5},  # 5 items per page for a term
#       {name = "authors"}, # Basic definition: no feed or pagination
#     ]
#
taxonomies = []

# When set to "true", a search index is built from the pages and section
# content for `default_language`.
build_search_index = false

# When set to "false", Sitemap.xml is not generated
generate_sitemap = true

# When set to "false", robots.txt is not generated
generate_robots_txt = true

# Configuration of the Markdown rendering
[markdown]
# When set to "true", all code blocks are highlighted.
highlight_code = false

# When set to "true", missing highlight languages are treated as errors. Defaults to false.
error_on_missing_highlight = false

# A list of directories used to search for additional `.sublime-syntax` and `.tmTheme` files.
extra_syntaxes_and_themes = []

# The theme to use for code highlighting.
# See below for list of allowed values.
highlight_theme = "base16-ocean-dark"

# When set to "true", emoji aliases translated to their corresponding
# Unicode emoji equivalent in the rendered Markdown files. (e.g.: :smile: => 😄)
render_emoji = false

# CSS class to add to external links (e.g. "external-link")
external_links_class =

# Whether external links are to be opened in a new tab
# If this is true, a `rel="noopener"` will always automatically be added for security reasons
external_links_target_blank = false

# Whether to set rel="nofollow" for all external links
external_links_no_follow = false

# Whether to set rel="noreferrer" for all external links
external_links_no_referrer = false

# Whether smart punctuation is enabled (changing quotes, dashes, dots in their typographic form)
# For example, `...` into `…`, `"quote"` into `“curly”` etc
smart_punctuation = false

# Whether parsing of definition lists is enabled
definition_list = false

# Whether to set decoding="async" and loading="lazy" for all images
# When turned on, the alt text must be plain text.
# For example, `![xx](...)` is ok but `![*x*x](...)` isn’t ok
lazy_async_image = false

# Whether footnotes are rendered in the GitHub-style (at the bottom, with back references) or plain (in the place, where they are defined)
bottom_footnotes = false

# This determines whether to insert a link for each header like the ones you can see on this site if you hover over
# a header.
# The default template can be overridden by creating an `anchor-link.html` file in the `templates` directory.
# This value can be "left", "right", "heading" or "none".
# "heading" means the full heading becomes the text of the anchor.
# See "Internal links & deep linking" in the documentation for more information.
insert_anchor_links = "none"

# Configuration of the link checker.
[link_checker]
# Skip link checking for external URLs that start with these prefixes
skip_prefixes = [
    "http://[2001:db8::]/",
]

# Skip anchor checking for external URLs that start with these prefixes
skip_anchor_prefixes = [
    "https://caniuse.com/",
]

# Treat internal link problems as either "error" or "warn", default is "error"
internal_level = "error"

# Treat external link problems as either "error" or "warn", default is "error"
external_level = "error"

# Various slugification strategies, see below for details
# Defaults to everything being a slug
[slugify]
paths = "on"
taxonomies = "on"
anchors = "on"
# Whether to remove date prefixes for page path slugs.
# For example, content/posts/2016-10-08_a-post-with-dates.md => posts/a-post-with-dates
# When true, content/posts/2016-10-08_a-post-with-dates.md => posts/2016-10-08-a-post-with-dates
paths_keep_dates = false

[search]
# Whether to include the title of the page/section in the index
include_title = true
# Whether to include the description of the page/section in the index
include_description = false
# Whether to include the RFC3339 datetime of the page in the search index
include_date = false
# Whether to include the path of the page/section in the index (the permalink is always included)
include_path = false
# Whether to include the rendered content of the page/section in the index
include_content = true
# At which code point to truncate the content to. Useful if you have a lot of pages and the index would
# become too big to load on the site. Defaults to not being set.
# truncate_content_length = 100

# Whether to produce the search index as a javascript file or as a JSON file
# Accepted values:
# - "elasticlunr_javascript", "elasticlunr_json"
# - "fuse_javascript", "fuse_json"
index_format = "elasticlunr_javascript"

# Optional translation object for the default language
# Example:
#     default_language = "fr"
#
#     [translations]
#     title = "Un titre"
#
[translations]

# Additional languages definition
# You can define language specific config values and translations:
# title, description, generate_feeds, feed_filenames, taxonomies, build_search_index
# as well as its own search configuration and translations (see above for details on those)
[languages]
# For example
# [languages.fr]
# title = "Mon blog"
# generate_feeds = true
# taxonomies = [
#    {name = "auteurs"},
#    {name = "tags"},
# ]
# build_search_index = false

# You can put any kind of data here. The data
# will be accessible in all templates
# Example:
#     [extra]
#     author = "Famous author"
#
# author value will be available using {{ config.extra.author }} in templates
#
[extra]
```

## Syntax highlighting

Zola currently has the following highlight themes available:

<details>
    <summary><a class="external" href="https://tmtheme-editor.glitch.me/#!/editor/theme/1337">1337</a></summary>
    <img src="./images/1337.png" alt="1337 preview">
</details>
<details>
    <summary><a class="external" href="https://tmtheme-editor.glitch.me/#!/editor/theme/Agola%20Dark">agola-dark</a></summary>
    <img src="./images/agola-dark.png" alt="agola-dark preview">
</details>
<details>
    <summary><a class="external" href="https://tmtheme-editor.glitch.me/#!/editor/theme/Ascetic%20White">ascetic-white</a></summary>
    <img src="./images/ascetic-white.png" alt="ascetic-white preview">
</details>
<details>
    <summary><a class="external" href="https://tmtheme-editor.glitch.me/#!/editor/theme/Axar">axar</a></summary>
    <img src="./images/axar.png" alt="axar preview">
</details>
<details>
    <summary><a class="external" href="https://github.com/dempfi/ayu">ayu-dark</a></summary>
    <img src="./images/ayu-dark.png" alt="ayu-dark preview">
</details>
<details>
    <summary><a class="external" href="https://github.com/dempfi/ayu">ayu-light</a></summary>
    <img src="./images/ayu-light.png" alt="ayu-light preview">
</details>
<details>
    <summary><a class="external" href="https://github.com/dempfi/ayu">ayu-mirage</a></summary>
    <img src="./images/ayu-mirage.png" alt="ayu-mirage preview">
</details>
<details>
    <summary><a class="external" href="https://atelierbram.github.io/syntax-highlighting/atelier-schemes/dune/">base16-atelierdune-light</a></summary>
    <img src="./images/base16-atelierdune-light.png" alt="base16-atelierdune-light preview">
</details>
<details>
    <summary><a class="external" href="https://tmtheme-editor.glitch.me/#!/editor/theme/Base16%20Ocean%20Dark">base16-ocean-dark</a></summary>
    <img src="./images/base16-ocean-dark.png" alt="base16-ocean-dark preview">
</details>
<details>
    <summary><a class="external" href="https://tmtheme-editor.glitch.me/#!/editor/theme/Base16%20Ocean%20Light">base16-ocean-light</a></summary>
    <img src="./images/base16-ocean-light.png" alt="base16-ocean-light preview">
</details>
<details>
    <summary><a class="external" href="https://tmtheme-editor.glitch.me/#!/editor/theme/BBEdit">bbedit</a></summary>
    <img src="./images/bbedit.png" alt="bbedit preview">
</details>
<details>
    <summary><a class="external" href="https://tmtheme-editor.glitch.me/#!/editor/theme/Boron">boron</a></summary>
    <img src="./images/boron.png" alt="boron preview">
</details>
<details>
    <summary><a class="external" href="https://tmtheme-editor.glitch.me/#!/editor/theme/Charcoal">charcoal</a></summary>
    <img src="./images/charcoal.png" alt="charcoal preview">
</details>
<details>
    <summary><a class="external" href="https://tmtheme-editor.glitch.me/#!/editor/theme/Cheerfully%20Light">cheerfully-light</a></summary>
    <img src="./images/cheerfully-light.png" alt="cheerfully-light preview">
</details>
<details>
    <summary><a class="external" href="https://tmtheme-editor.glitch.me/#!/editor/theme/Classic%20Modified">classic-modified</a></summary>
    <img src="./images/classic-modified.png" alt="classic-modified preview">
</details>
<details>
    <summary><a class="external" href="https://tmtheme-editor.glitch.me/#!/editor/theme/Demain">demain</a></summary>
    <img src="./images/demain.png" alt="demain preview">
</details>
<details>
    <summary><a class="external" href="https://tmtheme-editor.glitch.me/#!/editor/theme/Dimmed%20Fluid">dimmed-fluid</a></summary>
    <img src="./images/dimmed-fluid.png" alt="dimmed-fluid preview">
</details>
<details>
    <summary><a class="external" href="https://draculatheme.com/">dracula</a></summary>
    <img src="./images/dracula.png" alt="dracula preview">
</details>
<details>
    <summary><a class="external" href="https://tmtheme-editor.glitch.me/#!/editor/theme/Gray%20Matter%20Dark">gray-matter-dark</a></summary>
    <img src="./images/gray-matter-dark.png" alt="gray-matter-dark preview">
</details>
<details>
    <summary><a class="external" href="https://github.com/kristopherjohnson/MonochromeSublimeText">green</a></summary>
    <img src="./images/green.png" alt="green preview">
</details>
<details>
    <summary><a class="external" href="https://github.com/morhetz/gruvbox">gruvbox-dark</a></summary>
    <img src="./images/gruvbox-dark.png" alt="gruvbox-dark preview">
</details>
<details>
    <summary><a class="external" href="https://github.com/morhetz/gruvbox">gruvbox-light</a></summary>
    <img src="./images/gruvbox-light.png" alt="gruvbox-light preview">
</details>
<details>
    <summary><a class="external" href="https://tmtheme-editor.glitch.me/#!/editor/theme/IDLE">idle</a></summary>
    <img src="./images/idle.png" alt="idle preview">
</details>
<details>
    <summary><a class="external" href="https://tmtheme-editor.glitch.me/#!/editor/theme/Inspiredgithub">inspired-github</a></summary>
    <img src="./images/inspired-github.png" alt="inspired-github preview">
</details>
<details>
    <summary><a class="external" href="https://tmtheme-editor.glitch.me/#!/editor/theme/IR_White">ir-white</a></summary>
    <img src="./images/ir-white.png" alt="ir-white preview">
</details>
<details>
    <summary><a class="external" href="https://tmtheme-editor.glitch.me/#!/editor/theme/Kronuz">kronuz</a></summary>
    <img src="./images/kronuz.png" alt="kronuz preview">
</details>
<details>
    <summary><a class="external" href="https://tmtheme-editor.glitch.me/#!/editor/theme/Material%20Dark">material-dark</a></summary>
    <img src="./images/material-dark.png" alt="material-dark preview">
</details>
<details>
    <summary><a class="external" href="https://tmtheme-editor.glitch.me/#!/editor/theme/Material%20Light">material-light</a></summary>
    <img src="./images/material-light.png" alt="material-light preview">
</details>
<details>
    <summary><a class="external" href="https://tmtheme-editor.glitch.me/#!/editor/theme/Monokai">monokai</a></summary>
    <img src="./images/monokai.png" alt="monokai preview">
</details>
<details>
    <summary><a class="external" href="https://github.com/crabique/Nord-plist/tree/0d655b23d6b300e691676d9b90a68d92b267f7ec">nord</a></summary>
    <img src="./images/nord.png" alt="nord preview">
</details>
<details>
    <summary><a class="external" href="https://github.com/GalAster/vscode-theme-nyx">nyx-bold</a></summary>
    <img src="./images/nyx-bold.png" alt="nyx-bold preview">
</details>
<details>
    <summary><a class="external" href="https://github.com/andresmichel/one-dark-theme">one-dark</a></summary>
    <img src="./images/one-dark.png" alt="one-dark preview">
</details>
<details>
    <summary><a class="external" href="https://github.com/sonph/onehalf">OneHalfDark</a></summary>
    <img src="./images/OneHalfDark.png" alt="OneHalfDark preview">
</details>
<details>
    <summary><a class="external" href="https://github.com/sonph/onehalf">OneHalfLight</a></summary>
    <img src="./images/OneHalfLight.png" alt="OneHalfLight preview">
</details>
<details>
    <summary><a class="external" href="https://github.com/tompave/rails_base_16">railsbase16-green-screen-dark</a></summary>
    <img src="./images/railsbase16-green-screen-dark.png" alt="railsbase16-green-screen-dark preview">
</details>
<details>
    <summary><a class="external" href="https://tmtheme-editor.glitch.me/#!/editor/theme/Solarized%20(dark)">solarized-dark</a></summary>
    <img src="./images/solarized-dark.png" alt="solarized-dark preview">
</details>
<details>
    <summary><a class="external" href="https://tmtheme-editor.glitch.me/#!/editor/theme/Solarized%20(light)">solarized-light</a></summary>
    <img src="./images/solarized-light.png" alt="solarized-light preview">
</details>
<details>
    <summary><a class="external" href="https://github.com/idleberg/Subway.tmTheme">subway-madrid</a></summary>
    <img src="./images/subway-madrid.png" alt="subway-madrid preview">
</details>
<details>
    <summary><a class="external" href="https://github.com/idleberg/Subway.tmTheme">subway-moscow</a></summary>
    <img src="./images/subway-moscow.png" alt="subway-moscow preview">
</details>
<details>
    <summary><a class="external" href="https://tmtheme-editor.glitch.me/#!/editor/theme/Tomorrow">Tomorrow</a></summary>
    <img src="./images/Tomorrow.png" alt="Tomorrow preview">
</details>
<details>
    <summary><a class="external" href="https://github.com/erremauro/TwoDark">two-dark</a></summary>
    <img src="./images/two-dark.png" alt="two-dark preview">
</details>
<details>
    <summary><a class="external" href="https://tmtheme-editor.glitch.me/#!/editor/theme/Visual%20Studio%20Dark">visual-studio-dark</a></summary>
    <img src="./images/visual-studio-dark.png" alt="visual-studio-dark preview">
</details>
<details>
    <summary><a class="external" href="https://github.com/colinta/zenburn">zenburn</a></summary>
    <img src="./images/zenburn.png" alt="zenburn preview">
</details>

Zola uses the Sublime Text themes, making it very easy to add more.
If you want a theme not listed above, please open an issue or a pull request on the [Zola repo](https://github.com/getzola/zola).

Alternatively you can use the `extra_syntaxes_and_themes` configuration option to load your own custom themes from a .tmTheme file.
See [Syntax Highlighting](@/documentation/content/syntax-highlighting.md) for more details.

## Slugification strategies

By default, Zola will turn every path, taxonomies and anchors to a slug, an ASCII representation with no special characters.
You can however change that strategy for each kind of item, if you want UTF-8 characters in your URLs for example. There are 3 strategies:

- `on`: the default one, everything is turned into a slug
- `safe`: characters that cannot exist in files on Windows (`<>:"/\|?*`) or Unix (`/`) are removed, everything else stays
- `off`: nothing is changed, your site might not build on some OS and/or break various URL parsers

Since there are no filename issues with anchors, the `safe` and `off` strategies are identical in their case: the only change
is space being replaced by `_` since a space is not valid in an anchor.

Note that if you are using a strategy other than the default, you will have to manually escape whitespace and Markdown
tokens to be able to link to your pages. For example an internal link to a file named `some space.md` will need to be
written like `some%20space.md` in your Markdown files.
