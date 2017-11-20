+++
title = "Tags & Categories"
weight = 90
+++

Gutenberg has built-in support for basic taxonomies: tags and categories.

Those taxonomies are automatically built across the whole site based on
the `tags` and `category` fields of the front-matter: you do not need to define
that a tag or a category exists. You have to set `generate_tags_pages` and/or 
`generate_categories_pages` in your [config.toml](./documentation/getting-started/configuration.md).

The taxonomy pages will only be created if at least one item is found and
are available at the following paths:

```plain
$BASE_URL/tags/
$BASE_URL/tags/$TAG_SLUG
$BASE_URL/categories/
$BASE_URL/categories/$CATEGORY_SLUG
```

It is currently not possible to change those paths or to create custom taxonomies.
