+++
title = "Pagination"
weight = 30
+++

A paginated section gets the same `section` variable as a normal
[section page](./documentation/templates/pages-sections.md#section-variables).
In addition, a paginated section gets a `paginator` variable of the `Pager` type:

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
