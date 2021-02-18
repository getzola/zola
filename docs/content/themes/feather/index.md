
+++
title = "feather"
description = "A modern blog theme"
template = "theme.html"
date = 2021-02-18T22:27:50+01:00

[extra]
created = 2021-02-18T22:27:50+01:00
updated = 2021-02-18T22:27:50+01:00
repository = "https://github.com/piedoom/feather"
homepage = "https://github.com/piedoom/feather"
minimum_version = "0.5.1"
license = "MIT"
demo = ""

[extra.author]
name = "doomy"
homepage = "https://alexanderlozada.com"
+++        

# feather
A lightweight theme for zola 

Feather is a blog theme specifically designed for the static site generator [Zola](https://www.getzola.org/).  It's as far as 
I know the first theme for the platform that isn't a port or just contained within template files.  

Feather is *not* intended for anything other than blogs.  

# Developing & Contributing
Because feather comes with example content, you can run the theme just like any Gutenberg
blog with `zola serve`.  Your changes will autoreload!

## Considerations
Please don't edit the `content` folder directly for your own blog, use it as a theme like
intended!

# Usage
Using feather is easy.  Install [Zola](https://www.getzola.org/) and follow 
[the guide for creating a site and using a theme](https://www.getzola.org/documentation/themes/installing-and-using-themes/).  Then,
add `theme = "feather"` to your `config.toml` file.

If you intend to publish your site to Github Pages, please check out [this tutorial](http://vaporsoft.net/publishing-gutenberg-to-github/).

## Options
Zola allows themes to [define `[extra]` variables](https://www.getzola.org/documentation/getting-started/configuration/)
in the config.  Here's a full list of theme variables with example values and comments.

```
[extra]
# specify an image for the big header at the top of the page.
# this will be a background cover image, so make it large enough
# so that it won't upscale enough to pixelate
feather_header_image = "/theme_images/default.gif"

# if you want comments, add your disqus embed code here.  
# you can obtain a code on the disqus website.
feather_disqus_code = "<div id='disqus_thread'></div><script> var disqus_config = ... etc ... etc ..."

# if you'd like people to be able to donate to you, you can include a donate link
# to be displayed in the footer of the website.
feather_donate_link = "https://paypal.me/piedoomy"

# if you want analytics, add your tracking code HTML here.
feather_analytics = "<script async src='https://www.googletagmanager.com/gtag/... etc ... etc ..."
```

## Features
Feather also provides several features optional.

- [x] Categories page
- [x] Tags page
- [x] Sorting chronologically 
- [x] Comments
- [x] Analytics 
- [x] Donate links

## Using Categories and Tags
In order to use categories and tags, you'll need to enable their compilation by adding the following lines to your `config.toml`
(make sure they aren't under `[extra]`):

```toml
generate_categories_pages = true
generate_tags_pages = true
```

# Live demo
[My dev site](http://vaporsoft.net/) uses the latest version of feather.


        