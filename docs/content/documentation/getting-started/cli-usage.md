+++
title = "CLI usage"
weight = 2
+++

Gutenberg only has 3 commands: init, build and serve.

You can view the help of the whole program by running `gutenberg --help` and
the command help by running `gutenberg <cmd> --help`.

## init

Creates the directory structure used by Gutenberg at the given directory.

```bash
$ gutenberg init <my_site>
```

will create a new folder named `my_site` and the files/folders needed by
Gutenberg.

## build

This will build the whole site in the `public` directory.

```bash
$ gutenberg build
```

## serve

This will build and serve the site using a local server. You can also specify
the interface/port combination to use if you want something different than the default (`127.0.0.1:1111`).

```bash
$ gutenberg serve
$ gutenberg serve --port 2000
$ gutenberg serve --interface 0.0.0.0 
$ gutenberg serve --interface 0.0.0.0 --port 2000
```

The serve command will watch all your content and will provide live reload, without
hard refresh if possible.

If you fail to see your change, this means that Gutenberg couldn't reload that bit and you will
need to restart `gutenberg serve`.
