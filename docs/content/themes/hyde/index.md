
+++
title = "hyde"
description = "A classic blog theme"
template = "theme.html"
date = 2017-10-24T15:01:52+02:00

[extra]
created = 2019-04-06T11:27:43+02:00
updated = 2017-10-24T15:01:52+02:00
repository = "https://github.com/getzola/hyde"
homepage = "https://github.com/getzola/hyde"
minimum_version = "0.5.0"
license = "MIT"
demo = "https://zola-hyde.netlify.com"

[extra.author]
name = "Vincent Prouillet"
homepage = "https://www.vincentprouillet.com"
+++        

# hyde
Hyde is a brazen two-column [Zola](https://github.com/getzola/zola) based on the Jekyll theme of the same name that pairs a prominent sidebar with uncomplicated content.

![Hyde screenshot](https://f.cloud.github.com/assets/98681/1831228/42af6c6a-7384-11e3-98fb-e0b923ee0468.png)


## Contents

- [Installation](#installation)
- [Options](#options)
  - [Sidebar menu](#sidebar-menu)
  - [Sticky sidebar content](#sticky-sidebar-content)
  - [Themes](#themes)
  - [Reverse layout](#reverse-layout)

## Installation
First download this theme to your `themes` directory:

```bash
$ cd themes
$ git clone https://github.com/getzola/hyde.git
```
and then enable it in your `config.toml`:

```toml
theme = "hyde"
```

## Options

### Sidebar menu
Set a field in `extra` with a key of `hyde_links`:
```toml
[extra]
hyde_links = [
    {url = "https://google.com", name = "Google.com"},
    {url = "https://google.fr", name = "Google.fr"},
]
```
Each link needs to have a `url` and a `name`.

### Sticky sidebar content
By default Hyde ships with a sidebar that affixes it's content to the bottom of the sidebar. You can optionally disable this by setting `hyde_sticky` to false in your `config.toml`.

### Themes
Hyde ships with eight optional themes based on the [base16 color scheme](https://github.com/chriskempson/base16). Apply a theme to change the color scheme (mostly applies to sidebar and links).

![Hyde in red](https://f.cloud.github.com/assets/98681/1831229/42b0b354-7384-11e3-8462-31b8df193fe5.png)

There are eight themes available at this time.

![Hyde theme classes](https://f.cloud.github.com/assets/98681/1817044/e5b0ec06-6f68-11e3-83d7-acd1942797a1.png)

To use a theme, set the `hyde_theme` field in `config.toml` to any of the themes name:

```toml
[extra]
hyde_theme = "theme-base-08"
```

To create your own theme, look to the Themes section of [included CSS file](https://github.com/poole/hyde/blob/master/public/css/hyde.css). Copy any existing theme (they're only a few lines of CSS), rename it, and change the provided colors.

### Reverse layout

![Hyde with reverse layout](https://f.cloud.github.com/assets/98681/1831230/42b0d3ac-7384-11e3-8d54-2065afd03f9e.png)

Hyde's page orientation can be reversed by setting `hyde_reverse` to `true` in the `config.toml`.

        