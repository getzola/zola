+++
title = "RSS"
weight = 50
+++

Gutenberg will look for a `rss.xml` file in the `templates` directory or 
use the built-in one. Currently it is only possible to have one RSS feed for the whole
site, you cannot create a RSS feed per section or taxonomy.

**Only pages with a date and that are not draft will be available.**

The RSS template gets two variables in addition of the config:

- `last_build_date`: the date of the latest post
- `pages`: see [the page variables](./documentation/templates/pages-sections.md#page-variables) for
a detailed description of this variable.
