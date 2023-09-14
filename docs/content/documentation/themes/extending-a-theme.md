+++
title = "Customizing a theme"
weight = 30
+++

When your site uses a theme, you can replace parts of it in your site's templates folder. For any given theme template, you can either override a single block in it, or replace the whole template. If a site template and a theme template collide, the site template will be given priority. Whether a theme template collides or not, theme templates remain accessible from any template within `theme_name/templates/`.

## Replacing a template

When your site uses a theme, the generated structure follows the theme's structure whenever possible, i.e. there are no user defined templates with the same name and relative path as the theme's; for example: with two files `templates/page.html` and `themes/theme_name/templates/page.html`, the site template is the one that will be used. Such a conflict results in the theme's template being ignored in favor of the template defined by the user.  

## Overriding a block

If you don't want to replace a whole template, but override parts of it, you can [extend the template](https://keats.github.io/tera/docs/#inheritance) and redefine some specific blocks. For example, if you want to override the `title` block in your theme's page.html, you can create a page.html file in your site templates with the following content:

```
{% extends "theme_name/templates/page.html" %}
{% block title %}{{ page.title }}{% endblock %}
```

If you extend `page.html` and not `theme_name/templates/page.html` specifically, it will extend the site's page template if it exists, and the theme's page template otherwise. This makes it possible to override your theme's base template(s) from your site templates, as long as the theme templates do not hardcode the theme name in template paths. For instance, children templates in the theme should use `{% extends 'index.html' %}`, not `{% extends 'theme_name/templates/index.html' %}`.
