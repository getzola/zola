+++
title = "Page"
weight = 30
+++

A page is any file ending with `.md` in the `content` directory, except files
named `_index.md`.

## Front-matter

The front-matter is a set of metadata embedded in a file. In Gutenberg,
it is at the beginning of the file, surrounded by `+++` and uses TOML.

While none of the front-matter variables are mandatory, the opening and closing `+++` are required.

Here is an example page with all the variables available:

```md
+++
title = ""
description = ""

# The date of the post.
# 2 formats are allowed: YYYY-MM-DD (2012-10-02) and RFC3339 (2002-10-02T15:00:00Z)
# Do not wrap dates in quotes, the line below only indicates that there is no default date
date = ""

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

# An array of strings allowing you to group pages with them
tags = []

# An overarching category name for that page, allowing you to group pages with it
category = ""

# The order as defined in the Section page
order = 0

# The weight as defined in the Section page
weight = 0

# Use aliases if you are moving content but want to redirect previous URLs to the 
# current one. This takes an array of path, not URLs.
aliases = []

# Whether the page should be in the search index. This is only used if
# `build_search_index` is set to true in the config and the parent section 
# hasn't set `in_search_index` to false in its front-matter
in_search_index = true

# Template to use to render this page
template = "page.html"

# Your own data
[extra]
+++

Some content
```

## Summary

You can ask Gutenberg to create a summary if you only want to show the first 
paragraph of each page in a list for example.

To do so, add `<!-- more -->` in your content at the point where you want the
summary to end and the content up to that point will be also available separately
in the [template](./documentation/templates/pages-sections.md#page-variables).
