
+++
title = "UI Navigation"
description = "A multilingual theme designed for accessibility rather than visual."
template = "theme.html"
date = 2025-01-12T12:55:23+06:30

[taxonomies]
theme-tags = ['blind', 'multilingual', 'accessible', 'responsive']

[extra]
created = 2025-01-12T12:55:23+06:30
updated = 2025-01-12T12:55:23+06:30
repository = "https://github.com/harrymkt/zluinav"
homepage = "https://github.com/harrymkt/zluinav"
minimum_version = "0.19.2"
license = "MIT"
demo = "https://harrymkt.github.io/zluinav"

[extra.author]
name = "Harry Min Khant"
homepage = "https://harrymkt.github.io"
+++        

# UI Navigation
UI Navigation, or known as zluinav, is a Zola theme designed for accessibility rather than visual and made as easy as possible using templates and macros. Since I am a blind developer, I'd like to develop with accessibility as possible so visually impaired users can use them.

This theme is also available for Hugo at [Hguinav](https://github.com/harrymkt/hguinav).

[Zola](https://www.getzola.org/) is a fast site generator written in Rust powered by tera as its templating engine and has a powerful theme creation feature.

[Theme demo](https://harrymkt.github.io/zluinav)

## License
This theme is distributed under the terms of the [MIT License](https://github.com/harrymkt/zluinav/blob/main/LICENSE.md).

## Features of zluinav theme
- Accessibility; Zluinav is designed to be accessible as possible, especially for blind and visually impaired. This is done by using accessibility tags, such as ARIA, and other possible accessibility features.
- Configuration; use an extensive [configuration](https://harrymkt.github.io/zluinav/docs#extra-variables) parameters to control your site, from the main configuration file to [frontmadder configuration](https://harrymkt.github.io/zluinav/docs/extra/frontmadder), and more.
- Blog with pagination enabled; multiple blogs can be created by copying the blog directory in the content folder to the new directory for a new blog. This means that you can have multiple blogs in one site. In fact, Zola doesn't have its build-in posts, but it is possible using sections. Please note that directory other than blog will require you to manually set the `template` to `blogpage.html` and `page_template` to `section_paginated.html` in its `_index.md` file.
- Documentation site; build accessible [documentation sites](https://harrymkt.github.io/zluinav/docs/documentation) by using built-in 1subsection templates specifically designed for documentation.
- Multilingual; build your site in multiple languages.
- Custom [Menus](https://harrymkt.github.io/zluinav/docs/extra/config#menus); can be set via `config.extra.menus.menu_name`.
- Taxonomies support.
- Built-in [search](https://harrymkt.github.io/zluinav/docs/search) support, with a variety of search formats to choose.
- Customizable extrahead, header, navigation, and footer by base templates and blocks.
- Fast; Zola generates within a few milliseconds. Zluinav is built with HTML using aria whenever possible for accessibility with assistive screen reader as well as using JavaScript. You can rebase the templates, should you wish to add your own content.
- Copy code blocks; add code blocks which can then be copied using buttons and display the code language if available, helped by JavaScript.
- Variables; add [variables](https://harrymkt.github.io/zluinav/docs/writing) to your page content to be replaced during the site generate.
- Local date display; display dates in user's local timezone, no madder what timezone the date is set.
- Use extensive macros and shortcodes to make your content length shorten.
- Ability to toggle the use of JavaScript for both config and per-page frontmadder.
- Comprehensive documentation; Zluinav provides a full Comprehensive documentation including possible templates, shortcodes, blocks, configurable parameters, and more, everything as it updates.

## Installation
Using git clone:
```bash
cd themes
git clone https://github.com/harrymkt/zluinav.git
```
Or [download manually](https://github.com/harrymkt/zluinav/archive/refs/heads/main.zip) and paste in the themes directory.

Or, add to the Git submodule (recommended):
```bash
git submodule add --name zluinav https://github.com/harrymkt/zluinav.git themes/zluinav
git submodule update --remote
```

In your config.toml file, add the following
```toml
theme = "zluinav"
```

## Customization
For more customizable options and configurations, please see [documentation](https://harrymkt.github.io/zluinav/docs)

## Contribution
Contributions to this theme are welcome, provided that the following requirements are met:
- Use 2 level space indentation for HTML. Use 1 tab level indentation for CSS and JavaScript. If Markdown files need indentation, use 1 tab.
- Be the templates accessible for visually impaired and/or blind and prefer readability. Don't worry, I will process in case accessibility issues before pull requests are merged.
- Photos are not required in this theme. You may design with CSS for visual if you so wish.
- When creating a pull request it is advised that you:
	- Use different branch other than main; this avoids issues with updating in case your pull request gets rejected.
	- Add label if possible.

Thank you!
        