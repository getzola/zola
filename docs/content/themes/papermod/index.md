
+++
title = "PaperMod"
description = "A fast, clean, responsive theme ported to Zola."
template = "theme.html"
date = 2025-01-07T10:04:32Z

[taxonomies]
theme-tags = []

[extra]
created = 2025-01-07T10:04:32Z
updated = 2025-01-07T10:04:32Z
repository = "https://github.com/cydave/zola-theme-papermod.git"
homepage = "https://github.com/cydave/zola-theme-papermod"
minimum_version = "0.4.0"
license = "MIT"
demo = "https://cydave.github.io/zola-theme-papermod/"

[extra.author]
name = "cydave"
homepage = "https://github.com/cydave"
+++        

# Zola PaperMod

![](screenshot.png)


A work in progress port of the [hugo-PaperMod](https://github.com/adityatelange/hugo-PaperMod) theme by [@adityatelange](https://github.com/adityatelange) to [Zola](https://www.getzola.org/) 

Due to config changes introduced with Zola 0.19, only Zola 0.19.1 and later are currently supported.

Demo @ https://cydave.github.io/zola-theme-papermod/


## Features

+ [x] Blog post archive
+ [x] Blog post RSS feeds
+ [x] Tags
+ [x] Tag-based RSS feeds
+ [x] Optional: Custom taxonomies
+ [x] Light / Dark theme switching (with configurable default preference)
+ [x] Syntax highlighting for code snippets (Zola's built-in syntax highlighting)
+ [x] Custom navigation
+ [ ] 3 Modes:
    + [ ] Regular Mode
    + [ ] Home-Info Mode
    + [ ] Profile Mode
+ [x] Code copy buttons
+ [x] Search page
+ [ ] SEO Metadata
+ [ ] Language switcher (multi-language support)


## Installation

1. Download the Theme

```
git submodule add https://github.com/cydave/zola-theme-papermod themes/papermod
```

2. Add `theme = "papermod"` to your zola `config.toml`
3. Copy over the example content to get started

```
cp -r themes/papermod/content content
```


## Options

Papermod customizations exist under a designated `extra.papermod` section.
Refer to [config.toml](config.toml) for available options.


## Contributing

If you would like to help out porting hugo-Papermod to Zola feel free to pick
up a feature and start working on it. All help, no matter how small the
contribution is highly appreciated.

        