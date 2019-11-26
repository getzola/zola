+++
title = "RSS"
weight = 50
+++

If the site `config.toml` file sets `generate_rss = true`, then Zola will
generate an `rss.xml` page for the site, which will live at `base_url/rss.xml`. To
generate the `rss.xml` page, Zola will look for an `rss.xml` file in the `templates`
directory or, if one does not exist, it will use the use the built-in rss template.

**Only pages with a date will be available.**

The RSS template gets three variables in addition to `config`:

- `feed_url`: the full url to that specific feed
- `last_build_date`: the date of the latest post
- `pages`: see [page variables](@/documentation/templates/pages-sections.md#page-variables) for
a detailed description of what this contains
