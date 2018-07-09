+++
title = "Taxonomies"
weight = 40
+++

The default templates for the taxonomies pages are the following:

- `$TAXONOMY_NAME/single.html`: gets a `terms` variable
- `$TAXONOMY_NAME/list.html`: individual `term`

You can override any of those templates by putting one with the same path in the `templates` directory.
`terms` is an array of `TaxonomyItem` sorted alphabetically, while `term` is a single `TaxonomyItem`.

A `TaxonomyItem` has the following fields:

```ts
name: String;
slug: String;
permalink: String;
pages: Array<Page>;
```

As `pages` can span many sections, the `pages` array is sorted by date.
