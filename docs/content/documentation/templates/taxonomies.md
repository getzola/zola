+++
title = "Taxonomies"
weight = 40
+++

Zola will look up the following files in the `templates` directory:

- `$TAXONOMY_NAME/single.html`
- `$TAXONOMY_NAME/list.html`

First, a `TaxonomyTerm` has the following fields:

```ts
name: String;
slug: String;
permalink: String;
pages: Array<Page>;
```

## Non-paginated taxonomies
If a taxonomy is not paginated, the templates get the following variables:

### Single term (`single.html`)
```ts
// The site config
config: Config;
// The data of the taxonomy, from the config
taxonomy: TaxonomyConfig;
// The current full permalink for that page
current_url: String;
// The current path for that page
current_path: String;
// The current term being rendered
term: TaxonomyTerm;
```

### Taxonomy list (`list.html`)
```ts
// The site config
config: Config;
// The data of the taxonomy, from the config
taxonomy: TaxonomyConfig;
// The current full permalink for that page
current_url: String;
// The current path for that page
current_path: String;
// All terms for that taxonomy
terms: Array<TaxonomyTerm>;
```

## Paginated taxonomies
