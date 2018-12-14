# zola (né Gutenberg)
[![Build Status](https://travis-ci.com/getzola/zola.svg?branch=master)](https://travis-ci.com/getzola/zola)
[![Build status](https://ci.appveyor.com/api/projects/status/i0ufvx2sdm2cmawo/branch/master?svg=true)](https://ci.appveyor.com/project/Keats/zola/branch/master)

A fast static site generator in a single binary with everything built-in.

Documentation is available on [its site](https://www.getzola.org/documentation/getting-started/installation/) or
in the `docs/content` folder of the repository and the community can use [its forum](https://zola.discourse.group).

## Comparisons with other static site generators

|                                 |    Zola   | Cobalt | Hugo | Pelican |
|:-------------------------------:|:---------:|--------|------|---------|
| Single binary                   |     ✔     |    ✔   |   ✔  |    ✕    |
| Language                        |    Rust   |  Rust  |  Go  |  Python |
| Syntax highlighting             |     ✔     |    ✔   |   ✔  |    ✔    |
| Sass compilation                |     ✔     |    ✔   |   ✔  |    ✔    |
| Assets co-location              |     ✔     |    ✔   |   ✔  |    ✔    |
| i18n                            |     ✕     |    ✕   |   ✔  |    ✔    |
| Image processing                |     ✔     |    ✕   |   ✔  |    ✔    |
| Sane & powerful template engine |     ✔     |    ~   |   ~  |    ✔    |
| Themes                          |     ✔     |    ✕   |   ✔  |    ✔    |
| Shortcodes                      |     ✔     |    ✕   |   ✔  |    ✔    |
| Internal links                  |     ✔     |    ✕   |   ✔  |    ✔    |
| Link checker                    |     ✔     |    ✕   |   ✕  |    ✔    |
| Table of contents               |     ✔     |    ✕   |   ✔  |    ✔    |
| Automatic header anchors        |     ✔     |    ✕   |   ✔  |    ✔    |
| Aliases                         |     ✔     |    ✕   |   ✔  |    ✔    |
| Pagination                      |     ✔     |    ✕   |   ✔  |    ✔    |
| Custom taxonomies               |     ✔     |    ✕   |   ✔  |    ✕    |
| Search                          |     ✔     |    ✕   |   ✕  |    ✔    |
| Data files                      |     ✔     |    ✔   |   ✔  |    ✕    |
| LiveReload                      |     ✔     |    ✕   |   ✔  |    ✔    |
| Netlify support                 |     ~     |    ✕   |   ✔  |    ✕    |
| Breadcrumbs                     |     ✔     |    ✕   |   ✕  |    ✔    |
| Custom output formats           |     ✕     |    ✕   |   ✔  |    ?    |


### Supported content formats

- Zola: markdown
- Cobalt: markdown
- Hugo: markdown, asciidoc, org-mode
- Pelican: reStructuredText, markdown, asciidoc, org-mode, whatever-you-want

### Template engine explanation

Cobalt gets `~` as, while based on [Liquid](https://shopify.github.io/liquid/), the Rust library doesn't implement all its features but there is no documentation on what is and isn't implemented. The errors are also cryptic. Liquid itself is not powerful enough to do some of things you can do in Jinja2, Go templates or Tera.

Hugo gets `~`. It is probably the most powerful template engine in the list after Jinja2 (hard to beat python code in templates) but personally drives me insane, to the point of writing my own template engine and static site generator. Yes, this is a bit biased.

### Pelican notes
Many features of Pelican are coming from plugins, which might be tricky
to use because of version mismatch or lacking documentation. Netlify supports Python
and Pipenv but you still need to install your dependencies manually.
