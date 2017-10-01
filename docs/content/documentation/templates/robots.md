+++
title = "Robots.txt"
weight = 70
+++

Gutenberg will look for a `robots.txt` file in the `templates` directory or 
use the built-in one.

Robots.txt is the simplest of all templates: it doesn't take any variables
and the default is what most site want.

```jinja2
User-agent: *
```
