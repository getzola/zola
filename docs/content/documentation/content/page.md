+++
title = "Page"
weight = 30
+++

A page is any file ending with `.md` in the `content` directory, except for `_index.md`
[section definitions](@/documentation/content/section.md).

If a file is named `index.md`, it will generate a page with the name of its containing directory.
This is equivalent to creating a `.md` file with the same name as the parent directory, but also
allows you to use asset co-location, as discussed in the
[overview](@/documentation/content/overview.md#asset-colocation) section.
For example, both `content/about.md` and `content/about/index.md` produce a page at `{base_url}/about`.
This also means you can't have both of these files at the same time.

If the file is given any name *other* than `index.md` or `_index.md`, then it will
create a page with that name (without the `.md`). For example, naming a file in the root of your
content directory `about.md` will create a page at `{base_url}/about`. Filenames containing
special characters are optionally sanitized as described [below](#output-paths). Filenames should
**not** contain dots, because the part after the last dot will be treated a language name, and
so will be removed.

Filenames beginning with a date followed by an underscore (`_`) or dash (`-`), or ending with a
dot (`.`) followed by a language name are treated specially. These tell information to Zola
about the page, similarly to the front matter, and are stripped from the output. See
[below](#path-from-filename) how to use these.

## Output paths

For any page within your content folder, its output path will be defined by either
- its `slug` frontmatter key if set, or
- its filename

Either way, these proposed path will be sanitized before being used. See the
[configuration documentation](@/documentation/getting-started/configuration.md#slugification-strategies)
for how it's done.

If you want URLs containing non-ASCII characters, `slugify.paths` needs to be set to `"safe"` or `"off"`.

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

This frontmatter will output the article to `{base_url}/zines/femmes-libres-libération-kurde` with
`slugify.paths` set to `"safe"` or `"off"`, and to `{base_url}/zines/femmes-libres-liberation-kurde`
with the default value for `slugify.paths` of `"on"`.

### Path from filename

When the article's output path is not specified in the frontmatter, it is extracted from the file's
path in the content folder.
Consider a file `content/foo/bar/thing.md`. The output path is constructed:
- if the filename is `index.md`, its parent folder name (`bar`) is used as output path
- otherwise, the output path is extracted from `thing` path (the filename without the `.md` extension)

If the path found starts with a datetime string (`YYYY-mm-dd` or [a RFC3339 datetime](https://www.ietf.org/rfc/rfc3339.txt))
followed by an underscore (`_`) or a dash (`-`), this date is removed from the output path and
will be used as the page date, unless already set in the front-matter. Note that the full RFC3339
datetime contains colons, which is not a valid character in a filename on Windows.

If a filename contains a dot (beside the one for `.md`), the part after the dot is removed, and will
be set as the page's language. This must match one of the `language_alias`es or language codes
set in the [configuration](@/documentation/content/multilingual.md#configuration).

The output path extracted from the file path is then optionally slugified, depending on the `slugify.paths`
config, as explained previously.

**Example:**
The file `content/blog/2018-10-10-hello-world.md` will yield a page at `{base_url}/blog/hello-world`.

## Front matter

The TOML front matter is a set of metadata embedded in a file at the beginning of the file enclosed
by triple pluses (`+++`).

Although none of the front matter variables are mandatory, the opening and closing `+++` are required.

Here is an example page with all the available variables. The values provided below are the
default values.

```toml
title = ""
description = ""

# The date of the post.
# Two formats are allowed: YYYY-MM-DD (2012-10-02) and RFC3339 (2002-10-02T15:00:00Z).
# Do not wrap dates in quotes; the line below only indicates that there is no default date.
# If the section variable `sort_by` is set to `date`, then any page that lacks a `date`
# will not be rendered.
# Setting this overrides a date set in the filename.
date =

# The last updated date of the post, if different from the date.
# Same format as `date`.
updated =

# The weight as defined on the Section page of the documentation.
# If the section variable `sort_by` is set to `weight`, then any page that lacks a `weight`
# will not be rendered.
weight = 0

# A draft page is only loaded if the `--drafts` flag is passed to `zola build`, `zola serve` or `zola check`.
draft = false

# If set, this slug will be instead of the filename to make the URL.
# The section path will still be used.
slug = ""

# The path the content will appear at.
# If set, it cannot be an empty string and will override both `slug` and the filename.
# The sections' path won't be used.
# It should not start with a `/` and the slash will be removed if it does.
path = ""

# Use aliases if you are moving content but want to redirect previous URLs to the
# current one. This takes an array of paths, not URLs.
aliases = []

# When set to "true", the page will be in the search index. This is only used if
# `build_search_index` is set to "true" in the Zola configuration and the parent section
# hasn't set `in_search_index` to "false" in its front matter.
in_search_index = true

# Template to use to render this page.
template = "page.html"

# The taxonomies for this page. The keys need to be the same as the taxonomy
# names configured in `config.toml` and the terms are an arry of Strings. For example,
# tags = ["rust", "web"].
[taxonomies]

# Your own data.
[extra]
```

## Summary

You can ask Zola to create a summary if, for example, you only want to show the first
paragraph of the page content in a list.

To do so, add <code>&lt;!-- more --&gt;</code> in your content at the point
where you want the summary to end. The content up to that point will be
available separately in the
[template](@/documentation/templates/pages-sections.md#page-variables).

A span element in this position with a `continue-reading` id is created, so you can link directly to it if needed. For example:
`<a href="{{ page.permalink }}#continue-reading">Continue Reading</a>`.
