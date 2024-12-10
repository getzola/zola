
+++
title = "Academic Paper"
description = "A Zola theme for a blog-post-style website to facilitate scientific communication of your academic paper"
template = "theme.html"
date = 2024-11-15T14:32:29-05:00

[taxonomies]
theme-tags = []

[extra]
created = 2024-11-15T14:32:29-05:00
updated = 2024-11-15T14:32:29-05:00
repository = "https://github.com/aterenin/academic-paper.git"
homepage = "https://github.com/aterenin/academic-paper"
minimum_version = "0.18.0"
license = "MIT"
demo = "https://aterenin.github.io/academic-paper"

[extra.author]
name = "Alexander Terenin"
homepage = "https://avt.im"
+++        

# Academic Paper: a Zola theme

[Academic Paper](https://aterenin.github.io/academic-paper) is a Zola theme designed for hosting a website for scientific communication of an academic paper in the style of a blog post. 
A demo website built with Academic Paper can be found at [aterenin.github.io/academic-paper](https://aterenin.github.io/academic-paper), and an example repository using this theme can be found at [github.com/aterenin/papers.avt.im](https://github.com/aterenin/papers.avt.im), with links to the pages in this repository found at [avt.im/archive?papers](https://avt.im/archive?papers).

# Features

[Academic Paper](https://github.com/aterenin/academic-paper) is designed to be reasonably feature-complete. Its features include:

* An automatic header which lists the title, author, venue, year, and along with customizable buttons.
* Syntax highlighting and math rendering via KaTeX which can be done both client-side and server-side with appropriate configuration.
* Figures via a Zola shortcode `figure(alt=['Image alt text'],src=['path/to/image.png'])`, which supports captions, subfigures, subcaptions, and is rendered using responsive flexbox.
* Markdown footnotes via Zola's footnote support.
* Metadata including Twitter Summary Card, OpenGraph, and JSON-LD, implemented similar to [Jekyll SEO Tag](https://github.com/jekyll/jekyll-seo-tag): these ensure pages are search-engine-fiendly and provide social media websites with links which are displayed when links are shared,

# Design and maintainability

[Academic Paper](https://github.com/aterenin/academic-paper) is [designed to last](https://jeffhuang.com/designed_to_last/).
This means it follows a set of best practices which try to ensure websites correctly built with it will work correctly in the indefinite future with minimal maintenance, even as the internet changes and links break over time.
As consequence, the theme has no JavaScript or CSS dependencies if KaTeX is used server-side.

# Documentation

The examples below document the theme's options which are available in the TOML files, which are listed as comments within each file.

## Config.toml 

```toml
base_url = "https://example.com"
compile_sass = true # should be set to true
build_search_index = false # not used by the theme
generate_feed = false # not used by the theme
minify_html = true # to ensure correct rendering due to minification of whitespace, should be set to true, unless there is a reason to override it

[markdown]
highlight_code = true # should be set to true unless the page has no code to highlight
highlight_theme = "css" # this theme includes its own CSS-based styling of highlighting, so this should be set to CSS
# other Markdown options - as described in the Zola documentation - go here, and set according to user preference

[extra]
footer_text = "This website is built using [Zola](https://www.getzola.org) and the [Academic Paper](http://github.com/aterenin/academic-paper/) theme, which is [designed to last](https://jeffhuang.com/designed_to_last/)." # by default this page adds a small and non-intrusive footer with some text linking to this repository - you can set this to false to remove the footer if you prefer
server_side_katex = false # set to true to enable server-side KaTeX rendering via scripts/katex.js, this will also include KaTeX CSS and fonts in the build
```

## Page and section configuration 

```toml
+++
title = "Paper Title"
[extra]
authors = [ # authors should be listed as an array in [extra] rather than via Zola's built-in support
    {name = "Author 1", star = true}, # prints a star next to the author name, often useful for 'equal contribution' or similar flags
    {name = "Author 2", url = "https://example.com/", star = true}, # url is optional
    {name = "Author 3"},
]
star = 'Equal contribution' # adds the text 'Equal contribution' with a star superscript to the title
venue = {name = "Example Conference", date = 2023-12-10, url = "https://example.org/"} # date of publication should be listed here, to distinguish it from the date the website itself was written and updated, which can be added via Zola's built-in support
buttons = [ # this theme supports any set of buttons, but and will by default include an SVG icon for the examples listed below
    {name="Paper", url = "https://example.com", no_icon = true}, # to disable drawing the icon, set no_icon to true
    {name="PDF", url = "https://example.com"},
    {name="Code", url = "https://example.com"},
    {name="Video", url = "https://example.com"},
    {name="Slides", url = "https://example.com"},
    {name="Poster", url = "https://example.com"},
    {name="Your custom button", url = "https://example.com"}, # to add an icon, add it as an include, and override the macro icons.html
]
katex = true # to enable math via katex - whether using server-side or client-side rendering - set katex to true
favicon = false # set to true to use favicon.ico as the page's favicon
large_card = false # set to true to generate a large-size Twitter card
+++

Your page's Markdown content goes here...
```
        