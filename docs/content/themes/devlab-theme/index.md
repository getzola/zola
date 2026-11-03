
+++
title = "devlab-theme"
description = "A composable Zola theme for product sites, blogs, documentation and books."
template = "theme.html"
date = 2026-10-30T22:52:35+05:00

[taxonomies]
theme-tags = ['documentation', 'blog', 'book', 'product', 'responsive', 'search', 'dark-mode']

[extra]
created = 2026-10-30T22:52:35+05:00
updated = 2026-10-30T22:52:35+05:00
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
zola check --skip-external-links
zola build
zola serve
```

The consumer regression suite also requires Python 3.11+ and Node.js. It builds
small sites with the theme and checks the rendered HTML and JavaScript behavior:

```sh
node --version && NODE_BINARY=node python3 tests/test_theme.py
python3 tests/test_starter.py
python3 tests/test_component_labels.py
NODE_BINARY=node python3 tests/test_regional_locales.py
python3 tests/test_scroll_regions.py
```

CI runs these checks before building the demo. The Node.js check is required so
the JavaScript tests cannot be silently skipped when Node.js is missing.

Browser checks use a separate development package with Node.js 20+ and
Chromium, Firefox and WebKit. The Chromium print regression also requires
`pdftotext` (the `poppler-utils` package on Debian/Ubuntu):

```sh
cd tests/browser
npm ci
npx playwright install --with-deps chromium firefox webkit
npm test
```

The tests build and serve a temporary consumer site. To use an installed browser,
run `PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH=/path/to/chromium npm test -- --project=chromium`
for a Chromium-only run. The equivalent Firefox and WebKit executable variables
are available for those projects. CI runs all three engines in the Playwright
image matching the pinned test package version. Linux WebKit checks do not
replace testing Safari on a real Apple device.
Browser tools are not required to build a site using the theme.

CI also validates the editor schema against configurations and documented
examples. See [schema development checks](content/docs/reference/configuration/defaults.md#optional-editor-validation)
for its separate Python environment and commands.

The demo's source archives remain unavailable until the corresponding tag is
published. CI verifies the tag and archive availability before enabling them.

## License

[MIT](LICENSE). Library licenses and exact upstream versions are listed in
[Third-party notices](THIRD_PARTY_NOTICES.md) and the
[search module provenance](third_party/lunr-languages/README.md).

        