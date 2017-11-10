+++
title = "Overview"
weight = 10
+++

Gutenberg uses the [Tera](https://tera.netlify.com) template engine and is very similar
to Jinja2, Liquid or Twig.

As this documentation will only talk about how templates work in Gutenberg, please read
the [Tera template documentation](https://tera.netlify.com/docs/templates/) if you want
to learn more about it first.

All templates live in the `templates` directory and built-in or themes templates can
be overriden by creating a template with same name in the correct path. For example,
you can override the RSS template by creating a `templates/rss.xml` file.

If you are not sure what variables are available in a template, you can just stick `{{ __tera_context }}` in it
to print the whole context.

A few variables are available on all templates minus RSS and sitemap:

- `config`: the [configuration](./documentation/getting-started/configuration.md) without any modifications
- `current_path`: the path (full URL without the `base_url`) of the current page, never starting with a `/`
- `current_url`: the full URL for that page

## Built-in filters
Gutenberg adds a few filters, in addition of the ones already present in Tera.

### markdown
Converts the given variable to HTML using Markdown. This doesn't apply any of the
features that Gutenberg adds to Markdown: internal links, shortcodes etc won't work.

### base64_encode
Encode the variable to base64.

### base64_decode
Decode the variable from base64.


## Built-in global functions
Gutenberg adds a few global functions to Tera in order to make it easier to develop complex sites.

### `get_page`
Takes a path to a `.md` file and returns the associated page

```jinja2
{% set page = get_page(path="blog/page2.md") %}
```

### `get_section`
Takes a path to a `_index.md` file and returns the associated section

```jinja2
{% set section = get_page(path="blog/_index.md") %}
```

### ` get_url`
Gets the permalink for the given path.
If the path starts with `./`, it will be understood as an internal
link like the ones used in markdown.

```jinja2
{% set url = get_url(path="./blog/_index.md") %}
```

This can also be used to get the permalinks for static assets for example if
we want to link to the file that is located at `static/css/app.css`:

```jinja2
{{ get_url(path="css/app.css") }}
```

For assets it is reccommended that you pass `trailing_slash=false` to the `get_url` function. This prevents errors
when dealing with certain hosting providers. An example is:

```jinja2
{{ get_url(path="css/app.css", trailing_slash=false) }}
```

In the case of non-internal links, you can also add a cachebust of the format `?t=1290192` at the end of a URL
by passing `cachebust=true` to the `get_url` function.
