
+++
title = "Clean Blog"
description = "A port of Start Bootstrap Clean Blog for Zola"
template = "theme.html"
date = 2021-02-18T22:27:50+01:00

[extra]
created = 2021-02-18T22:27:50+01:00
updated = 2021-02-18T22:27:50+01:00
repository = "https://github.com/dave-tucker/zola-clean-blog"
homepage = "https://github.com/dave-tucker/zola-clean-blog"
minimum_version = "0.4.0"
license = "MIT"
demo = "https://zola-clean-blog.netlify.com/"

[extra.author]
name = "Dave Tucker"
homepage = "https://dtucker.co.uk"
+++        

zola-clean-blog
===============

![screenshot](screenshot.png)

A port of the StartBootstrap Clean Blog theme, for Zola.

## Demo

[Live Demo](https://zola-clean-blog.netlify.com)

## Usage

To use the theme, clone this repository to your `themes` directory.
It requires that you use the categories and tags taxonomies.
This can be done with the following additions to `config.toml`:
```toml
theme = "zola-clean-blog"

taxonomies = [
    {name = "categories", rss = true, paginate_by=5},
    {name = "tags", rss = true, paginate_by=5},
]
```

## Features

- Paginated Home/Categories/Tag Pages
- Customizable Menu
- Customizable Social Links

## How To Customize

- To replace header images, add a new image to `static/img/$page-bg.jpg` where `$page` is one of `about`, `home`, `post` or `contact`.

- To replace the copyright field, create your own `templates/index.html` to extend the template and add a `copyright` block:
```
{%/* extends "themes/zola-clean-blog/templates/index.html" */%}
{%/* block copyright */%}
Copyright %copy; Example, Inc. 2016-2019
{%/* endblock copyright */%}
```

- To add a new menu item, override `clean_blog_menu` in your `config.toml`. You can use `$BASE_URL` to reference your own site.

- To add a new social link, override `clean_blog_social` in your `config.toml`. You can use `$BASE_URL` to reference your own site.

- To add Google Analytics, you may add your script to the `extrascripts` block using your own `index.html`
```
{%/* extends "themes/zola-clean-blog/templates/index.html" */%}
{%/* block analytics */%}
<script>
...
</script>
{%/* endblock analytics */%}
```
        