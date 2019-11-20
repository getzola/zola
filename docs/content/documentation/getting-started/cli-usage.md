+++
title = "CLI usage"
weight = 2
+++

Zola only has 4 commands: `init`, `build`, `serve` and `check`.

You can view the help of the whole program by running `zola --help` and
the command help by running `zola <cmd> --help`.

## init

Creates the directory structure used by Zola at the given directory after asking a few basic configuration questions.
Any choices made during these prompts can easily be changed by modifying `config.toml`.

```bash
$ zola init my_site
$ zola init
```

If the `my_site` folder already exists, Zola will only populate it if it does not contain non-hidden files (dotfiles are ignored). If no `my_site` argument is passed, Zola will try to populate the current directory.

You can initialize a git repository and a Zola site directly from within a new folder:

```bash
$ git init
$ zola init
```

## build

This will build the whole site in the `public` directory (original content is deleted).

```bash
$ zola build
```

You can override the config `base_url` by passing a new URL to the `base-url` flag.

```bash
$ zola build --base-url $DEPLOY_URL
```

This is useful for example when you want to deploy previews of a site to a dynamic URL, such as Netlify
deploy previews.

You can override the default output directory 'public' by passing another value to the `output-dir` flag.

```bash
$ zola build --output-dir $DOCUMENT_ROOT
```

You can also point to a config file other than `config.toml` like so (note that the position of the `config` flag is important):

```bash
$ zola --config config.staging.toml build
```

By default, drafts are not loaded. If you wish to include them, pass the `--drafts` flag.

## serve

This will build and serve the site using a local server. You can also specify
the interface/port combination to use if you want something different than the default (`127.0.0.1:1111`).

You can also specify different addresses for the interface and base_url using `--interface` and `--base-url` (or `-u`), respectively, if for example you are running Zola in a Docker container.

Use the `--open` flag to automatically open the locally hosted instance in your
web browser.

In the event you don't want Zola to run a local webserver, you can use the `--watch-only` flag.

Before starting, Zola deletes the 'public' directory to ensure that it starts from a clean slate.

```bash
$ zola serve
$ zola serve --port 2000
$ zola serve --interface 0.0.0.0
$ zola serve --interface 0.0.0.0 --port 2000
$ zola serve --interface 0.0.0.0 --base-url 127.0.0.1
$ zola serve --interface 0.0.0.0 --port 2000 --output-dir www/public
$ zola serve --watch-only
$ zola serve --open
```

The serve command will watch all your content and provide live reload without
a hard refresh if possible.

Some changes cannot be handled automatically and thus live reload may not always work. If you
fail to see your changes in the browser or get an error, try to restart `zola serve`.


You can also point to a config file other than `config.toml` like so (note that the position of the `config` option is important):

```bash
$ zola --config config.staging.toml serve
```

By default, drafts are not loaded. If you wish to include them, pass the `--drafts` flag.

### check

The check subcommand will try to build all pages just like the build command would, but without writing any of the
results to disk. Additionally, it will also check all external links in Markdown files by trying to fetch
them (links in template files are not checked).

By default, drafts are not loaded. If you wish to include them, pass the `--drafts` flag.

## Colored output

The following three commands control colored output if your terminal supports it.

*Note*: coloring is automatically disabled when the output is redirected to a pipe or a file (i.e., when the standard output is not a TTY).

You can disable this behavior by exporting one of the two following environment variables:

- `NO_COLOR` (the value does not matter)
- `CLICOLOR=0`

Should you want to force the use of colors, you can set the following environment variable:

- `CLICOLOR_FORCE=1`
