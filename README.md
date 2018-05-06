# Gutenberg
[![Build Status](https://travis-ci.org/Keats/gutenberg.svg?branch=master)](https://travis-ci.org/Keats/gutenberg)
[![Build status](https://ci.appveyor.com/api/projects/status/h4t9r6h5gom839q0/branch/master?svg=true)](https://ci.appveyor.com/project/Keats/gutenberg/branch/master)

A fast static site generator in a single binary with everything built-in.

Documentation is available on [its site](https://www.getgutenberg.io/documentation/getting-started/installation/) or
in the `docs/content` folder of the repository.

## Comparisons with other static site generators

|                          | Gutenberg | Cobalt | Hugo | Pelican |
|--------------------------|-----------|--------|------|---------|
| Single binary            |     ✔     |    ✔   |   ✔  |    ✕    |
| Language                 |    Rust   |  Rust  |  Go  |  Python |
| Syntax highlighting      |     ✔     |    ✔   |   ✔  |    ✔    |
| Sass compilation         |     ✔     |    ✕   |   ✕  |    ✔    |
| Assets co-location       |     ✔     |    ✔   |   ✔  |    ✔    |
| i18n                     |     ✕     |    ✕   |   ✔  |    ✔    |
| Image processing         |     ✕     |    ✕   |   ✔  |    ✔    |
| Sane template engine     |     ✔     |    ✔   |  ✕✕✕ |    ✔    |
| Themes                   |     ✔     |    ✕   |   ✔  |    ✔    |
| Shortcodes               |     ✔     |    ✕   |   ✔  |    ✔    |
| Internal links           |     ✔     |    ✕   |   ✔  |    ✔    |
| Table of contents        |     ✔     |    ✕   |   ✔  |    ✔    |
| Automatic header anchors |     ✔     |    ✕   |   ✔  |    ✔    |
| Aliases                  |     ✔     |    ✕   |   ✔  |    ✔    |
| Pagination               |     ✔     |    ✕   |   ✔  |    ✔    |
| Custom taxonomies        |     ✕     |    ✕   |   ✔  |    ✕    |
| Search                   |     ✔     |    ✕   |   ✕  |    ✔    |
| Data files               |     ✕     |    ✔   |   ✔  |    ✕    |

Supported content formats:

- Gutenberg: markdown
- Cobalt: markdown
- Hugo: markdown, asciidoc, org-mode
- Pelican: reStructuredText, markdown, asciidoc, org-mode, whatever-you-want

Note that many features of Pelican are coming from plugins, which might be tricky
to use because of version mismatch or lacking documentation.

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

Gutenberg only works with syntaxes in the `.sublime-syntax` format. If your syntax
is in `.tmLanguage` format, open it in Sublime Text and convert it to `sublime-syntax` by clicking on
Tools > Developer > New Syntax from ... and put it at the root of `sublime_syntaxes`.

You can also add a submodule to the repository of the wanted syntax:

```bash
$ cd sublime_syntaxes
$ git submodule add https://github.com/elm-community/SublimeElmLanguageSupport
```

Note that you can also only copy manually the updated syntax definition file but this means
Gutenberg won't be able to automatically update it.

You can check for any updates to the current packages by running:

```bash
$ git submodule update --remote --merge
```

And finally from the root of the components/highlighting crate run the following command:

```bash
$ cargo run --example generate_sublime synpack ../../sublime_syntaxes ../../sublime_syntaxes/newlines.packdump ../../sublime_syntaxes/nonewlines.packdump
```

#### Adding a theme
A gallery containing lots of themes is located at https://tmtheme-editor.herokuapp.com/#!/editor/theme/Agola%20Dark.
More themes can be easily added to gutenberg, just make a PR with the wanted theme added in the `sublime_themes` directory
and run the following command from the root of the components/rendering:

```bash
$ cargo run --example generate_sublime themepack ../../sublime_themes ../../sublime_themes/all.themedump
```

You should see the list of themes being added.
