+++
title = "Robots.txt"
weight = 70
+++

Zola will look for a `robots.txt` file in the `templates` directory or
use the built-in one.

Robots.txt is the simplest of all templates: it only gets the config
and the default is what most site want:

```jinja2
User-agent: *
```
