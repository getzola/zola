+++
title = "Tags & Categories"
weight = 40
+++

Tags and categories actually get the same data but with different variable names.
The default templates for those pages are the following:

- `tags.html`: list of tags, gets variable `tags`
- `tag.html`: individual tag, gets variable `tag`
- `categories.html`: list of categories, gets variable `categories`
- `category.html`: individual category, gets variable `category`

You can override any of those templates by putting one with the same name in the `templates` directory.
`tags` and `categories` both are an array of `TaxonomyItem` sorted alphabetically, while `tag` and `category` 
are a `TaxonomyItem`.

A `TaxonomyItem` has the following fields:

```ts
name: String;
slug: String;
// Permalink to the generated page
permalink: String;
pages: Array<Page>;
```

Currently, there is no way to define different taxonomy templates per section, change
the path used for them or paginate them.

