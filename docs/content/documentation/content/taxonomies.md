+++
title = "Taxonomies"
weight = 90
+++

Gutenberg has built-in support for taxonomies.

The first step is to define the taxonomies in your [config.toml](./documentation/getting-started/configuration.md).

A taxonomy has 4 variables:

- `name`: a required string that will be used in the URLs, usually the plural version (i.e. tags, categories etc)
- `paginate`: if this is set to a number, each term page will be paginated by this much.
- `paginate_path`: if set, will be the path used by paginated page and the page number will be appended after it.
For example the default would be page/1
- `rss`: if set to `true`, a RSS feed will be generated for each individual term.

Once this is done, you can then set taxonomies in your content and Gutenberg will pick
them up.

The taxonomy pages will only be created if at least one non-draft page is found and
are available at the following paths:

```plain
$BASE_URL/$NAME/
$BASE_URL/$NAME/$SLUG
```
