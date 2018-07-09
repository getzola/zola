+++
title = "Taxonomies"
weight = 90
+++

Gutenberg has built-in support for taxonomies.

The first step is to define the taxonomies in your [config.toml](./documentation/getting-started/configuration.md).

A taxonomy has 3 variables:

- `name`: a required string that will be used in the URLs, usually the plural version (i.e. tags, categories etc)
- `paginate`: if this is set to a number, each term page will be paginated by this much.
- `rss`: if set to `true`, a RSS feed will be generated for each individual term.

Once this is done, you can then set taxonomies in your content and Gutenberg will pick
them up.

The taxonomy pages will only be created if at least one non-draft page is found and
are available at the following paths:

```plain
$BASE_URL/$NAME/
$BASE_URL/$NAME/$SLUG
```
