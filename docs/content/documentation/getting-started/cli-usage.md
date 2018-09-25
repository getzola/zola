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
$ gutenberg init my_site
```

will create a new folder named `my_site` and the files/folders needed by
Gutenberg.

## build

This will build the whole site in the `public` directory.

```bash
$ gutenberg build
```

You can override the config `base_url` by passing a new URL to the `base-url` flag.

```bash
$ gutenberg build --base-url $DEPLOY_URL
```

This is useful for example when you want to deploy previews of a site to a dynamic URL, such as Netlify
deploy previews.

+You can override the default output directory 'public' by passing a other value to the `output-dir` flag.

```bash
$ gutenberg build --output-dir $DOCUMENT_ROOT
```

You can also point to another config file than `config.toml` like so - the position of the `config` option is important:

```bash
$ gutenberg --config config.staging.toml build
```

## serve

This will build and serve the site using a local server. You can also specify
the interface/port combination to use if you want something different than the default (`127.0.0.1:1111`).

You can also specify different addresses for the interface and base_url using `-u`/`--base-url`, for example
if you are running Gutenberg in a Docker container.

```bash
$ gutenberg serve
$ gutenberg serve --port 2000
$ gutenberg serve --interface 0.0.0.0
$ gutenberg serve --interface 0.0.0.0 --port 2000
$ gutenberg serve --interface 0.0.0.0 --base-url 127.0.0.1
$ gutenberg serve --interface 0.0.0.0 --port 2000 --output-dir www/public
```

The serve command will watch all your content and will provide live reload, without
hard refresh if possible.

Gutenberg does a best-effort to live reload but some changes cannot be handled automatically. If you
fail to see your change or get a weird error, try to restart `gutenberg serve`.


You can also point to another config file than `config.toml` like so - the position of the `config` option is important:

```bash
$ gutenberg --config config.staging.toml serve
```

## Colored output

Any of the three commands will emit colored output if your terminal supports it.

*Note*: coloring is automatically disabled when the output is redirected to a pipe or a file (ie. when the standard output is not a TTY).

You can disable this behavior by exporting one of the two following environment variables:

- `NO_COLOR` (the value does not matter)
- `CLICOLOR=0`

Should you want to force the use of colors, you can set the following environment variable:

- `CLICOLOR_FORCE=1`
