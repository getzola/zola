+++
title = "Multilingual sites"
weight = 130
+++

Zola supports having a site in multiple languages.

## Configuration
To get started, you will need to add the languages you want to support
to your `zola.toml`. For example:

```toml
[languages.fr]
generate_feeds = true # there will be a feed for French content
build_search_index = true
taxonomies = [
    {name = "auteurs"},
    {name = "tags"},
]

[languages.fr.translations]
summary = "Mon blog"

[languages.it]
# Italian language doesn't have any taxonomies/feed/search index

[languages.it.translations]
summary = "Mio blog"

# translations for the default language are not prefixed by languages.code
[translations]
summary = "My blog"
```

Note: By default, Chinese and Japanese search indexing is not included. You can include
the support by building `zola` using `cargo build --features indexing-ja --features indexing-zh`.
Please also note that, enabling Chinese indexing will increase the binary size by approximately
5 MB while enabling Japanese indexing will increase the binary size by approximately 70 MB
due to the incredibly large dictionaries.

## Content
Once the languages have been added, you can start to translate your content. Zola
uses the filename to detect the language:

- `content/an-article.md`: this will be the default language
- `content/an-article.fr.md`: this will be in French

If the language code in the filename does not correspond to one of the languages or
the default language configured, an error will be shown.

If your default language has an `_index.md` in a directory, you will need to add an `_index.{code}.md`
file with the desired front-matter options as there is no language fallback.

## Inheriting metadata
Translated pages often repeat most of the front matter of the default-language page.
Setting `inherit_metadata = true` in `config.toml` makes a translated page (e.g. `an-article.fr.md`)
inherit every front-matter field it omits from the default-language page (`an-article.md`).

For example, with the setting enabled:

```toml
# an-article.md
+++
title = "On the road"
date = 2026-07-01
weight = 10

[taxonomies]
tags = ["travel"]

[extra]
cover = "road.jpg"
motto = "Always pack light."
+++
```

```toml
# an-article.fr.md
+++
title = "En route"
+++
```

The French page gets `date`, `weight`, `tags`, `cover` and `motto` from the English page.

The merge rule is: tables merge per key recursively, the translated page wins every conflict,
and arrays or scalars replace the inherited value wholly. In the example above, writing
`motto = "Toujours léger."` under `[extra]` in the French page would override only that key
and still inherit `cover`. Setting `tags = []` in the translation empties the inherited list.

Three fields are never inherited: `slug`, `path` and `aliases`, since URLs are language-specific.

Taxonomies defined only for the default language (e.g. `authors` when the translation's
language only defines `auteurs`) are dropped when they arrive via inheritance.

A section can override the site setting with `inherit_metadata = true/false` in its `_index.md`.
The value cascades to that section's pages and subsections: for each page, the closest section
setting wins, preferring the page's own language (`_index.fr.md`) over the default one
(`_index.md`) at the same level.

Under `zola serve --fast`, editing the default-language page does not re-render the
translations that inherit from it until the next full reload.

## Output
Zola outputs the translated content with a base URL of `{base_url}/{code}/`.
The only exception to this is if you are setting a translated page `path` directly in the front matter.
