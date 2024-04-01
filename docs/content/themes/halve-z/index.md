
+++
title = "halve-z"
description = "Retro two-column theme"
template = "theme.html"
date = 2024-04-02T04:09:32+10:00

[extra]
created = 2024-04-02T04:09:32+10:00
updated = 2024-04-02T04:09:32+10:00
repository = "https://github.com/charlesrocket/halve-z"
homepage = "https://github.com/charlesrocket/halvez"
minimum_version = "0.18.0"
license = "MIT"
demo = "https://halve-z.netlify.app/"

[extra.author]
name = "-k"
homepage = "https://failsafe.monster/"
+++        

# halve-z
[![Netlify Status](https://api.netlify.com/api/v1/badges/352a12ed-cdba-4545-9256-9fb698f5a94f/deploy-status?branch=trunk)](https://app.netlify.com/sites/halve-z/deploys)

A two-column theme for **Zola**.

## Features

This is a _retro_ port of [Halve](https://github.com/TaylanTatli/Halve) for **Jekyll**. It features:

* taxonomies
* auto color schemes
* ToC
* media shortcodes
* SEO
* CSP
* project cards
* comments ([giscus](http://giscus.app))
* read time

## Installation

Add theme submodule using `git`:

```sh
git submodule add https://github.com/charlesrocket/halve-z themes/halve-z
```

## Configuration

1. Copy theme's [config.toml](https://github.com/charlesrocket/halve-z/blob/trunk/config.toml) in your project's root directory. Set variables as required and add `theme = "halve-z"` at the **top** of the config file.
2. Copy the content to get started:

```
cp -R -f themes/halve-z/content/ content/
```

## Usage

See demo [posts](https://halve-z.netlify.app/posts/).

        