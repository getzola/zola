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

There are two kinds of shortcodes: 

- ones that do not take a body like the YouTube example above
- ones that do, a quote for example

In both cases, their arguments must be named and they will all be passed to the template.

Any shortcodes in code blocks will be ignored.

Lastly, a shortcode name (and thus the corresponding `.html` file) as well as the arguments name 
can only contain numbers, letters and underscores, or in Regex terms the following: `[0-9A-Za-z_]`.
While theoretically an argument name could be a number, it will not be possible to use it in the template in that case.

Argument values can be of 4 types:

- string: surrounded by double quotes `"..."`
- bool: `true` or `false`
- float: a number with a `.` in it
- integer: a number without a `.` in it

Malformed values will be silently ignored.

### Shortcodes without body

On a new line, call the shortcode as if it was a Tera function in a variable block. All the examples below are valid
calls of the YouTube shortcode.

```md
Here is a YouTube video:

{{ youtube(id="dQw4w9WgXcQ") }}

{{ youtube(id="dQw4w9WgXcQ", autoplay=true) }}

{{ youtube(id="dQw4w9WgXcQ", autoplay=true, class="youtube") }}
```

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

{% quote(author="Vincent") %}
A quote
{% end %}
```

The body of the shortcode will be automatically passed down to the rendering context as the `body` variable and needs
to be in a newline.

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
{{ youtube(id="dQw4w9WgXcQ") }}

{{ youtube(id="dQw4w9WgXcQ", autoplay=true) }}

{{ youtube(id="dQw4w9WgXcQ", autoplay=true, class="youtube") }}
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
{{ gist(url="https://gist.github.com/Keats/e5fb6aad409f28721c0ba14161644c57") }}

{{ gist(url="https://gist.github.com/Keats/e5fb6aad409f28721c0ba14161644c57", class="gist") }}
```

Result example:

{{ gist(url="https://gist.github.com/Keats/e5fb6aad409f28721c0ba14161644c57") }}

### Twitch
Embed a Twitch stream with or without chat.

The arguments are:

- `channel`: the channel name (mandatory)
- `chat`: shows the stream chat next to the video embed 
- `class`: a class to add the `div` surrounding the iframe
- `videoWidth`: the px **width** the video embed should have 
- `videoHeight`: the px **height** the video embed should have
- `chatWidth`: the px **width** the chat embed should have 
- `chatHeight`: the px **height** the chat embed should have

Usage example:

```md
{{ twitch(channel="twitch") }}

{{ twitch(channel="twitch", class="some-class", chat=true) }}

{{ twitch(channel="twitch", chat=true, videoWidth=300. videoHeight=100, chatWidth=100, chatHeight=100) }}
```

Result example:

{{ twitch(channel="twitch", chat=true, videoWidth=300, videoHeight=100, chatWidth=100, chatHeight=100, class="twitch-shortcode") }}
