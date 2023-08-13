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

In case you want to extend, or modify, the built-in templates, you can get a
copy from [the source code here](https://github.com/getzola/zola/tree/master/components/templates/src/builtins)
and place it in the `templates/` directory with the appropriate name. You can
check the documentation for the specifications for Atom 1.0 and RSS 2.0 in
[W3C Feed Validation Service](https://validator.w3.org/feed/docs/).

**Only pages with a date will be available.**

The author in the feed is set as
- The first author in `authors` set in the 
  [front matter](@/documentation/content/page.md#front-matter)
- If that is not present it falls back to the `author` in the 
  [Configuration](@/documentation/getting-started/configuration.md)
- If that is also not preset it is set to `Unknown`.

Note that `atom.xml` and `rss.xml` require different formats for specifying the
author. According to [RFC 4287][atom_rfc] `atom.xml` requires the author's
name, for example `"John Doe"`. While according to the 
[RSS 2.0 Specification][rss_spec] the email address is required, and the name
optionally included, for example `"lawyer@boyer.net"` or 
`"lawyer@boyer.net (Lawyer Boyer)"`.

The feed template gets five variables:

- `config`: the site config
- `feed_url`: the full url to that specific feed
- `last_updated`: the most recent `updated` or `date` field of any post
- `pages`: see [page variables](@/documentation/templates/pages-sections.md#page-variables)
  for a detailed description of what this contains
- `lang`: the language code that applies to all of the pages in the feed,
  if the site is multilingual, or `config.default_language` if it is not

Feeds for taxonomy terms get two more variables, using types from the
[taxonomies templates](@/documentation/templates/taxonomies.md):

- `taxonomy`: of type `TaxonomyConfig`
- `term`: of type `TaxonomyTerm`, but without `term.pages` (use `pages` instead)

You can also enable separate feeds for each section by setting the
`generate_feed` variable to true in the respective section's front matter.
Section feeds will use the same template as indicated in the `config.toml` file.
Section feeds, in addition to the five feed template variables, get the
`section` variable from the [section
template](@/documentation/templates/pages-sections.md).

Enable feed autodiscovery allows feed readers and browsers to notify user about a RSS or Atom feed available on your web site. So it is easier for user to subscribe.
As an example this is how it looks like using [Firefox](https://en.wikipedia.org/wiki/Mozilla_Firefox) [Livemarks](https://addons.mozilla.org/en-US/firefox/addon/livemarks/?src=search) addon.

![RSS feed autodiscovery example.](rss_feed.png)

You can enable posts autodiscovery modifying your blog `base.html` template adding the following code in between the `<head>` tags.
```html
{% block rss %}
  <link rel="alternate" type="application/rss+xml" title="RSS" href="{{/* get_url(path="rss.xml", trailing_slash=false) */}}">
{% endblock %}
```
You can as well use an Atom feed using `type="application/atom+xml"` and `path="atom.xml"`.

All pages on your site will refer to your post feed.

In order to enable the tag feeds as well, you can overload the `block rss` using the following code in your `tags/single.html` template.
```html
{% block rss %}
  {% set rss_path = "tags/" ~ term.name ~ "/rss.xml" %}
  <link rel="alternate" type="application/rss+xml" title="RSS" href="{{/* get_url(path=rss_path, trailing_slash=false) */}}">
{% endblock rss %}
```
Each tag page will refer to it's dedicated feed.

[atom_rfc]: https://www.rfc-editor.org/rfc/rfc4287
[rss_spec]: https://www.rssboard.org/rss-specification#ltauthorgtSubelementOfLtitemgt