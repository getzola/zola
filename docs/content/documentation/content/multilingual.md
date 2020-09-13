+++
title = "Multilingual sites"
weight = 130
+++

Zola supports having a site in multiple languages.

## Configuration
To get started, you need to add a default language and any other languages you want to support
to your `config.toml`. For example:

```toml
base_url = "mywebsite.com"

# The default language must be set if you put other languages in the config
default_language = "en"

# Put any options for the default language here
title = "My multilingual site"
taxonomies = [
  { name = "author" }
]

# Languages are set as sub-tables of a top-level `languages` table
[languages]

  # Write the code of the language after the dot
  [languages.fr]
  # Some settings can be overridden for each language
  title = "Mon site multilingue"

  # Taxonomies are defined on a per-language basis
  taxonomies = [
    { name = "auteur" }
  ]

  # You can specify language variants, too
  [languages."de-AT"]
  language_alias = "german"  # And then set a friendly name for them
```

The configuration above creates
- an English version at `mywebsite.com/` with the title "My multilingual site" and a single
  "author" taxonomy,
- a French version at `mywebsite.com/fr" with the title <span lang="fr">"Mon site multilingue"</span>
  and a single <span lang="fr">"auteur"</span> taxonomy, and
- an Austrian German version at `mywebsite.com/german", with no title set and no taxonomies.

Languages are identified by [the type of code](https://en.wikipedia.org/wiki/IETF_language_tag)
that's used in HTML. You can specify an exact language variant, so themes will be in the right
spelling (e.g. *light colors* for `en-US` vs. *light colours* for `en-GB`), and screen readers will
use the right dialect. Note that these are canonicalized, so `en_gb` becomes `en-GB` internally.
Setting an invalid code will result in an error.

Language aliases let you optionally assign a "friendly" name to a language. These will be used for
URLs and in file names. If your site used a previous version of Zola or a different static site
generator in multilingual mode, set these to the language names used by them, to avoid breaking
links to your site. It defaults to the [canonicalized form](https://tools.ietf.org/html/bcp47#section-4.5)
of the language's code.

Language aliases aren't slugified, so URL- and path-safe characters **must** be used.

The default language is configured with the top-level variables. Most of these can be overridden
for each language. Refer to the [`config.toml` options](@/documentation/getting-started/configuration.md)
for specifics.

Note: By default, Chinese and Japanese search indexing is not included. You can include
the support by building `zola` using `cargo build --features search/indexing-ja search/indexing-zh`.
Please also note that, enabling Chinese indexing will increase the binary size by approximately
5 MB while enabling Japanese indexing will increase the binary size by approximately 70 MB 
due to the incredibly large dictionaries.

## Content
Once the languages have been added, you can start to translate your content. Zola
uses the filename to detect the language:

- `content/an-article.md`: this will be the default language
- `content/an-article.fr.md`: this will be in French

If the language code in the filename does not correspond to any of the aliases, an error will be
shown.

If your default language has an `_index.md` in a directory, you will need to add an
`_index.{language_alias}.md` file with the desired front-matter options as there is no
language fallback.

## Output
For the translations, Zola outputs the content with a base URL of `{base_url}/{language_alias}/`.

An exception is if you set a path in the front matter, that path will be used.

## In themes
TODO
