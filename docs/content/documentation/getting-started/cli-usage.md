+++
title = "CLI usage"
weight = 15
+++

Zola only has 4 commands: `init`, `build`, `serve` and `check`.

You can view the help for the whole program by running `zola --help` and
that for a specific command by running `zola <cmd> --help`.

## init

Creates the directory structure used by Zola at the given directory after asking a few basic configuration questions.
Any choices made during these prompts can be easily changed by modifying `config.toml`.

```bash
$ zola init my_site
$ zola init
```

If the `my_site` directory already exists, Zola will only populate it if it contains only hidden files (dotfiles are ignored). If no `my_site` argument is passed, Zola will try to populate the current directory.

In case you want to attempt to populate a non-empty directory and are brave, you can use `zola init --force`. Note that this will _not_ overwrite existing folders or files; in those cases you will get a `File exists (os error 17)` error or similar.

You can initialize a git repository and a Zola site directly from within a new folder:

```bash
$ git init
$ zola init
```

## build

This will build the whole site in the `public` directory (if this directory already exists, it is deleted).

```bash
$ zola build
```

You can override the config `base_url` by passing a new URL to the `base-url` flag.

```bash
$ zola build --base-url $DEPLOY_URL
```

This is useful for example when you want to deploy previews of a site to a dynamic URL, such as Netlify
deploy previews.

You can override the default output directory `public` by passing another value to the `output-dir` flag. If this directory already exists, the user will be prompted whether to replace the folder; you can override this prompt by passing the --force flag.

```bash
$ zola build --output-dir $DOCUMENT_ROOT
```

You can point to a config file other than `config.toml` like so (note that the position of the `config` option is important):

```bash
$ zola --config config.staging.toml build
```

You can also process a project from a different directory with the `root` flag. If building a project 'out-of-tree' with the `root` flag, you may want to combine it with the `output-dir` flag. (Note that like `config`, the position is important):
```bash
$ zola --root /path/to/project build
```

By default, drafts are not loaded. If you wish to include them, pass the `--drafts` flag.

## serve

This will build and serve the site using a local server. You can also specify
the interface/port combination to use if you want something different than the default (`127.0.0.1:1111`).

You can also specify different addresses for the interface and base_url using `--interface` and `-u`/`--base-url`, respectively, if for example you are running Zola in a Docker container.

> By default, devices from the local network **won't** be able to access the served pages. This may be of importance when you want to test page interaction and layout on your mobile device or tablet. If you set the interface to `0.0.0.0` however, devices from your local network will be able to access the served pages by requesting the local ip-address of the machine serving the pages and port used.
>
> In order to have everything work correctly, you might also have to alter the `base-url` flag to your local ip or set it to `/` to use server-base relative paths.

Use the `--open` flag to automatically open the locally hosted instance in your
web browser.

Before starting, Zola will delete the output directory (by default `public` in project root) to start from a clean slate.

If you are specifying the directory but are also using the `output-dir` flag, Zola will not use the specified directory if it already exists unless the --force flag is used.

```bash
$ zola serve
$ zola serve --port 2000
$ zola serve --interface 0.0.0.0
$ zola serve --interface 0.0.0.0 --port 2000
$ zola serve --interface 0.0.0.0 --base-url 127.0.0.1
$ zola serve --interface 0.0.0.0 --base-url /
$ zola serve --interface 0.0.0.0 --port 2000 --output-dir www/public
$ zola serve --open
```

The serve command will watch all your content and provide live reload without
a hard refresh if possible. If you are using WSL2 on Windows, make sure to store the website on the WSL file system.

Some changes cannot be handled automatically and thus live reload may not always work. If you
fail to see your change or get an error, try restarting `zola serve`.

You can also point to a config file other than `config.toml` like so (note that the position of the `config` option is important):

```bash
$ zola --config config.staging.toml serve
```

By default, drafts are not loaded. If you wish to include them, pass the `--drafts` flag.

## check

The check subcommand will try to build all pages just like the build command would, but without writing any of the
results to disk. Additionally, it will also check all external links in Markdown files by trying to fetch
them (links in the template files are not checked).

By default, drafts are not loaded. If you wish to include them, pass the `--drafts` flag.

## Colored output

Colored output is used if your terminal supports it.

*Note*: coloring is automatically disabled when the output is redirected to a pipe or a file (i.e., when the standard output is not a TTY).

You can disable this behavior by exporting one of the following two environment variables:

- `NO_COLOR` (the value does not matter)
- `CLICOLOR=0`

To force the use of colors, you can set the following environment variable:

- `CLICOLOR_FORCE=1`
