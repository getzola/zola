# zola (né Gutenberg) <img src="docs/static/logos/Zola-logo-main-coffee.svg" align="right" alt="zola logo" width="30%"/>

[![Build Status](https://dev.azure.com/getzola/zola/_apis/build/status/getzola.zola?branchName=master)](https://dev.azure.com/getzola/zola/_build/latest?definitionId=1&branchName=master)
![GitHub all releases](https://img.shields.io/github/downloads/getzola/zola/total)

A fast static site generator in a single binary with everything built-in.

To find out more see the [Zola Documentation](https://www.getzola.org/documentation/getting-started/overview/), look
in the [docs/content](docs/content) folder of this repository or visit the [Zola community forum](https://zola.discourse.group).

This tool and its template engine [tera](https://keats.github.io/tera/) were born from an intense dislike of the (insane) Golang template engine and therefore of
Hugo that I was using before for 6+ sites.

## List of features

- [Single binary](https://www.getzola.org/documentation/getting-started/cli-usage/)
- [Syntax highlighting](https://www.getzola.org/documentation/content/syntax-highlighting/)
- [Sass compilation](https://www.getzola.org/documentation/content/sass/)
- Assets co-location
- [Multilingual site support](https://www.getzola.org/documentation/content/multilingual/) (Basic currently)
- [Image processing](https://www.getzola.org/documentation/content/image-processing/)
- [Themes](https://www.getzola.org/documentation/themes/overview/)
- [Shortcodes](https://www.getzola.org/documentation/content/shortcodes/)
- [Internal links](https://www.getzola.org/documentation/content/linking/)
- [External link checker](https://www.getzola.org/documentation/getting-started/cli-usage/#check)
- [Table of contents automatic generation](https://www.getzola.org/documentation/content/table-of-contents/)
- Automatic header anchors
- [Aliases](https://www.getzola.org/documentation/content/page/#front-matter)
- [Pagination](https://www.getzola.org/documentation/templates/pagination/)
- [Custom taxonomies](https://www.getzola.org/documentation/templates/taxonomies/)
- [Search with no servers or any third parties involved](https://www.getzola.org/documentation/content/search/)
- [Live reload](https://www.getzola.org/documentation/getting-started/cli-usage/#serve)
- Deploy on many platforms easily: [Netlify](https://www.getzola.org/documentation/deployment/netlify/), [Vercel](https://www.getzola.org/documentation/deployment/vercel/), [Cloudflare Pages](https://www.getzola.org/documentation/deployment/cloudflare-pages/), etc

## License

This project contains code under multiple licenses.

Code introduced after version 0.22 is licensed under the EUPL-1.2.
Code that existed prior to commit 3c9131db0d203640b6d5619ca1f75ce1e0d49d8f remains licensed under the MIT License, including in later versions of the project.

See LICENSE and LICENSE-MIT for details.
