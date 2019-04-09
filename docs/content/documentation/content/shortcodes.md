+++
title = "Shortcodes"
weight = 40
+++

While Markdown is good at writing, it isn't great when you need write inline
HTML to add some styling for example.

To solve this, Zola borrows the concept of [shortcodes](https://codex.wordpress.org/Shortcode_API)
from WordPress.
In our case, the shortcode corresponds to a template that is defined in the `templates/shortcodes` directory or a built-in one that can
be used in a Markdown file. If you want to use something similar to shortcodes in your templates, try [Tera macros](https://tera.netlify.com/docs/templates/#macros).

## Writing a shortcode
Let's write a shortcode to embed YouTube videos as an example.
In a file called `youtube.html` in the `templates/shortcodes` directory, paste the
following:

```jinja2
<div {% if class %}class="{{class}}"{% endif %}>
    <iframe
        src="https://www.youtube.com/embed/{{id}}{% if autoplay %}?autoplay=1{% endif %}"
        webkitallowfullscreen
        mozallowfullscreen
        allowfullscreen>
    </iframe>
</div>
```

This template is very straightforward: an iframe pointing to the YouTube embed URL wrapped in a `<div>`.
In terms of input, it expects at least one variable: `id`. Since the other variables
are in a `if` statement, we can assume they are optional.

That's it, Zola will now recognise this template as a shortcode named `youtube` (the filename minus the `.html` extension).

The markdown renderer will wrap an inline HTML node like `<a>` or `<span>` into a paragraph. If you want to disable that,
simply wrap your shortcode in a `div`.

Shortcodes are rendered before parsing the markdown so it doesn't have access to the table of contents. Because of that,
you also cannot use the `get_page`/`get_section`/`get_taxonomy` global function. It might work while running `zola serve` because
it has been loaded but it will fail during `zola build`.

## Using shortcodes

There are two kinds of shortcodes:

- ones that do not take a body like the YouTube example above
- ones that do, a quote for example

In both cases, their arguments must be named and they will all be passed to the template.

Lastly, a shortcode name (and thus the corresponding `.html` file) as well as the arguments name
can only contain numbers, letters and underscores, or in Regex terms the following: `[0-9A-Za-z_]`.
While theoretically an argument name could be a number, it will not be possible to use it in the template in that case.

Argument values can be of 5 types:

- string: surrounded by double quotes, single quotes or backticks
- bool: `true` or `false`
- float: a number with a `.` in it
- integer: a number without a `.` in it
- array: an array of any kind of values, except arrays

Malformed values will be silently ignored.

Both type of shortcodes will also get either a `page` or `section` variable depending on where they were used and a `config`
one. Those values will overwrite any arguments passed to a shortcode so shortcodes should not use arguments called like one
of these.

### Shortcodes without body

Simply call the shortcode as if it was a Tera function in a variable block. All the examples below are valid
calls of the YouTube shortcode.

```md
Here is a YouTube video:

{{/* youtube(id="dQw4w9WgXcQ") */}}

{{/* youtube(id="dQw4w9WgXcQ", autoplay=true) */}}

An inline {{/* youtube(id="dQw4w9WgXcQ", autoplay=true, class="youtube") */}} shortcode
```

Note that if you want to have some content that looks like a shortcode but not have Zola try to render it,
you will need to escape it by using `{{/*` and `*/}}` instead of `{{` and `}}`.

### Shortcodes with body
For example, let's imagine we have the following shortcode `quote.html` template:

```jinja2
<blockquote>
    {{ body }} <br>
    -- {{ author}}
</blockquote>
```

We could use it in our markup file like so:

```md
As someone said:

{%/* quote(author="Vincent") */%}
A quote
{%/* end */%}
```

The body of the shortcode will be automatically passed down to the rendering context as the `body` variable and needs
to be in a newline.

If you want to have some content that looks like a shortcode but not have Zola try to render it,
you will need to escape it by using `{%/*` and `*/%}` instead of `{%` and `%}`. You won't need to escape
anything else until the closing tag.

## Built-in shortcodes

Zola comes with a few built-in shortcodes. If you want to override a default shortcode template,
simply place a `{shortcode_name}.html` file in the `templates/shortcodes` directory and Zola will
use that instead.

### YouTube
Embed a responsive player for a YouTube video.

The arguments are:

- `id`: the video id (mandatory)
- `class`: a class to add the `div` surrounding the iframe
- `autoplay`: whether to autoplay the video on load

Usage example:

```md
{{/* youtube(id="dQw4w9WgXcQ") */}}

{{/* youtube(id="dQw4w9WgXcQ", autoplay=true) */}}

{{/* youtube(id="dQw4w9WgXcQ", autoplay=true, class="youtube") */}}
```

Result example:

{{ youtube(id="dQw4w9WgXcQ") }}

### Vimeo
Embed a player for a Vimeo video.

The arguments are:

- `id`: the video id (mandatory)
- `class`: a class to add the `div` surrounding the iframe

Usage example:

```md
{{/* vimeo(id="124313553") */}}

{{/* vimeo(id="124313553", class="vimeo") */}}
```

Result example:

{{ vimeo(id="124313553") }}

### Streamable
Embed a player for a Streamable video.

The arguments are:

- `id`: the video id (mandatory)
- `class`: a class to add the `div` surrounding the iframe

Usage example:

```md
{{/* streamable(id="92ok4") */}}

{{/* streamable(id="92ok4", class="streamble") */}}
```

Result example:

{{ streamable(id="92ok4") }}

### Gist
Embed a [Github gist](https://gist.github.com).

The arguments are:

- `url`: the url to the gist (mandatory)
- `file`: by default, the shortcode will pull every file from the URL unless a specific filename is requested
- `class`: a class to add the `div` surrounding the iframe

Usage example:

```md
{{/* gist(url="https://gist.github.com/Keats/e5fb6aad409f28721c0ba14161644c57") */}}

{{/* gist(url="https://gist.github.com/Keats/e5fb6aad409f28721c0ba14161644c57", class="gist") */}}
```

Result example:

{{ gist(url="https://gist.github.com/Keats/e5fb6aad409f28721c0ba14161644c57") }}
