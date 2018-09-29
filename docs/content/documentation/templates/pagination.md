+++
title = "Pagination"
weight = 30
+++

Two things can get paginated: a section or a taxonomy term.

A paginated section gets the same `section` variable as a normal
[section page](./documentation/templates/pages-sections.md#section-variables) minus its pages
while a paginated taxonomy gets the a `taxonomy` variable of type `TaxonomyConfig`, equivalent
to the taxonomy definition in the `config.toml`.

In addition, a paginated page gets a `paginator` variable of the `Pager` type:

```ts
// How many items per page
paginate_by: Number;
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
// All pagers for this section, but with their `pages` attribute set to an empty array
pagers: Array<Pagers>;
// Which page are we on
current_index: Number;
```
