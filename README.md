# zola (né Gutenberg)
[![Build Status](https://travis-ci.com/getzola/zola.svg?branch=master)](https://travis-ci.com/getzola/zola)
[![Build status](https://ci.appveyor.com/api/projects/status/i0ufvx2sdm2cmawo/branch/master?svg=true)](https://ci.appveyor.com/project/Keats/zola/branch/master)

A fast static site generator in a single binary with everything built-in.

Documentation is available on [its site](https://www.getzola.org/documentation/getting-started/installation/) or
in the `docs/content` folder of the repository.

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
| Custom ouput formats            |     ✕     |    ✕   |   ✔  |    ?    |


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

## Contributing
As the documentation site is automatically built on commits to master, all development
should happen on the `next` branch, unless it is fixing the current documentation.

If you want a feature added or modified, please open an issue to discuss it before doing a PR.

### Adding syntax highlighting languages and themes

#### Adding a syntax
Syntax highlighting depends on submodules so ensure you load them first:

```bash
$ git submodule update --init
```

Zola only works with syntaxes in the `.sublime-syntax` format. If your syntax
is in `.tmLanguage` format, open it in Sublime Text and convert it to `sublime-syntax` by clicking on
Tools > Developer > New Syntax from ... and put it at the root of `sublime_syntaxes`.

You can also add a submodule to the repository of the wanted syntax:

```bash
$ cd sublime_syntaxes
$ git submodule add https://github.com/elm-community/SublimeElmLanguageSupport
```

Note that you can also only copy manually the updated syntax definition file but this means
Zola won't be able to automatically update it.

You can check for any updates to the current packages by running:

```bash
$ git submodule update --remote --merge
```

And finally from the root of the components/config crate run the following command:

```bash
$ cargo run --example generate_sublime synpack ../../sublime_syntaxes ../../sublime_syntaxes/newlines.packdump
```

#### Adding a theme
A gallery containing lots of themes is located at https://tmtheme-editor.herokuapp.com/#!/editor/theme/Agola%20Dark.
More themes can be easily added to Zola, just make a PR with the wanted theme added in the `sublime_themes` directory
and run the following command from the root of the components/config:

```bash
$ cargo run --example generate_sublime themepack ../../sublime_themes ../../sublime_themes/all.themedump
```

You should see the list of themes being added.
