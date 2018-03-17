+++
title = "Section"
weight = 20
+++

A section is automatically created when a folder is found 
in the `content` section, unless it only contains a `index.md` file and is actually
a page with assets.

You can add `_index.md` file to a folder to augment a section and give it
some metadata and/or content.

The index page is actually a section created automatically like any other: you can add metadata
and content by adding `_index.md` at the root of the `content` folder.

## Front-matter

The front-matter is a set of metadata embedded in a file. In Gutenberg,
it is at the beginning of the file, surrounded by `+++` and uses TOML.

While none of the front-matter variables are mandatory, the the opening and closing `+++` are required.

Here is an example `_index.md` with all the variables available:


```md
+++
title = ""

description = ""

# Whether to sort by "date", "order", "weight" or "none". More on that below
sort_by = "none"

# Used by the parent section to order its subsections.
# Higher values means it will be at the end.
weight = 0

# Template to use to render this section page
template = "section.html"

# How many pages to be displayed per paginated page. 
# No pagination will happen if this isn't set or if the value is 0
paginate_by = 0

# If set, will be the path used by paginated page and the page number will be appended after it. 
# For example the default would be page/1
paginate_path = "page"

# Whether to insert a link for each header like the ones you can see in this site if you hover one
# The default template can be overridden by creating a `anchor-link.html` in the `templates` directory
# Options are "left", "right" and "none"
insert_anchor_links = "none"

# Whether to render that section homepage or not. 
# Useful when the section is only there to organize things but is not meant
# to be used directly
render = true

# Whether to redirect when landing on that section. Defaults to `None`.
# Useful for the same reason as `render` but when you don't want a 404 when
# landing on the root section page
redirect_to = ""

# Your own data
[extra]
+++

Some content
```

Keep in mind that any configuration apply only to the direct pages, not to the subsections' pages.

## Pagination

To enable pagination for a section's pages, simply set `paginate_by` to a positive number and it will automatically
paginate by this much. See [pagination template documentation](./documentation/templates/pagination.md) for more information
on what will be available in the template.

You can also change the pagination path - the word displayed while paginated in the URL, like `page/1` - 
by setting the `paginate_path` variable, which defaults to `page`.

## Sorting
Sections' pages can be sorted three different ways, not counting the unsorted default and 
is enabled by setting the `sort_by` front-matter variable.

Any page that cannot be sorted, for example if missing the date variable while sorting by `date`, will be ignored and
won't be rendered. The terminal will warn you if this is happening.

### `date`
This will sort all pages by their `date` field, from the most recent to the oldest.

### `weight`
This will be sort all pages by their `weight` field. Heavier weights fall at the bottom: 5 would be before 10.

### `order`
This will be sort all pages by their `order` field. Order is the opposite of weight, think of it as enumerating 
the content: this is my first post, my second, etc. A page with `order: 5` will appear after a page with `order: 10` in the sorted list.

