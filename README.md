# Gutenberg
[![Build Status](https://travis-ci.org/Keats/gutenberg.svg?branch=master)](https://travis-ci.org/Keats/gutenberg)
[![Build status](https://ci.appveyor.com/api/projects/status/h4t9r6h5gom839q0/branch/master?svg=true)](https://ci.appveyor.com/project/Keats/gutenberg/branch/master)

An opinionated static site generator written in Rust.

Documentation is available on [its site](https://www.getgutenberg.io/documentation/getting-started/installation/) or
in the `docs/content` folder of the repository.

## Example sites

- [vincent.is](https://vincent.is): https://gitlab.com/Keats/vincent.is
- [code<future](http://www.codelessfuture.com/)
- http://t-rex.tileserver.ch (https://github.com/pka/t-rex-website/)
- [adrien.is](https://adrien.is): https://github.com/Fandekasp/fandekasp.github.io
- [Philipp Oppermann's blog](https://os.phil-opp.com/): https://github.com/phil-opp/blog_os/tree/master/blog
- [seventeencups](https://www.seventeencups.net): https://github.com/17cupsofcoffee/seventeencups.net

## Adding syntax highlighting languages and themes

### Adding a syntax
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
$ git submodule add https://github.com/elm-community/Elm.tmLanguage.git
```

Note that you can also only copy manually the updated syntax definition file but this means
Gutenberg won't be able to automatically update it.

You can check for any updates to the current packages by running:

```bash
$ git submodule update --remote --merge
```

And finally from the root of the components/rendering crate run the following command:

```bash
$ cargo run --example generate_sublime synpack ../../sublime_syntaxes ../../sublime_syntaxes/newlines.packdump ../../sublime_syntaxes/nonewlines.packdump
```

### Adding a theme
A gallery containing lots of themes at https://tmtheme-editor.herokuapp.com/#!/editor/theme/Agola%20Dark.
More themes can be easily added to gutenberg, just make a PR with the wanted theme added in the `sublime_themes` directory
and run the following command from the root of the components/rendering:

```bash
$ cargo run --example generate_sublime themepack ../../sublime_themes ../../sublime_themes/all.themedump
```

You should see the list of themes being added.
