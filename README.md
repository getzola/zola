# zola (né Gutenberg)

A fast static site generator in a single binary with everything built-in.

**zola version:** 0.13.0 ([CHANGELOG](CHANGELOG.md))

**Test results:** [![Build Status](https://dev.azure.com/getzola/zola/_apis/build/status/getzola.zola?branchName=master)](https://dev.azure.com/getzola/zola/_build/latest?definitionId=1&branchName=master)

[**Minimum Supported Rust Version**](azure-pipelines.yml#L24)

## Intro

zola, previously known as gutenberg, is a simple and fast static site generator written in Rust. It aims to do one thing and do it well, with everything you need built-in. Documentation is available on [getzola.org](https://www.getzola.org/documentation/getting-started/installation/) or in the `docs/content` folder. A community forum for support, ideas and feedback is available at [zola.discourse.group](https://zola.discourse.group).

Zola provides services to many people already (see [EXAMPLES.md](EXAMPLES.md)), but has not been stabilized yet. This doesn't mean that zola itself is unstable and will crash randomly, but rather than configuration and usage patterns will continue to change for the foreseeable future in order to make zola more useful and more robust.

Feature requests are always welcome, although we carefully consider each new feature for its perceived usefulness, relative to the complexity for implementation/maintaining. Pull requests for new features (patches) are also welcome, but we encourage contributors to first start discussions on [the forums](https://zola.discourse.group/) to collect user feedback on their ideas.

Bug reports and patches for those are welcome at any time on our repository. If something is getting in your way or otherwise painful to use, it's probably a bug for other people as well, so do not hesitate to report it, so it can be fixed for everyone once and for all.

## Demo

[getzola.org](https://www.getzola.org/) is entirely produced with zola. Please head over to [EXAMPLES.md](EXAMPLES.md) for more example sites produced with zola.

## Comparisons with other static site generators

zola may or may not be the right tool for you. If you would like to see at a glance the pro's and con's of using Zola, here you will find some tables comparing it with [Cobalt](https://cobalt-org.github.io/), [Hugo](https://gohugo.io/) and [Pelican](https://blog.getpelican.com/). This comparison tries to be unbiased (factual), however some information is missing, and we are of course more familiar with the support of certain features in Zola (compared to other tools), so feedback and patches are welcome to improve it.

**Note:** Many features of Pelican come from plugins, which might be tricky to use because of a version mismatch or inadequate documentation. Netlify supports Python and Pipenv but you still need to install your dependencies manually.

### High-level overview

|                                 | [Zola](https://getzola.org/)   | [Cobalt](https://cobalt-org.github.io/) | [Hugo](https://gohugo.io/)   | [Pelican](https://blog.getpelican.com/) |
|:--------------------------------|:------:|:------:|:------:|:-------:|
| Single binary                   | ![yes] | ![yes] | ![yes] | ![no]   |
| Language                        | Rust   | Rust   | Go     | Python  |
| Syntax highlighting             | [![yes]](https://www.getzola.org/documentation/content/syntax-highlighting/) | ![yes] | ![yes] | ![yes]  |
| Sass compilation                | [![yes]](https://www.getzola.org/documentation/content/sass/) | ![yes] | ![yes] | ![yes]  |
| Multilingual site               | [![ehh]](https://www.getzola.org/documentation/content/multilingual/) | ![no]  | ![yes] | ![yes]  |
| Image processing                | [![yes]](https://www.getzola.org/documentation/content/image-processing/) | ![no]  | ![yes] | ![yes]  |
| Link checker                    | [![yes]](https://www.getzola.org/documentation/getting-started/cli-usage/#check) | ![no]  | ![no]  | ![yes]  |
| Search                          | [![ehh]](https://www.getzola.org/documentation/content/search/) | ![no]  | ![no]  | ![yes]  |
| Data files                      | [![yes]](https://www.getzola.org/documentation/templates/overview/#load-data) | ![yes] | ![yes] | ![no]   |
| LiveReload                      | [![yes]](https://www.getzola.org/documentation/getting-started/cli-usage/#serve) | ![no]  | ![yes] | ![yes]  |

**Notes:**

- *Multilingual site*
  - zola gets ![ehh] because despite having a basic translations system, it is being redesigned to evade its current limitations (see [discussion on the forum](https://zola.discourse.group/t/rfc-internationalization-system-rework/546))
- *Search*
  - zola gets ![ehh] because search engine is disabled by default in some languages, as explained [in the docs](https://www.getzola.org/documentation/content/multilingual/#configuration)

### Content library

|                                 | [Zola](https://getzola.org/)   | [Cobalt](https://cobalt-org.github.io/) | [Hugo](https://gohugo.io/)   | [Pelican](https://blog.getpelican.com/) |
|:--------------------------------|:------:|:------:|:------:|:-------:|
| Aliases                         | ![yes] | ![no]  | ![yes] | ![yes]  |
| Assets co-location              | [![ehh]](https://www.getzola.org/documentation/content/overview/#asset-colocation) | ![yes] | ![yes] | ![yes]  |
| HTML shortcodes                 | [![yes]](https://www.getzola.org/documentation/content/shortcodes/) | ![no]  | ![yes] | ![yes]  |
| Format-specific shortcodes      | [![yes]](https://www.getzola.org/documentation/content/shortcodes/) |   ?    |    ?   |    ?    |
| Internal links                  | [![ehh]](https://www.getzola.org/documentation/content/linking/) | ![no]  | ![yes] | ![yes]  |
| Warn or error for broken links  | [![no]](https://github.com/getzola/zola/issues/977#issuecomment-759725671)  | ![no]  | [![yes]](https://gohugo.io/content-management/cross-references/) | ? |
| Custom taxonomies               | [![yes]](https://www.getzola.org/documentation/content/taxonomies/) | ![no]  | ![yes] | ![no]   |
| Multiple static mounts          | [![no]](https://github.com/getzola/zola/issues/499) | ![no] | [![yes]](https://gohugo.io/hugo-modules/configuration/#module-config-mounts) | ? |

**Notes:**

- *Assets co-location*
  - zola receives ![ehh] because assets cannot be shared across pages/sections, despite living in the same folder ([discussion on the forum](https://zola.discourse.group/t/reusing-markdown-docs-from-github-repo-in-zola-site/776))
- *Internal links*
  - zola receives ![ehh] because so there is currently no reliable way to reference static assets from the content pages (though a shortcode can do it very easily), which is a limitation that only affects builds for a subfolder (see discussion about [path unification](https://github.com/getzola/zola/issues/977)) ; sites built for the webroot of a domain are unaffected by this limitation

The following input formats are supported:

|                      | [Zola](https://getzola.org/)   | [Cobalt](https://cobalt-org.github.io/) | [Hugo](https://gohugo.io/)   | [Pelican](https://blog.getpelican.com/) |
|:--------------------------------|:------:|:------:|:------:|:-------:|
| Markdown             | ![yes] | ![yes] | ![yes] | ![yes] |
| AsciiDoc             | [![no]](https://github.com/getzola/zola/issues/1160)  | ![no]  | ![yes] | ![yes] |
| org-mode             | [![no]](https://zola.discourse.group/t/support-for-org-mode/789)  | ![no]  | ![yes] | ![yes] |
| reStructuredText     | [![no]](https://zola.discourse.group/t/alternative-input-formats/76)  | ![no]  | ![no]  | ![yes] |
| extensible (plugins) | ![no]  | ![no]  | ![no]  | ![yes] |

### Output rendering

|                                 | [Zola](https://getzola.org/)   | [Cobalt](https://cobalt-org.github.io/) | [Hugo](https://gohugo.io/)   | [Pelican](https://blog.getpelican.com/) |
|:--------------------------------|:------:|:------:|:------:|:-------:|
| Breadcrumbs                     | ![yes] | ![no]  | ![no]  | ![yes]  |
| Automatic header anchors        | [![yes]](https://www.getzola.org/documentation/content/linking/#anchor-insertion) | ![no]  | ![yes] | ![yes]  |
| Pagination                      | [![yes]](https://www.getzola.org/documentation/templates/pagination/) | ![no]  | ![yes] | ![yes]  |
| Table of contents               | [![yes]](https://www.getzola.org/documentation/templates/pages-sections/#table-of-contents) | ![no]  | ![yes] | ![yes]  |
| Themes                          | [![yes]](https://www.getzola.org/documentation/themes/creating-a-theme/) | ![no]  | ![yes] | ![yes]  |
| Extensible themes               | [![yes]](https://www.getzola.org/documentation/themes/extending-a-theme) |   ?    |  [![yes]](https://gohugo.io/templates/base/)   |   [![yes]](https://docs.getpelican.com/en/latest/themes.html#inheritance)   |
| Sane & powerful template engine | [![yes]](https://www.getzola.org/documentation/templates/overview/) | ![yes] | ![ehh] | ![yes]  |
| Custom output formats           | [![no]](https://zola.discourse.group/t/proposal-custom-output-formats/68/4)  | ![no]  | ![yes] | ![no]   |
| URL templates                   | [![no]](https://github.com/getzola/zola/issues/635)  | ![no]  | [![yes]](https://gohugo.io/content-management/urls/#permalinks-configuration-example) | ? |
| UglyURLs                        | [![no]](https://github.com/getzola/zola/issues/840)  | ![no]  | [![yes]](https://gohugo.io/content-management/urls/#ugly-urls) | ? |
| Relative URLs                   | [![ehh]](https://github.com/getzola/zola/issues/711)  | ![no]  | [![yes]](https://gohugo.io/content-management/urls/#relative-urls) | ? |
| [Backlinks](https://en.wikipedia.org/wiki/Backlink)         | ![no]  | ![no]    | [![no]](https://github.com/gohugoio/hugo/issues/8077) | ? |
| Diagrams                        | [![no]](https://zola.discourse.group/t/diagramming-tool-integrations-plantuml-svgbob-graphviz-etc/269) | ![no] | [![no]](https://github.com/gohugoio/hugo/issues/7765) | [![yes]](https://github.com/getpelican/pelican-plugins/tree/master/plantuml) |

**Notes:**

- *Sane & powerful template engine*
  - hugo gets ![ehh] because golang templates are simply not meant for webdesign (despite being very powerful), to the point of driving @Keats insane to create his own template engine ([tera](https://github.com/keats/tera)) and static-site generator (zola) ; yes this is a little biased
- *Relative URLs*
  - zola gets ![ehh] because it can only produce relative URLs for the webroot, not a subfolder (see discussion about [path unification](https://github.com/getzola/zola/issues/711))

### Deployment integrations

While it's usually possible to use any kind of static site generator to deploy to a website, some platforms have restrictions and specific tooling to publish content. Here's some example documentation for deploying on specific platforms:

|                                     | [Zola](https://getzola.org/)   | [Cobalt](https://cobalt-org.github.io/) | [Hugo](https://gohugo.io/)   | [Pelican](https://blog.getpelican.com/) |
|:------------------------------------|:------:|:------:|:------:|:-------:|
| [Netlify](https://www.netlify.com/) |  [![yes]](https://www.getzola.org/documentation/deployment/netlify/)  |  ![no]   |  ![yes]  |   ![no]  |
| [Vercel](https://vercel.com/)       |  [![yes]](https://www.getzola.org/documentation/deployment/vercel/)  |  ![no]   |  ![yes]  |  ![yes]  |
| [SourceHut](https://srht.site/)    |  [![yes]](https://www.getzola.org/documentation/deployment/sourcehut/)  |    ?     |     ?    |     ?    |
| [Github](https://pages.github.com/) |  [![yes]](https://www.getzola.org/documentation/deployment/github-pages/) |   ?   | ![yes] | ![yes] |
| [Gitlab](https://docs.gitlab.com/ee/user/project/pages/) | [![yes]](https://www.getzola.org/documentation/deployment/gitlab-pages/) | ? | ![yes] | ![yes] |

## Contributing

If you would like to contribute to improve zola together as a community, there are many ways to do so. In this section, we outline a few ways you could help the zola project. If you're looking to make a contribution now, please read [CONTRIBUTING.md](CONTRIBUTING.md).

### Documentation and pro tips

When you notice something isn't clear in the documentation, you may submit a patch to improve it for every one else. If you're doing something which you think could be of interest to other folks, please write about it [on the forums](https://zola.discourse.group/). If you notice some usage patterns emerging on the forums, feel free to turn that into a documentation page to help other people achieve the same goals.

### Usability and architecture

Despite being three decades old, the world wide web isn't always the most convenient platform to use. zola tries to make your life better, but if you're experiencing difficulties with it, don't hesitate to report those. Please be clear about what was confusing or unexpected regarding what you tried to achieve.

zola doesn't shine for all use cases we can think of, and could use feedback in particular for:

- a better [translations system](https://zola.discourse.group/t/rfc-internationalization-system-rework/546)
- a [unified path](https://github.com/getzola/zola/issues/977) representation
- support for [non-Markdown input formats](https://zola.discourse.group/t/alternative-input-formats/76)
- support for [non-HTML output formats](https://zola.discourse.group/t/proposal-custom-output-formats/68/4)

### Translations

The documentation is currently only available in english, which is a shame because most of the world does not speak english. If you would like to help making zola available to more people, feel free to translate documentation in your language.

### Programming

Some areas of zola could be improved. If you're just getting started, we maintain a list of [good first issues](https://github.com/getzola/zola/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22) to get you started hacking on the codebase. If you'd like to tackle some harder problems, you can take a look at the ![ehh] and ![no] features from the comparison table. Additionally, we also have a list of [issues we'd like help on](https://github.com/getzola/zola/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22).

Some (upstream) projects we rely on to build zola could use some help:

- [tera](https://github.com/keats/tera/) rendering engine has some issues marked [`good first issue`](https://github.com/Keats/tera/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22) as well as some [`help wanted`](https://github.com/Keats/tera/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22)
- [pest](https://github.com/pest-parser/pest/issues/489#issuecomment-775114684) parser is [looking for new contributors](https://github.com/pest-parser/pest/issues/489#issuecomment-775114684)

### Packaging

zola is not yet packaged for all systems. An updated list of zola packages can be found [in the documentation](https://www.getzola.org/documentation/getting-started/installation/). Some platforms we would like to have packages for include:

- [Debian](https://debian.org/): zola is currently unavailable, even on `experimental` ; ideally, we would like to support [backports](https://backports.debian.org/) on `stable` as well, because zola is evolving fast
- [nix](https://nixos.org/): zola on nix is outdated (v0.12.0 as of writing this)
- [GNU guix](https://guix.gnu.org/): zola is currently unavailable in GNU guix

## License

[MIT license](LICENSE)


[yes]: ./is-yes.svg
[ehh]: ./is-ehh.svg
[no]:  ./is-no.svg


