
+++
title = "simplify"
description = "A minimal blog theme built with simple.css"
template = "theme.html"
date = 2022-04-05T18:27:13-07:00

[extra]
created = 2022-04-05T18:27:13-07:00
updated = 2022-04-05T18:27:13-07:00
repository = "https://github.com/tarunjana/simplify.git"
homepage = "https://github.com/tarunjana/simplify"
minimum_version = "0.15.3"
license = "MIT"
demo = "https://simplify-zola.netlify.app"

[extra.author]
name = "Tarun Jana"
homepage = "https://www.tarunjana.in/"
+++        

Simplify is a minimal [Zola](https://www.getzola.org/) theme built with
[Simple.css](https://simplecss.org/).

## Demo

To have a taste of what Simplify is, please click [here](https://simplify-zola.netlify.app).

## Screenshot

![Screenshot](/screenshot.png)

## Installation

Install Zola in your machine as described in the [official docs](https://www.getzola.org/documentation/getting-started/installation/) and follow the steps below to use Simplify theme in your site.

1. Create a new Zola site (if you don't have a Zola site already):

```bash
zola init my-website
```

2. Go to your site root:

```bash
cd my-website
```

3. Initialize an empty git repository:

```bash
git init
```

4. Add simplify theme as a git submodule:

```bash
git submodule add https://github.com/tarunjana/simplify.git themes/simplify
```

5. Add the theme in your `config.toml`:

```toml
theme = "simplify"
```

## Features

1. Auto dark/light mode according to system preference
2. Inject anything in the `<head>...</head>` tag.
3. Math typesetting with KaTeX.

## Documentation

Please see the [wiki](https://github.com/tarunjana/simplify/wiki).

## Credit

This theme is the product of some awesome projects listed below:

- [Zola](https://www.getzola.org/)
- [Simple.css](https://simplecss.org/)
- [KaTeX](https://katex.org/)

## License

[MIT](https://mit-license.org)
        