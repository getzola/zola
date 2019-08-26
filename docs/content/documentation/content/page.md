+++
title = "Page"
weight = 30
+++

A page is any file ending with `.md` in the `content` directory, except files named `_index.md` because these produce sections and not pages.

Creating an `about.md` file is exactly equivalent to creating an `about/index.md` file.  The only difference between the two methods is that creating the `about` folder allows you to use asset colocation, as discussed in the [Overview](@/documentation/content/overview.md#assets-colocation) section of this documentation.

## Output paths

For any page within your content folder, its output path will be defined by either:

- its `slug` frontmatter key
- its filename

Either way, these proposed path will be sanitized before being used. If `slugify_paths` is enabled in the site's config, paths are [slugified](https://docs.rs/slug/0.1.4/slug/fn.slugify.html). Otherwise, a simpler sanitation is performed: the characters `/` and `#` are stripped from the proposed path.

**NOTE:** To produce URLs containing non-English characters (UTF8), `slugify_paths` needs to remain disabled.

### Path from frontmatter

The output path for the page will first be read from the `slug` key in the page's frontmatter.

**Example:** (file `content/zines/mlf-kurdistan.md`)

```
+++
title = "Le mouvement des Femmes Libres, à la tête de la libération kurde"
slug = "femmes-libres-libération-kurde"
+++
This is my article.
```

This frontmatter will output the article to `[base_url]/zines/femmes-libres-libération-kurde` with `slugify` disabled, and to `[base_url]/zines/femmes-libres-liberation-kurde` with `slugify` enabled.

### Path from filename

When the article's output path is not specified in the frontmatter, it is extracted from the file's path in the content folder. Consider a file `content/foo/bar/thing.md`. The output path is constructed:
- if the filename is `index.md`, its parent folder name (`bar`) is used as output path
- otherwise, the output path is extracted from `thing` (the filename without the `.md` extension)

If the path found starts with a datetime string (`YYYY-mm-dd` or [a RFC3339 datetime](https://www.ietf.org/rfc/rfc3339.txt)) followed by an underscore (`_`) or a dash (`-`), this date is removed from the output path and will be used as the page date (unless already set in the front-matter). Note that the full RFC3339 datetime contains colons, which is not a valid character in a filename on Windows.

The output path extracted from the file path is then slugified or not depending on the `slugify` config, as explained previously.

**Example:** The file `content/blog/2018-10-10-hello-world.md` will generated a page available at will be available at `[base_url]/hello-world`.

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

# A draft page is only rendered in `zola serve`, they are ignored in `zola build` and `zola check`
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
