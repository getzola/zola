# Contributing
**As the documentation site is automatically built on commits to master, all development happens on
the `next` branch, unless it is fixing the current documentation.**

However, if you notice an error or typo in the documentation, feel free to directly submit a PR without opening an issue.

## Feature requests
If you want a feature added or modified, please open a thread on the [forum](https://zola.discourse.group/) to discuss it before doing a PR.

Requested features will not be all added: an ever-increasing features set makes for a hard to use and explain softwares.
Having something simple and easy to use for 90% of the usecases is more interesting than covering 100% usecases after sacrificing simplicity.

## Issues tagging

As the development happens on the `next` branch, issues are kept open until a release containing the fix is out.
During that time, issues already resolved will have a `done` tag.

If you want to work on an issue, please mention it in a comment to avoid potential duplication of work. If you have
any questions on how to approach it do not hesitate to ping me (@keats).
Easy issues are tagged with `help wanted` and/or `good first issue`

## Adding syntax highlighting languages and themes

### Adding a syntax
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

### Adding a theme
A gallery containing lots of themes is located at https://tmtheme-editor.herokuapp.com/#!/editor/theme/Agola%20Dark.
More themes can be easily added to Zola, just make a PR with the wanted theme added in the `sublime_themes` directory
and run the following command from the root of the components/config:

```bash
$ cargo run --example generate_sublime themepack ../../sublime_themes ../../sublime_themes/all.themedump
```

You should see the list of themes being added.
