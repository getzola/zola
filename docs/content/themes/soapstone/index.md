
+++
title = "Soapstone"
description = "A bare bones dark theme with some color tweakability"
template = "theme.html"
date = 2024-01-25T10:41:35+02:00

[extra]
created = 2024-01-25T10:41:35+02:00
updated = 2024-01-25T10:41:35+02:00
repository = "https://github.com/MattyRad/soapstone.git"
homepage = "https://github.com/MattyRad/soapstone"
minimum_version = "0.4.0"
license = "MIT"
demo = "https://mattyrad.github.io/soapstone/"

[extra.author]
name = "Matt Radford"
homepage = "https://mradford.com"
+++        

# Soapstone

A divless dark theme for zola. [See it in action](https://mattyrad.github.io/soapstone/).

![sample](/screenshot.png)

See [installation](https://www.getzola.org/documentation/themes/installing-and-using-themes/) for installation directions.

## Extra config

The following config is optional, but can add a few niceties.

```toml
[extra]
list_header = "Hello World" # title of the main page
favicon_href = "http://example.com" # link to favicon
gravatar_img_src = "https://0.gravatar.com/avatar/abc123?s=60" # adds gravatar image in footer
gravatar_href = "https://example.com" # link for gravatar image
github_link = "https://github.com/JohnDoe" # adds a github link in footer
about_link = "https://example.com" # adds an about link in footer
signature_img_src = "/example.png" # adds an image to bottom of article
signature_text = "Signing off!" # adds signature text to bottom of articles
ga_code = "UA-1234" # adds google analytics code
theme_color = "#000" # for meta browser theme only
```

        