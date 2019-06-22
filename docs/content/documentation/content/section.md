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

Any non-Markdown file in the section folder is added to the `assets` collection of the section, as explained in the [Content Overview](@/documentation/content/overview.md#assets-colocation). These files are then available from the Markdown using relative links.

## Front-matter

The `_index.md` file within a folder defines the content and metadata for that section.  To set
the metadata, add front matter to the file.

The front-matter is a set of metadata embedded in a file. In Zola,
it is at the beginning of the file, surrounded by `+++` and uses TOML.

After the closing `+++`, you can add content that will be parsed as markdown and will be available
to your templates through the `section.content` variable.

While none of the front-matter variables are mandatory, the opening and closing `+++` are required.

Here is an example `_index.md` with all the variables available.  The values provided below are the
default values.


```md
+++
title = ""

description = ""

# Whether to sort pages by "date", "weight", or "none". More on that below
sort_by = "none"

# Used by the parent section to order its subsections.
# Lower values have priority.
weight = 0

# Template to use to render this section page
template = "section.html"

# Apply the given template to ALL pages below the section, recursively.
# If you have several nested sections each with a page_template set, the page
# will always use the closest to itself.
# However, a page own `template` variable will always have priority.
# Not set by default
page_template =

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

# Whether to redirect when landing on that section. Defaults to not being set.
# Useful for the same reason as `render` but when you don't want a 404 when
# landing on the root section page.
# Example: redirect_to = "documentation/content/overview"
redirect_to = ""

# Whether the section should pass its pages on to the parent section. Defaults to `false`.
# Useful when the section shouldn't split up the parent section, like
# sections for each year under a posts section.
transparent = false

# Use aliases if you are moving content but want to redirect previous URLs to the
# current one. This takes an array of path, not URLs.
aliases = []

# Your own data
[extra]
+++

Some content
```

Keep in mind that any configuration apply only to the direct pages, not to the subsections' pages.

## Pagination

To enable pagination for a section's pages, simply set `paginate_by` to a positive number and it will automatically
paginate by this much. See [pagination template documentation](@/documentation/templates/pagination.md) for more information
on what will be available in the template.

You can also change the pagination path (the word displayed while paginated in the URL, like `page/1`)
by setting the `paginate_path` variable, which defaults to `page`.

## Sorting
It is very common for Zola templates to iterate over pages or sections
to display all pages/sections a given directory.  Consider a very simple
example: a `blog` directory with three files: `blog/Post_1.md`,
`blog/Post_2.md`, and `blog/Post_3.md`.  To iterate over these posts and
create a list of links to the posts, a simple template might look like this:

```j2
{% for post in section.pages %}
  <h1><a href="{{ post.permalink }}">{{ post.title }}</a></h1>
{% endfor %}
```

This would iterate over the posts, and would do so in a specific order
based on the `sort_by` variable set in the `_index.md` page for the
containing section.  The `sort_by` variable can be given three values: `date`,
`weight`, and `none`.  If no `sort_by` method is set, the pages will be
sorted in the `none` order, which is not intended to be used for sorted content.

Any page that is missing the data it needs to be sorted will be ignored and
won't be rendered. For example, if a page is missing the date variable the
containing section sets `sort_by = "date"`, then that page will be ignored.
The terminal will warn you if this is happening.

If several pages have the same date/weight/order, their permalink will be used
to break the tie following an alphabetical order.

## Sorting Pages
The `sort_by` front-matter variable can have the following values:

### `date`
This will sort all pages by their `date` field, from the most recent (at the
top of the list) to the oldest (at the bottom of the list).  Each page will
get `page.earlier` and `page.later` variables that contain the pages with
earlier and later dates, respectively.

### `weight`
This will be sort all pages by their `weight` field, from lightest weight
(at the top of the list) to heaviest (at the bottom of the list).  Each
page gets `page.lighter` and `page.heavier` variables that contain the
pages with lighter and heavier weights, respectively.

When iterating through pages, you may wish to use the Tera `reverse` filter,
which reverses the order of the pages.  Thus, after using the `reverse` filter,
pages sorted by weight will be sorted from lightest (at the top) to heaviest
(at the bottom); pages sorted by date will be sorted from oldest (at the top)
to newest (at the bottom).

`reverse` has no effect on `page.later`/`page.earlier`/`page.heavier`/`page.lighter`.

## Sorting Subsections
Sorting sections is a bit less flexible: sections are always sorted by `weight`,
and do not have any variables that point to the next heavier/lighter sections.

Based on this, by default the lightest (lowest `weight`) subsections will be at
the top of the list and the heaviest (highest `weight`) will be at the bottom;
the `reverse` filter reverses this order.

**Note**: Unlike pages, permalinks will **not** be used to break ties between
equally weighted sections.  Thus, if the `weight` variable for your section is not set (or if it
is set in a way that produces ties), then your sections will be sorted in
**random** order. Moreover, that order is determined at build time and will
change with each site rebuild.  Thus, if there is any chance that you will
iterate over your sections, you should always assign them weight.
