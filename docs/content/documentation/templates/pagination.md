+++
title = "Pagination"
weight = 30
+++

Two things can get paginated: a section and a taxonomy term.

A paginated section gets the same `section` variable as a normal
[section page](./documentation/templates/pages-sections.md#section-variables) minus its pages
while both a paginated taxonomy page and a paginated section page gets a
`paginator` variable of the `Pager` type:

```ts
// How many items per page
paginate_by: Number;
// The base URL for the pagination: section permalink + pagination path
// You can concatenate an integer with that to get a link to a given pagination page.
base_url: String;
// How many pagers in this paginator
number_pagers: Number;
// Permalink to the first page
first: String;
// Permalink to the last page
last: String;
// Permalink to the previous page, if there is one
previous: String?;
// Permalink to the next page, if there is one
next: String?;
// All pages for the current page
pages: Array<Page>;
// Which page are we on
current_index: Number;
```

## Section

A paginated section gets the same `section` variable as a normal
[section page](./documentation/templates/pages-sections.md#section-variables)
minus its pages. The pages are instead in `paginator.pages`.

## Taxonomy term

A paginated taxonomy gets two variables aside from the `paginator` variable:

- a `taxonomy` variable of type `TaxonomyConfig`
- a `term` variable of type `TaxonomyTerm`.

See the [taxonomies page](@/documentation/templates/taxonomies.md) for a detailed version of the types.
