+++
title = "RSS"
weight = 50
+++

If the site `config.toml` file sets `generate_rss = true`, then Zola will
generate an `rss.xml` page for the site, which will live at `base_url/rss.xml`. To
generate the `rss.xml` page, Zola will look for a `rss.xml` file in the `templates`
directory or, if one does not exist, will use the use the built-in rss template.
Currently it is only possible to have one RSS feed for the whole site; you cannot
create a RSS feed per section or taxonomy.

**Only pages with a date and that are not draft will be available.**

The RSS template gets two variables in addition of the config:

- `last_build_date`: the date of the latest post
- `pages`: see [the page variables](./documentation/templates/pages-sections.md#page-variables) for
a detailed description of what this contains
