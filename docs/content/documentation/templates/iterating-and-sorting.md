+++
title = "Iterating and Sorting"
weight = 25
+++

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

This would iterate over the posts, and would do so in a specific order.

## Pages
Since all the posts are pages, the for loop will iterate over them based on
the `sort_by` order defined in the `_index.md` file for the containing 
section (that is, in `blog/_index.md`).  Additionally, based on the `sort_by`
variable, the page variables `page.next` and `page.previous` will be set.
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

## Sections
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
