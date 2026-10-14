
+++
title = "devlab-theme"
description = "A composable Zola theme for product sites, blogs, documentation and books."
template = "theme.html"
date = 2026-10-06T17:09:20+02:00

[taxonomies]
theme-tags = ['documentation', 'blog', 'book', 'product', 'responsive', 'search', 'dark-mode']

[extra]
created = 2026-10-06T17:09:20+02:00
updated = 2026-10-06T17:09:20+02:00
repository = "https://codeberg.org/ripetitor/devlab-theme.git"
homepage = "https://codeberg.org/RiPetitor/devlab-theme"
minimum_version = "0.23.6"
license = "MIT"
demo = "https://ripetitor.codeberg.page/devlab-theme/"

[extra.author]
name = "RiPetitor"
homepage = "https://codeberg.org/RiPetitor"
+++        

[![Please don't upload to GitHub](https://nogithub.codeberg.page/badge.svg)](https://nogithub.codeberg.page)
[![Zola](https://img.shields.io/badge/Zola-0.23.6-blue?style=flat-square)](https://www.getzola.org/)
[![Version](https://img.shields.io/badge/version-0.9.0-blue?style=flat-square)](content/blog/devlab-theme-v0-9-0.md)
[![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)](LICENSE)

# DevLab Theme

A Zola theme with layouts for pages, blogs, documentation and books. Content is
written in Markdown. Configuration lives in `zola.toml`. Builds require Zola
**0.23.6 or newer**.

The [v0.9.0 release notes](content/blog/devlab-theme-v0-9-0.md) describe layouts,
collections, navigation, community cards and template extensions.

![DevLab documentation](screenshot.png)

[Live demo](https://ripetitor.codeberg.page/devlab-theme/) · [Documentation](https://ripetitor.codeberg.page/devlab-theme/docs/) · [Updates](https://ripetitor.codeberg.page/devlab-theme/blog/)

## Setup

[Install the theme](content/docs/getting-started/installation.md), then create
`zola.toml` and `content/_index.md` with the
[quick start](content/docs/getting-started/quick-start.md).
Keep content, styles and template overrides in the site repository.

## Documentation

| Topic | Reference |
| --- | --- |
| Landing blocks | [Landing pages](content/docs/reference/landing.md) |
| Independent documentation trees | [Collections](content/docs/reference/collections.md) |
| First post, tags, authors and feeds | [Blog guide](content/docs/guides/blog.md) |
| Books and chapter order | [Reader](content/docs/reference/reader.md) |
| Site and theme settings | [Configuration](content/docs/reference/configuration/_index.md) |
| Callouts, cards, tabs and other content blocks | [Components](content/docs/reference/components/_index.md) |
| Translations and sharing metadata | [Languages](content/docs/reference/languages.md) |
| Template slots and CSS tokens | [Extensions](content/docs/reference/extensions.md) |
| Updating a v0.8 site | [Migration](content/docs/guides/upgrading-v0-8.md) |
| Hosting and live-route checks | [Deployment](content/docs/guides/deployment.md) |

## Development

Building and previewing the theme requires Zola 0.23.6 or newer. Start from the
source `main` branch, since `pages` contains the generated demo.

```sh
zola build
zola serve
```

The demo's source archives remain unavailable until the corresponding tag is
published. CI verifies the tag and archive availability before enabling them.

## License

[MIT](LICENSE). Library licenses and exact upstream versions are listed in
[Third-party notices](THIRD_PARTY_NOTICES.md) and the
[search module provenance](third_party/lunr-languages/README.md).

        