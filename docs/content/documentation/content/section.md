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
It is very common for Gutenberg templates to iterate over pages or sections
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
`weight`, and `order`.  If no `sort_by` method is set, the pages will be
sorted in a default order that is not guaranteed to correspond to any of the
explicit orders.  The value of `sort_by` will also determine which pages
are listed stored in the `page.next` and `page.previous` variables.  The effects of these values is explained below.

Any page that is missing the data it needs to be sorted will be ignored and
won't be rendered. For example, if a page is missing the date variable the 
containing section sets `sort_by = "date"`, then that page will be ignored.  The terminal will warn you if this is happening.

If several pages have the same date/weight/order, their permalink will be used to break the tie following an alphabetical order.

## Sorting Pages
The `sort_by` front-matter variable can have the following values:

### `date`
This will sort all pages by their `date` field, from the most recent (at the
top of the list) to the oldest (at the bottom of the list).  Each page will
get a `page.next` variable that points *down* the list (to the page just 
older than the current page) and a `page.previous` variable that points up
the list (to the just newer page).

### `weight`
This will be sort all pages by their `weight` field, from lightest weight 
(at the top of the list) to heaviest (at the bottom of the list).  Each 
page gets a `page.next` variable that points *up* the list (to the page that
is just lighter than the current page) and a `page.previous` variable that 
points down the list (to the page that is just heavier than the current page).

### `order`
This will be sort all pages by their `order` field. Order is the opposite of weight; think of it as listing the order in which pages were posted, with the 
oldest (first) at the bottom of the list. Each page also gets a 
`page.next` variable that points *up* the list (to the page with a higher order
than the current page) and a `page.previous` variable that points down the list
(to the page just lower in order).

To make this a bit more concrete, let's play out the simple example raised 
above.  Imagine that we set the `weight` and `order` both to 1 in `Post_1`,
both to 2 in `Post_2` and both to 3 in `Post_3`.  (In practice, there would
typically be no reason to set *both* `order` and `weight`). 

If we then set `sort_by = "weight"` in the `blog/_index.md` file, we would
get the following order from a Tera for loop:

 *  Page_1 [`page.next = null`, `page.previous = Page_2`]
 *  Page_2 [`page.next = Page_1`, `page.previous = Page_2`]
 *  Page_3 [`page.next = Page_2`, `page.previous = Page_2`]

If, however, we set the `sort_by` front-matter variable to `order`, we 
would get:
 *  Page_3 [`page.next = null`, `page.previous = Page_2`]
 *  Page_2 [`page.next = Page_3`, `page.previous = Page_1`]
 *  Page_1 [`page.next = Page_2`, `page.previous = null`]

Note that the order is reversed but in *both* cases the `page.previous` is
pointing *up* the list, and `page.next` is pointing *down* the list.  This 
fits many common use cases, including when Gutenberg is used for a blog as
in this simple example.

However, Gutenberg is flexible enough to accommodate alternate use cases as
well.  If you would prefer the `page.next` and `page.previous` variables 
to point in the opposite direction, you can use Tera's `reverse` filter. 
`reverse` causes the order to be reversed but does *not* alter the behaviour 
of `next` and `previous`.  Thus, combining `sort_by = "weight"` with `reverse`
gives you the same sequence as using `sort_by = "order"` but with `next` 
and `previous` pointing in the other direction.  By combining `sort_by` and
`reverse`, you can achieve any combination of sorting order and
`next`/`previous` values.

## Sorting Subsections
Sorting sections is a bit less flexible but also much simpler.  This is 
because sections do not have `next` or `previous` values.  Further, they can
only be sorted by `weight`—thus, the `sort_by` value in the containing section
has no impact at all on any subsections (only on pages).

Based on this, by default the lightest (lowest `weight`) subsections will be at
the top of the list and the heaviest (highest `weight`) will be at the top;
the `reverse` filter reverses this order.

**Note**: If the `weight` variable for your section is not set (or if it 
is set in a way that produces ties), then your sections will be sorted in 
**random** order. Moreover, that order is determined at build time and will
change with each site rebuild.  Thus, if there is any chance that you will 
iterate over your sections, you should always assign them weight.
