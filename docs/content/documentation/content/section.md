+++
title = "Section"
weight = 20
+++

A section is created whenever a folder (or subfolder) in the `content` section contains an 
`_index.md` file.  If a folder does not contain an `_index.md` file, no section will be 
created, but markdown files within that folder will still create pages (known as orphan pages).

The index page (i.e., the page displayed when a user browses to your `base_url`) is a section, 
which is created whether or not you add an `_index.md` file at the root of your `content` folder.
If you do not create an `_index.md` file in your content directory, this main content section will
not have any content or metadata.  If you would like to add content or metadata, you can add an
`_index.md` file at the root of the `content` folder and edit it just as you would edit any other
`_index.md` file; your `index.html` template will then have access to that content and metadata.

## Front-matter

The `_index.md` file within a folder defines the content and metadata for that section.  To set
the metadata, add front matter to the file.

The front-matter is a set of metadata embedded in a file. In Gutenberg,
it is at the beginning of the file, surrounded by `+++` and uses TOML.

After the closing `+++`, you can add content that will be parsed as markdown and will be available
to your templates through the `section.content` variable.

While none of the front-matter variables are mandatory, the opening and closing `+++` are required.

Here is an example `_index.md` with all the variables available:


```md
+++
title = ""

description = ""

# Whether to sort by "date", "order", "weight" or "none". More on that below
sort_by = "none"

# Used by the parent section to order its subsections.
# Lower values have priority.
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

# Whether the section pages should be in the search index. This is only used if
# `build_search_index` is set to true in the config
in_search_index = true

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

You can also change the pagination path (the word displayed while paginated in the URL, like `page/1`)
by setting the `paginate_path` variable, which defaults to `page`.

## Sorting
Sections' pages can be sorted three different ways, by `date`, by `weight`,
and by `order`.  This value will alter the way templates iterate through 
those pages using Tera's loops.  See
[iterating and sorting](./documentation/templates/iterating-and-sorting.md)
for details.  To specify the sorting method, set the
`sort_by` front-matter variable in the `_index.md` file for the section.  If 
no `sort_by` method is set, the pages will be sorted in a default order that 
is not guaranteed to correspond to any of the explicit orders.

Any page that is missing the data it needs to be sorted will be ignored and
won't be rendered. For example, if a page is missing the date variable the 
containing section sets `sort_by = "date"`, then that page will be ignored.  The terminal will warn you if this is happening.

If several pages have the same date/weight/order, their permalink will be used to break the tie following an alphabetical order.
