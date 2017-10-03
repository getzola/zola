+++
title = "Installing & using themes"
weight = 20
+++


## Installing a theme

The easiest way to install to theme is to clone its repository in the `themes`
directory.

```bash
$ cd themes
$ git clone THEME_REPO_URL
```

Cloning the repository using Git or another VCS will allow you to easily
update it but you can also simply download the files manually and paste
them in a folder.

## Using a theme

Now that you have the theme in your `themes` directory, you only need to tell
Gutenberg to use it to get started by setting the `theme` variable of the 
[configuration file](./documentation/getting-started/configuration.md). The theme
name has to be name of the directory you cloned the theme in.
For example, if you cloned a theme in `templates/simple-blog`, the theme name to use
in the configuration file is `simple-blog`.

## Customizing a theme

Any file from the theme can be overriden by creating a file with the same path and name in your `templates` or `static`
directory. Here are a few examples of that, assuming the theme name is `simple-blog`:

```plain
templates/pages/post.html -> replace themes/simple-blog/pages/post.html
templates/macros.html -> replace themes/simple-blog/macros.html
static/js/site.js -> replace themes/simple-blog/static/js/site.jss
```

Most themes will also provide some variables that are meant to be overriden: this happens in the `extra` section
of the [configuration file](./documentation/getting-started/configuration.md). 
Let's say a theme uses a `show_twitter` variable and sets it to `false` by default. If you want to set it to `true`, 
you can update your `config.toml` like so:

```toml
[extra]
show_twitter = false
```

You can modify files directly in the `themes` directory but this will make updating the theme harder and live reload won't work with those
files.
