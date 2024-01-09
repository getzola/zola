
+++
title = "ntun-zola-theme"
description = "A classic resume theme"
template = "theme.html"
date = 2024-01-09T08:33:22+01:00

[extra]
created = 2024-01-09T08:33:22+01:00
updated = 2024-01-09T08:33:22+01:00
repository = "https://github.com/Netoun/ntun.git"
homepage = "https://github.com/netoun/ntun"
minimum_version = "0.1.0"
license = "MIT"
demo = "https://netoun.github.io/ntun/"

[extra.author]
name = "Nicolas Coulonnier"
homepage = "https://netoun.com"
+++        

# **Ntun**

![alt text](screenshot.png "Screenshoot")

**Live demo** : https://netoun.github.io/ntun/
## Contents

- Installation
- Options

## Installation
First download this theme to your `themes` directory:

```bash
cd themes
git clone https://github.com/netoun/ntun.git
```
and then enable it in your `config.toml`:

```toml
theme = "ntun"
```

This theme requires index section in `about` (`content/about/_index.md`)

The posts should therefore be in directly under the `content about` folder.

## Options

Set a field in `extra` with a key of `after_dark_menu`:

```toml
[extra]
author = "Jon Snow"
author_image="me.jpg"
city="Winterfell"
years="281"

job = "King of the north"
description = "Dragons & Aunt ❤️"

links = [
    { url = "", title="", icon = "fab fa-github"},
    { url = "", title="", icon = "fab fa-twitter"},
    { url = "", title="", icon = "fab fa-linkedin"},
    { url = "mailto:", title="", icon = "fas fa-envelope"}
]

# if you add languages, put your emoji flag on array
languages_flags = [
    "🇬🇧"
]
```

If you put `$BASE_URL` in a url, it will automatically be replaced by the actual
site URL.

        