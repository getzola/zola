# zola (né Gutenberg)

[![Build Status](https://dev.azure.com/getzola/zola/_apis/build/status/getzola.zola?branchName=master)](https://dev.azure.com/getzola/zola/_build/latest?definitionId=1&branchName=master)

A fast static site generator in a single binary with everything built-in.

Documentation is available on [its site](https://www.getzola.org/documentation/getting-started/installation/) or
in the `docs/content` folder of the repository and the community can use [its forum](https://zola.discourse.group).

## Comparisons with other static site generators

|                                 | Zola   | Cobalt | Hugo   | Pelican |
|:--------------------------------|:------:|:------:|:------:|:-------:|
| Single binary                   | ![yes] | ![yes] | ![yes] | ![no]   |
| Language                        | Rust   | Rust   | Go     | Python  |
| Syntax highlighting             | ![yes] | ![yes] | ![yes] | ![yes]  |
| Sass compilation                | ![yes] | ![yes] | ![yes] | ![yes]  |
| Assets co-location              | ![yes] | ![yes] | ![yes] | ![yes]  |
| Multilingual site               | ![ehh] | ![no]  | ![yes] | ![yes]  |
| Image processing                | ![yes] | ![no]  | ![yes] | ![yes]  |
| Sane & powerful template engine | ![yes] | ![yes] | ![ehh] | ![yes]  |
| Themes                          | ![yes] | ![no]  | ![yes] | ![yes]  |
| Shortcodes                      | ![yes] | ![no]  | ![yes] | ![yes]  |
| Internal links                  | ![yes] | ![no]  | ![yes] | ![yes]  |
| Link checker                    | ![yes] | ![no]  | ![no]  | ![yes]  |
| Table of contents               | ![yes] | ![no]  | ![yes] | ![yes]  |
| Automatic header anchors        | ![yes] | ![no]  | ![yes] | ![yes]  |
| Aliases                         | ![yes] | ![no]  | ![yes] | ![yes]  |
| Pagination                      | ![yes] | ![no]  | ![yes] | ![yes]  |
| Custom taxonomies               | ![yes] | ![no]  | ![yes] | ![no]   |
| Search                          | ![yes] | ![no]  | ![no]  | ![yes]  |
| Data files                      | ![yes] | ![yes] | ![yes] | ![no]   |
| LiveReload                      | ![yes] | ![no]  | ![yes] | ![yes]  |
| Netlify support                 | ![yes] | ![no]  | ![yes] | ![no]   |
| Vercel support                  | ![yes] | ![no]  | ![yes] | ![yes]  |
| Cloudflare Pages support        | ![yes] | ![no]  | ![yes] | ![yes]  |
| Breadcrumbs                     | ![yes] | ![no]  | ![no]  | ![yes]  |
| Custom output formats           | ![no]  | ![no]  | ![yes] | ![no]   |

### Supported content formats

- Zola: markdown
- Cobalt: markdown
- Hugo: markdown, asciidoc, org-mode
- Pelican: reStructuredText, markdown, asciidoc, org-mode, whatever-you-want

### ![ehh] explanations

Hugo gets ![ehh] for the template engine because while it is probably the most powerful template engine in the list (after Jinja2) it personally drives me insane, to the point of writing my own template engine and static site generator. Yes, this is a bit biased.

Zola gets ![ehh] for multi-language support as it only has a basic support and does not (yet) offer things like i18n in templates.

### Pelican notes

Many features of Pelican come from plugins, which might be tricky to use because of a version mismatch or inadequate documentation. Netlify supports Python and Pipenv but you still need to install your dependencies manually.

[yes]: ./is-yes.svg
[ehh]: ./is-ehh.svg
[no]:  ./is-no.svg
