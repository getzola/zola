+++
title = "Shortcodes"
weight = 40
+++

While Markdown is good at writing, it isn't great when you need write inline
HTML to add some styling for example.

To solve this, Gutenberg borrows the concept of [shortcodes](https://codex.wordpress.org/Shortcode_API) 
from WordPress.
In our case, the shortcode corresponds to a template that is defined in the `templates/shortcodes` directory or a built-in one.

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

That's it, Gutenberg will now recognise this template as a shortcode named `youtube` (the filename minus the `.html` extension).

## Using shortcodes

There are two kinds of shortcodes: ones that do no take a body like the YouTube example above and ones that do, a quote for example.
In both cases, their arguments must be named and they will all be passed to the template.

Do note that shortcodes in code blocks will be ignored.

### Shortcodes without body
Those look like rendering a variable in Tera.

On a new line, call the shortcode as if it was a function in a variable block. All the examples below are valid
calls of the YouTube shortcode.

```md
{{ youtube(id="w7Ft2ymGmfc") }}

{{ youtube(id="w7Ft2ymGmfc", autoplay=true) }}

{{ youtube(id="w7Ft2ymGmfc", autoplay=true, class="youtube") }}
```

### Shortcodes with body
Those look like a block in Tera.
For example, let's imagine we have the following shortcode `quote.html` template:

```jinja2
<blockquote>
    {{ body }} <br>
    -- {{ author}}
</blockquote>
```

We could use it in our markup file like so:

```md
{% quote(author="Vincent") %}
A quote
{% end %}
```

The `body` variable used in the shortcode template will be implicitly passed down to the rendering
context automatically.

## Built-in shortcodes

Gutenberg comes with a few built-in shortcodes. If you want to override a default shortcode template,
simply place a `{shortcode_name}.html` file in the `templates/shortcodes` directory and Gutenberg will
use that instead.

### YouTube
Embed a responsive player for a YouTube video.

The arguments are:

- `id`: the video id (mandatory)
- `class`: a class to add the `div` surrounding the iframe
- `autoplay`: whether to autoplay the video on load

Usage example:

```md
{{ youtube(id="w7Ft2ymGmfc") }}

{{ youtube(id="w7Ft2ymGmfc", autoplay=true) }}

{{ youtube(id="w7Ft2ymGmfc", autoplay=true, class="youtube") }}
```

Result example:

{{ youtube(id="w7Ft2ymGmfc") }}

### Vimeo
Embed a player for a Vimeo video.

The arguments are:

- `id`: the video id (mandatory)
- `class`: a class to add the `div` surrounding the iframe

Usage example:

```md
{{ vimeo(id="124313553") }}

{{ vimeo(id="124313553", class="vimeo") }}
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
{{ streamable(id="2zt0") }}

{{ streamable(id="2zt0", class="streamble") }}
```

Result example:

{{ streamable(id="2zt0") }}

### Gist
Embed a [Github gist]().

The arguments are:

- `url`: the url to the gist (mandatory)
- `file`: by default, the shortcode will pull every file from the URL unless a specific filename is requested
- `class`: a class to add the `div` surrounding the iframe

Usage example:

```md
{{ gist(id="https://gist.github.com/Keats/e5fb6aad409f28721c0ba14161644c57") }}

{{ gist(id="https://gist.github.com/Keats/e5fb6aad409f28721c0ba14161644c57", class="gist") }}
```

Result example:

{{ gist(url="https://gist.github.com/Keats/e5fb6aad409f28721c0ba14161644c57") }}
