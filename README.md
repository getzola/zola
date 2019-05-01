# zola (né Gutenberg)

[![Build Status](https://travis-ci.com/getzola/zola.svg?branch=master)](https://travis-ci.com/getzola/zola)
[![Build status](https://ci.appveyor.com/api/projects/status/i0ufvx2sdm2cmawo/branch/master?svg=true)](https://ci.appveyor.com/project/Keats/zola/branch/master)

A fast static site generator in a single binary with everything built-in.

Documentation is available on [its site](https://www.getzola.org/documentation/getting-started/installation/) or
in the `docs/content` folder of the repository and the community can use [its forum](https://zola.discourse.group).

## Comparisons with other static site generators

|                                 | Zola                 | Cobalt               | Hugo                 | Pelican              |
|:--------------------------------|:--------------------:|:--------------------:|:--------------------:|:--------------------:|
| Single binary                   | ![yes](./is-yes.svg) | ![yes](./is-yes.svg) | ![yes](./is-yes.svg) | ![no](./is-no.svg)   |
| Language                        | Rust                 | Rust                 | Go                   | Python               |
| Syntax highlighting             | ![yes](./is-yes.svg) | ![yes](./is-yes.svg) | ![yes](./is-yes.svg) | ![yes](./is-yes.svg) |
| Sass compilation                | ![yes](./is-yes.svg) | ![yes](./is-yes.svg) | ![yes](./is-yes.svg) | ![yes](./is-yes.svg) |
| Assets co-location              | ![yes](./is-yes.svg) | ![yes](./is-yes.svg) | ![yes](./is-yes.svg) | ![yes](./is-yes.svg) |
| Multilingual site               | ![ehh](./is-ehh.svg) | ![no](./is-no.svg)   | ![yes](./is-yes.svg) | ![yes](./is-yes.svg) |
| Image processing                | ![yes](./is-yes.svg) | ![no](./is-no.svg)   | ![yes](./is-yes.svg) | ![yes](./is-yes.svg) |
| Sane & powerful template engine | ![yes](./is-yes.svg) | ![yes](./is-yes.svg) | ![ehh](./is-ehh.svg) | ![yes](./is-yes.svg) |
| Themes                          | ![yes](./is-yes.svg) | ![no](./is-no.svg)   | ![yes](./is-yes.svg) | ![yes](./is-yes.svg) |
| Shortcodes                      | ![yes](./is-yes.svg) | ![no](./is-no.svg)   | ![yes](./is-yes.svg) | ![yes](./is-yes.svg) |
| Internal links                  | ![yes](./is-yes.svg) | ![no](./is-no.svg)   | ![yes](./is-yes.svg) | ![yes](./is-yes.svg) |
| Link checker                    | ![yes](./is-yes.svg) | ![no](./is-no.svg)   | ![no](./is-no.svg)   | ![yes](./is-yes.svg) |
| Table of contents               | ![yes](./is-yes.svg) | ![no](./is-no.svg)   | ![yes](./is-yes.svg) | ![yes](./is-yes.svg) |
| Automatic header anchors        | ![yes](./is-yes.svg) | ![no](./is-no.svg)   | ![yes](./is-yes.svg) | ![yes](./is-yes.svg) |
| Aliases                         | ![yes](./is-yes.svg) | ![no](./is-no.svg)   | ![yes](./is-yes.svg) | ![yes](./is-yes.svg) |
| Pagination                      | ![yes](./is-yes.svg) | ![no](./is-no.svg)   | ![yes](./is-yes.svg) | ![yes](./is-yes.svg) |
| Custom taxonomies               | ![yes](./is-yes.svg) | ![no](./is-no.svg)   | ![yes](./is-yes.svg) | ![no](./is-no.svg)   |
| Search                          | ![yes](./is-yes.svg) | ![no](./is-no.svg)   | ![no](./is-no.svg)   | ![yes](./is-yes.svg) |
| Data files                      | ![yes](./is-yes.svg) | ![yes](./is-yes.svg) | ![yes](./is-yes.svg) | ![no](./is-no.svg)   |
| LiveReload                      | ![yes](./is-yes.svg) | ![no](./is-no.svg)   | ![yes](./is-yes.svg) | ![yes](./is-yes.svg) |
| Netlify support                 | ![ehh](./is-ehh.svg) | ![no](./is-no.svg)   | ![yes](./is-yes.svg) | ![no](./is-no.svg)   |
| Breadcrumbs                     | ![yes](./is-yes.svg) | ![no](./is-no.svg)   | ![no](./is-no.svg)   | ![yes](./is-yes.svg) |
| Custom output formats           | ![no](./is-no.svg)   | ![no](./is-no.svg)   | ![yes](./is-yes.svg) | ![no](./is-no.svg)   |

### Supported content formats

- Zola: markdown
- Cobalt: markdown
- Hugo: markdown, asciidoc, org-mode
- Pelican: reStructuredText, markdown, asciidoc, org-mode, whatever-you-want

### ![ehh](./is-ehh.svg) explanations

Hugo gets ![ehh](./is-ehh.svg) for the template engine because while it is probably the most powerful template engine in the list, after Jinja2, it personally drives me insane, to the point of writing my own template engine and static site generator. Yes, this is a bit biased.

Zola gets ![ehh](./is-ehh.svg) for the multi-language support as it only has a basic support and does not offer (yet) things like i18n in templates.

### Pelican notes

Many features of Pelican are coming from plugins, which might be tricky to use because of version mismatch or lacking documentation. Netlify supports Python and Pipenv but you still need to install your dependencies manually.
