
+++
title = "juice"
description = "An intuitive, elegant, and lightweight Zola theme for product sites."
template = "theme.html"
date = 2024-01-25T10:41:35+02:00

[extra]
created = 2024-01-25T10:41:35+02:00
updated = 2024-01-25T10:41:35+02:00
repository = "https://github.com/huhu/juice.git"
homepage = "https://github.com/huhu/juice"
minimum_version = "0.11.0"
license = "MIT"
demo = "https://juice.huhu.io"

[extra.author]
name = "Huhu teams"
homepage = "https://huhu.io"
+++        

# Juice

<img align="right" width="150" height="150" src="/content/juice.svg">

**Juice** is an intuitive, elegant, and responsive Zola theme for product sites.

- Build for product sites
- Simple and intuitive structure
- Clean and elegant design
- Responsive and mobile device compatible
- Customize and extend friendly

https://juice.huhu.io

# Installation

First download this theme to your `themes` directory:

```bash
$ cd themes
$ git clone https://github.com/huhu/juice.git
```

or add as a submodule
```bash
$ git submodule add https://github.com/huhu/juice  themes/juice
```

and then enable it in your `config.toml`:

```toml
theme = "juice"
```

# Structure

### Hero

**Juice** is designed for product websites, hence we let **hero** part fills whole of screen.
You can customize your **hero** by using `hero` block in the `templates/index.html`.

```html
{%/* extends "juice/templates/index.html" */%}
{%/* block hero */%}
    <div>
        Your cool hero html...
    </div>
{%/* endblock hero */%}
```

### Page

Every markdown file located in `content` directory will become a **Page**. There also will display as
a navigate link on the top-right corner.
You can change the frontmatter's `weight` value to sort the order (ascending order).

```
+++
title = "Changelog"
description = "Changelog"
weight = 2
+++

```

### CSS variables

You can override theme variable by creating a file named `_variables.html` in your `templates` directory.

See the default value [here](./templates/_variables.html)

### Favicon
The same way as changing the `hero` block in the `templates/index.html`, you can change the **favicon**.

```html
{%/* extends "juice/templates/index.html" */%}
{%/* block favicon */%}
    <link rel="icon" type="image/png" href="/favicon.ico">
{%/* endblock favicon */%}
```

### Fonts
If you changed the `--xy-font-family`-variable in `_variables.html`, you have to load the mentioned fonts in the `templates/index.html`.

```html
{%/* extends "juice/templates/index.html" */%}
{%/* block fonts */%}
    <link href="https://cdn.jsdelivr.net/npm/fork-awesome@1.2.0/css/fork-awesome.min.css" rel="stylesheet" crossorigin="anonymous">
    <link href="https://fonts.googleapis.com/css2?family=Babylonica&display=swap" rel="stylesheet">
{%/* endblock fonts */%}
```

# Configuration

You can customize some builtin property in `config.toml` file:

```toml
[extra]
juice_logo_name = "Juice"
juice_logo_path = "juice.svg"
juice_extra_menu = [
    { title = "Github", link = "https://github.com/huhu/juice"}
]
juice_exclude_menu = [
    "exclude_from_nav"
]
repository_url = "https://github.com/huhu/juice"
```

# Shortcodes

**Juice** have some builtin shortcodes available in `templates/shortcodes` directory.

- `issue(id)` - A shortcode to render issue url, e.g. `issue(id=1)` would render to the link `https://github.com/huhu/juice/issue/1`.
  
> The `repository_url` is required.

# Showcases

Please see the [showcases page](https://juice.huhu.io/showcases).

# Contributing

Thank you very much for considering contributing to this project!

We appreciate any form of contribution:

- New issues (feature requests, bug reports, questions, ideas, ...)
- Pull requests (documentation improvements, code improvements, new features, ...)
        