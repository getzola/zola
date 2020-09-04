+++
title = "Taxonomies"
weight = 90
+++

Zola has built-in support for taxonomies.

## Configuration

A taxonomy has five variables:

- `name`: a required string that will be used in the URLs, usually the plural version (i.e., tags, categories, etc.)
- `paginate_by`: if this is set to a number, each term page will be paginated by this much.
- `paginate_path`: if set, this path will be used by the paginated page and the page number will be appended after it.
For example the default would be page/1.
- `feed`: if set to `true`, a feed (atom by default) will be generated for each term.
- `lang`: only set this if you are making a multilingual site and want to indicate which language this taxonomy is for

Insert into the configuration file (config.toml):

⚠️ Place the taxonomies key in the main section and not in the `[extra]` section

**Example 1:** (one language)

```toml
taxonomies = [ name = "categories", rss = true ]
```

**Example 2:** (multilingual site)

```toml
taxonomies = [
    {name = "tags", lang = "fr"},
    {name = "tags", lang = "eo"},
    {name = "tags", lang = "en"},
]
```

## Using taxonomies

Once the configuration is done, you can then set taxonomies in your content and Zola will pick them up:

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

In a similar manner to how section and pages calculate their output path:
- the taxonomy name is never slugified
- the taxonomy term (e.g. as specific tag) is slugified when `slugify.taxonomies` is enabled (`"on"`, the default) in the configuration

The taxonomy pages are then available at the following paths:

```plain
$BASE_URL/$NAME/ (taxonomy)
$BASE_URL/$NAME/$SLUG (taxonomy entry)
```
Note that taxonomies are case insensitive so terms that have the same slug will get merged, e.g. sections and pages containing the tag "example" will be shown in the same taxonomy page as ones containing "Example" 
