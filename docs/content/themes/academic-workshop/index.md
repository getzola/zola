
+++
title = "Academic Workshop"
description = "A Zola theme for a website to list the schedule of your scientific workshop or seminar series"
template = "theme.html"
date = 2024-12-16T12:27:08-08:00

[taxonomies]
theme-tags = []

[extra]
created = 2024-12-16T12:27:08-08:00
updated = 2024-12-16T12:27:08-08:00
repository = "https://github.com/aterenin/academic-workshop.git"
homepage = "https://github.com/aterenin/academic-workshop"
minimum_version = "0.18.0"
license = "MIT"
demo = "https://aterenin.github.io/academic-workshop"

[extra.author]
name = "Alexander Terein"
homepage = "https://avt.im"
+++        

# Academic Workshop: a Zola theme

[Academic Workshop](https://aterenin.github.io/academic-workshop) is a Zola theme desgned for hosting a website for scientific workshop or scientific seminar series.
A demo website built with Academic Workshop can be found at [aterenin.github.io/academic-workshop](https://aterenin.github.io/academic-workshop), and example repositories using this theme, for a seminar series and workshop, respectively, can be found at [github.com/gp-seminar-series/gp-seminar-series.github.io](https://github.com/gp-seminar-series/gp-seminar-series.github.io) and [https://github.com/gp-seminar-series/neurips-2024](https://github.com/gp-seminar-series/neurips-2024).

# Features

[Academic Workshop](https://github.com/aterenin/academic-workshop) is designed to be reasonably feature-complete. Its features include:

* An automatic header which lists the tagline, title, and subtitles, with customizable buttons, as well as a background banner image with customizable CSS.
* Shortcodes for various types of lists such as lists of speakers and lists of previous seminars, as well as highlighting upcoming seminars on the front page, implemented in a fully responsive way to look professional on both desktop and mobile devices.
* Smart image resizing in image-grid-type lists, such as for lists of speakers.
* Support for creating a table of accepted workshop papers from a CSV file.
* Metadata including Twitter Summary Card, OpenGraph, and JSON-LD, implemented similar to [Jekyll SEO Tag](https://github.com/jekyll/jekyll-seo-tag): these ensure pages are search-engine-fiendly and provide social media websites with links which are displayed when links are shared,

# Design and maintainability

[Academic Workshop](https://github.com/aterenin/academic-workshop) is [designed to last](https://jeffhuang.com/designed_to_last/).
This means it follows a set of best practices which try to ensure websites correctly built with it will work correctly in the indefinite future with minimal maintenance, even as the internet changes and links break over time.
As consequence, the theme has no JavaScript or CSS dependencies of any kind.

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
highlight_code = false # should be set to false unless the page has code to highlight

[extra]
footer_text = "This website is built using [Zola](https://www.getzola.org) and the [Academic Workshop](http://github.com/aterenin/academic-workshop/) theme, which is [designed to last](https://jeffhuang.com/designed_to_last/)." # by default this page adds a small and non-intrusive footer with some text linking to this repository - you can set this to false to remove the footer if you prefer
title = {tagline = "Presenting the", title = "Academic Workshop Zola Theme", subtitles = ["For workshops, seminars, and academic events"]} # this contains the header's tagline, title, and list of subtitles, which are displayed in order
banner = {extension = "svg", size = "fixed", fade = true} # this enables a banner image stored in static/banner.svg, with the CSS class bg-fixed: this CSS class is designed for users to set the background image's height and width by overriding CSS - see _main.scss for other classes like bg-contain or bg-cover - the fade option enables a CSS-based fade around the text
buttons = [{name = "Example", url = "https://example.com/"}, {name="GitHub", url="http://github.com/aterenin/academic-workshop"}] # creates a list of buttons displayed in order, with links to the URLs
image = {resize = 400, ext = '.jpg'} # this sets the desired size for image resizing, as well as the default extension
list_page_limit = 10 # this sets the default number of items which show up in one page in a list
header_pages = [{name = "Home", url = "/#home"},{name = "Design", url = "/#design"}] # this sets the pages which show up in the navigation menu which gets displayed on mobile devices
```

## Page configuration 

If using this theme for a list of seminars, each seminar should be added as a page within a suitable section. An example configuration is given here. 

```toml
+++
title = "Seminar Title"
[extra]
author = "Example Author" # name of the author goes here
institution = "Example Institution" # name of the institution goes here
author_url = "https://example.com" # the author's webpage, which the theme will create links to
time = "16:00 UTC" # the time of the seminar
buttons = [{name = "Example", url = "https://example.com/"}] # a list of buttons displayed on the page and in lists, shown in order
highlight = true # whether or not the seminar gets displayed on a page using the highlight shortcode, which is designed to display a short list of upcoming seminars on the theme's front page - defaults to false
image = "placeholder.svg" # optional: allows ont to override the default image URL
+++

The seminar's abstract goes here
```
        