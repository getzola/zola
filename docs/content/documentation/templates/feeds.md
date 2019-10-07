+++
title = "Feeds"
weight = 50
aliases = ["/documentation/templates/rss/"]
+++

If the site `config.toml` file sets `generate_feed = true`, then Zola will
generate a feed file for the site, named according to the `feed_filename`
setting in `config.toml`, which defaults to `atom.xml`. Given the feed filename
`atom.xml`, the generated file will live at `base_url/atom.xml`, based upon the
`atom.xml` file in the `templates` directory, or the built-in Atom template.

`feed_filename` can be set to any value, but built-in templates are provided
for `atom.xml` (in the preferred Atom 1.0 format), and `rss.xml` (in the RSS
2.0 format). If you choose a different filename (e.g. `feed.xml`), you will
need to provide a template yourself.

**Only pages with a date will be available.**

The feed template gets three variables in addition to `config`:

- `feed_url`: the full url to that specific feed
- `last_updated`: the most recent `updated` or `date` field of any post
- `pages`: see [page variables](@/documentation/templates/pages-sections.md#page-variables) for
a detailed description of what this contains
