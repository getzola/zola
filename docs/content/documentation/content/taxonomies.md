+++
title = "Taxonomies"
weight = 90
+++

Zola has built-in support for *taxonomies*.

Taxonomies are user-defined groupings of content. Each page and section can belong to zero or more
*terms* in a taxonomy. This is different from [sections](@/documentation/content/section.md) where
it can belong to only one.

On most sites, you might come across two taxonomies: `tags` and `categories`. Usually, a page can
have a single category, and can have multiple tags. However, this is not a requirement.

## Configuration

A taxonomy has four variables:

- `name`: a required string that will be used in the URLs, usually the plural form (i.e., tags, categories, etc.)
- `paginate_by`: if this is set to a number, each term page will be paginated by this much.
- `paginate_path`: if set, this path will be used by the paginated page and the page number will be appended after it.
For example the default would be page/1.
- `feed`: if set to `true`, a feed (Atom by default) will be generated for each term.

Taxonomies only apply to pages in a particular language. Multilingual sites can have different
taxonomies for each language. These are defined in the corresponding `[languages.{code}]` section.

Insert into the configuration file (`config.toml`):

⚠️ Place the taxonomies key in the main section and not in the `[extra]` section

**Example 1:** (one language)

```toml
taxonomies = [
    {name = "categories", feed = true},
    {name = "tags"}
]
```

**Example 2:** (multilingual site)

```toml
default_language = "en"

taxonomies = [
    {name = "tags"}
]

[langugages]

    [languages.fr]
    taxonomies = [
        {name = "tags"}
    ]

    [languages."de-AT"]
    taxonomies = [
        {name = "tags"}
    ]
]
```

## Using taxonomies

Once the configuration is done, you can then set taxonomies in the front matter and Zola will pick them up:

**Example:**

```toml
+++
title = "Writing a static-site generator in Rust"
date = 2019-08-15

[taxonomies]
tags = ["rust", "web"]
categories = ["programming"]
+++
```

## Output paths

Similarly to how sections and pages compute their path,
- the taxonomy name is **not** slugified
- the taxonomy term (e.g. as specific tag) is slugified when `slugify.taxonomies` is enabled
  (`"on"` by default) in the configuration

The taxonomy pages are then available at the following paths:

```plain
{base_url}/{name}/        (taxonomy)
{base_url}/{name}/{term}  (taxonomy entry)
```
Note that taxonomies are case insensitive so terms that have the same slug will get merged.
For example, sections and pages with the tag "example" will be shown in the same taxonomy page
as those with "Example".
