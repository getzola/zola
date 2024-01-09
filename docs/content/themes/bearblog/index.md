
+++
title = "Bear"
description = "Bear blog theme"
template = "theme.html"
date = 2024-01-09T08:33:22+01:00

[extra]
created = 2024-01-09T08:33:22+01:00
updated = 2024-01-09T08:33:22+01:00
repository = "https://codeberg.org/alanpearce/zola-bearblog"
homepage = "https://codeberg.org/alanpearce/zola-bearblog"
minimum_version = "0.4.0"
license = "MIT"
demo = "https://zola-bearblog.netlify.app/"

[extra.author]
name = "Alan Pearce"
homepage = "https://alanpearce.eu"
+++        

# Zola ʕ•ᴥ•ʔ Bear Blog

[![Netlify Status](https://api.netlify.com/api/v1/badges/121b53ce-c913-4604-9179-eb3cca31cd2c/deploy-status)](https://app.netlify.com/sites/zola-bearblog/deploys)

🧸 A [Zola](https://www.getzola.org/)-theme based on [Bear Blog](https://bearblog.dev).

> Free, no-nonsense, super-fast blogging.

## Demo

For a current & working demo of this theme, please check out <https://zola-bearblog.netlify.app/> 🎯.

## Screenshot

![Screenshot][screenshot]

When the user's browser is running »dark mode«, the dark color scheme will be used automatically. The default is the light/white color scheme. Check out the [`style.html`](https://codeberg.org/alanpearce/zola-bearblog/src/branch/main/templates/style.html)-file for the implementation.

## Installation

If you already have a Zola site on your machine, you can simply add this theme via

```
git submodule add https://codeberg.org/alanpearce/zola-bearblog themes/zola-bearblog
```

Then, adjust the `config.toml` as detailed below.

For more information, read the official [setup guide][zola-setup-guide] of Zola.

Alternatively, you can quickly deploy a copy of the theme site to Netlify using this button:

[![Deploy to Netlify](https://www.netlify.com/img/deploy/button.svg)](https://app.netlify.com/start/deploy?repository=https://gitlab.com/alanpearce/zola-bearblog)

(Note that this method makes it harder to keep up-to-date with theme updates, which might be necessary for newer versions of Zola.)

## Adjust configuration / config.toml

Please check out the included [config.toml](https://codeberg.org/alanpearce/zola-bearblog/src/branch/main/config.toml)

## Content & structure

### Menu

Create an array in `extra` with a key of `main_menu`. `url` is passed to [`get_url`](https://www.getzola.org/documentation/templates/overview/#get-url)

```toml
[[extra.main_menu]]
name = "Bear"
url = "@/bear.md"

[[extra.main_menu]]
name = "Zola"
url = "@/zola.md"

[[extra.main_menu]]
name = "Blog"
url = "@/blog/_index.md"
```

### Adding / editing content

#### Index-Page

The contents of the `index`-page may be changed by editing your `content/_index.md`-file.


### Adding your branding / colors / css

Add a `custom_head.html`-file to your `templates/`-directory. In there you may add a `<style>`-tag, *or* you may add a `<link>`-tag referencing your own `custom.css` (in case you prefer to have a separate `.css`-file). Check out the [`style.html`](https://codeberg.org/alanpearce/zola-bearblog/src/branch/main/templates/style.html)-file to find out which CSS-styles are applied by default.

## Issues / Feedback / Contributing
Please use [Codeberg issues](https://codeberg.org/alanpearce/zola-bearblog/issues) and [Pull Requests](https://codeberg.org/alanpearce/zola-bearblog/pulls).

## Special Thanks 🎁

A special thank you goes out to [Herman](https://herman.bearblog.dev), for creating the original [ʕ•ᴥ•ʔ Bear Blog](https://bearblog.dev/) and [Jan Raasch](https://www.janraasch.com) for creating the hugo port of the Bear Blog theme.

## License
[MIT License](http://en.wikipedia.org/wiki/MIT_License) © [Alan Pearce](https://www.alanpearce.eu/)

[zola-setup-guide]: https://www.getzola.org/documentation/getting-started/installation/
[screenshot]: https://codeberg.org/alanpearce/zola-bearblog/raw/branch/main/screenshot.png

        