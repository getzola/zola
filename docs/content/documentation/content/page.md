+++
title = "Page"
weight = 30
+++

A page is any file ending with `.md` in the `content` directory, except files
named `_index.md`.

If a file ending with `.md` is named `index.md`, then it will generate a page
with the name of the containing folder (for example, `/content/about/index.md` would
create a page at `[base_url]/about`).  (Note the lack of an underscore; if the file
were named `_index.md`, then it would create a **section** at `[base_url]/about`, as
discussed in the prior part of this documentation.  But naming the file `index.md` will
create a **page** at `[base_url]/about`).

If the file is given any name *other* than `index.md` or `_index.md`, then it will
create a page with that name (without the `.md`). So naming a file in the root of your
content directory `about.md` would also create a page at `[base_url]/about`.
Another exception to that rule is that a filename starting with a datetime (YYYY-mm-dd or [a RFC3339 datetime](https://www.ietf.org/rfc/rfc3339.txt)) followed by
an underscore (`_`) or a dash (`-`) will use that date as the page date, unless already set
in the front-matter. The page name will be anything after `_`/`-` so a filename like `2018-10-10-hello-world.md` will
be available at `[base_url]/hello-world`. Note that the full RFC3339 datetime contains colons, which is not a valid
character in a filename on Windows.

As you can see, creating an `about.md` file is exactly equivalent to creating an
`about/index.md` file.  The only difference between the two methods is that creating
the `about` folder allows you to use asset colocation, as discussed in the
[Overview](@/documentation/content/overview.md#assets-colocation) section of this documentation.

## Front-matter

The front-matter is a set of metadata embedded in a file. In Zola,
it is at the beginning of the file, surrounded by `+++` and uses TOML.

While none of the front-matter variables are mandatory, the opening and closing `+++` are required.

Here is an example page with all the variables available.  The values provided below are the default
values.

```md
+++
title = ""
description = ""

# The date of the post.
# 2 formats are allowed: YYYY-MM-DD (2012-10-02) and RFC3339 (2002-10-02T15:00:00Z)
# Do not wrap dates in quotes, the line below only indicates that there is no default date.
# If the section variable `sort_by` is set to `date`, then any page that lacks a `date`
# will not be rendered.
# Setting this overrides a date set in the filename.
date =

# The weight as defined in the Section page
# If the section variable `sort_by` is set to `weight`, then any page that lacks a `weight`
# will not be rendered.
weight = 0

# A draft page will not be present in prev/next pagination
draft = false

# If filled, it will use that slug instead of the filename to make up the URL
# It will still use the section path though
slug = ""

# The path the content will appear at
# If set, it cannot be an empty string and will override both `slug` and the filename.
# The sections' path won't be used.
# It should not start with a `/` and the slash will be removed if it does
path = ""

# Use aliases if you are moving content but want to redirect previous URLs to the
# current one. This takes an array of path, not URLs.
aliases = []

# Whether the page should be in the search index. This is only used if
# `build_search_index` is set to true in the config and the parent section
# hasn't set `in_search_index` to false in its front-matter
in_search_index = true

# Template to use to render this page
template = "page.html"

# The taxonomies for that page. The keys need to be the same as the taxonomies
# name configured in `config.toml` and the values an array of String like
# tags = ["rust", "web"]
[taxonomies]

# Your own data
[extra]
+++

Some content
```

## Summary

You can ask Zola to create a summary if you only want to show the first
paragraph of each page in a list for example.

To do so, add <code>&lt;!-- more --&gt;</code> in your content at the point
where you want the summary to end and the content up to that point will be also
available separately in the
[template](@/documentation/templates/pages-sections.md#page-variables).

An anchor link to this position named `continue-reading` is created, wrapped in a paragraph
with a `zola-continue-reading` id, so you can link directly to it if needed for example:
`<a href="{{ page.permalink }}#continue-reading">Continue Reading</a>`
