
+++
title = "zallery"
description = "Gallery theme for zola"
template = "theme.html"
date = 2024-12-13T19:35:13-06:00

[taxonomies]
theme-tags = []

[extra]
created = 2024-12-13T19:35:13-06:00
updated = 2024-12-13T19:35:13-06:00
repository = "https://github.com/gamingrobot/zallery.git"
homepage = "https://github.com/gamingrobot/zallery"
minimum_version = "0.19.0"
license = "MIT"
demo = "https://gamingrobot.github.io/zallery-demo"

[extra.author]
name = "Morgan Creekmore"
homepage = "https://creekmore.dev"
+++        

# Zallery theme for Zola

Gallery and portfolio theme for [Zola](https://getzola.org).

Demo Site: [gamingrobot.github.io/zallery-demo](https://gamingrobot.github.io/zallery-demo/)  
Personal Portfolio: [gamingrobot.art](https://gamingrobot.art/)

## Screenshots

| Light mode | Dark mode |
| :------: | :-----------: |
| ![light mode](screenshot-light.jpg) | ![dark mode](screenshot-dark.jpg) |

## Features

- Dark and Light mode
- Auto creation of mobile friendly images
- Auto creation of thumbnails
- Auto conversion of images
- Maximize button on images
- [medium-zoom](https://github.com/francoischalifour/medium-zoom) support
- [ModelViewer](https://modelviewer.dev/) and [Sketchfab](https://sketchfab.com/) support
- Video embed support
- OpenGraph and Twitter embed support
- Responsive and mobile friendly

## Installation

Clone the theme into the themes folder:

```bash
git clone https://github.com/gamingrobot/zallery.git themes/zallery
```

Note: It is recomended that you copy the `config.toml` from the `themes/zallery` folder to the root folder of your site.

Then set your theme setting in `config.toml` to `zallery`:

```toml
theme = "zallery"
```

## Customization

To customize the theme's colors you will need to copy the `_variables.scss` into your sites `sass` folder and create a `zallery.scss` file with:

```scss
@import 'variables';
@import '../themes/zallery/sass/imports';
```

See the demo site for an example: [github.com/gamingrobot/zallery-demo/tree/master/sass](https://github.com/gamingrobot/zallery-demo/tree/master/sass)

## Options

### Menu Items

Customize the header navigation links

```toml
[extra]
menu = [
    {url = "atom.xml", name = "Feed"},
    {url = "https://github.com/gamingrobot/zallery", name = "Github"},
]
```

### Browser Bar Theme Color

Customize color to set the browser's url bar on mobile

```toml
[extra]
theme_color = "#313131"
```

### Author Url

Url used for the name in the copyright

```toml
[extra]
author_url = "https://example.com"
```

### Cover Image

Cover image to use on the main gallery pages for opengraph and twitter embeds

```toml
[extra]
cover_image = "img/cover.webp"
```

### Copyright and Powered by

To hide the copyright set this to `true`

```toml
[extra]
hide_copyright = false
```

To hide the "Powered by Zola & Zallery" set this to `true`

```toml
[extra]
hide_poweredby = false
```

### Gallery

Settings for the gallery view's thumbnails

```toml
[extra]
thumbnail_size = 400 # size in pixels, you may need to adjust the media queries in _gallery.scss
thumbnail_format = "webp" # auto, jpg, png, webp
thumbnail_quality = 100 # value in percentage, only for webp and jpg
```

### `img` shortcode settings

Settings for the `img` shortcode, allowing for automatic conversion and creating mobile friendly images

```toml
[extra]
covert_images = false # set to true to convert images to to the format in the image_format setting
create_mobile_images = false # set to true to create mobile friendly versions of the image
image_format = "webp" # auto, jpg, png, webp
image_quality = 90 # value in percentage, only for webp and jpg
```

### Frontmatter settings

These settings are for the frontmatter on each artwork

```toml
[extra]
thumbnail = "image.jpg" # image to resize into a thumbnail and cover image
modelviewer = true # enable modelviewer javascript for this artwork
```

### Javascript libraries

#### ModelViewer

Set to `true` to enable [modelviewer](https://modelviewer.dev/) support. This can also be set in the artwork frontmatter or in `config.toml`

```toml
[extra]
modelviewer = true
```

#### JSZoom

Set to `true` to enable [javascript zoom](https://github.com/francoischalifour/medium-zoom) support.

```toml
[extra]
jszoom = true
```

#### GoatCounter

Set to the goatcounter tag to enable [goatcounter](https://www.goatcounter.com/) support

```toml
[extra]
goatcounter = ""
```

## Shortcodes

### `img`

```jinja2
{{/* img(src="image.jpg", mobile_src="image-mobile.jpg", alt="alt text", text="text", fit="") */}}
```

- `src` (required) - Image path
- `mobile_src` (optional) - Mobile friendly version
- `alt` (optional) - Alt text
- `text` (optional) - Text to put under the image (if `alt` is not specified, text will be use for alt text)
- `fit` (optioanl) - Defaults to `fit-view`, can be set to `max-width` to make the image fill the width of the page

### `video`

```jinja2
{{/* video(src="image.jpg", autoplay=false) */}}
```

- `src` (required) - Video path
- `autoplay` (optional) - Set to `true` to enable autoplay

### `youtube` / `vimeo`

```jinja2
{{/* youtube(id="", autoplay=false) */}}
{{/* vimeo(id="", autoplay=false) */}}
```

- `id` (required) - Id of the video
- `autoplay` (optional) - Set to `true` to enable autoplay

### `model`

Note: Requires `modelviewer` to be enabled in `config.toml`

```jinja2
{{/* model(src="image.jpg", skybox="", poster="") */}}
```

- `src` (required) - Model path
- `skybox` (optional) -  Skybox HDR
- `poster` (optional) - Image to show when loading
- `alt` (optional) - Alt text

### `sketchfab`

```jinja2
{{/* sketchfab(id="") */}}
```

- `id` (required) - Id of the model

        