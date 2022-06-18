
+++
title = "archie-zola"
description = "A zola theme based on Hugo archie."
template = "theme.html"
date = 2022-06-15T01:35:37-07:00

[extra]
created = 2022-06-15T01:35:37-07:00
updated = 2022-06-15T01:35:37-07:00
repository = "https://github.com/XXXMrG/archie-zola.git"
homepage = "https://github.com/XXXMrG/archie-zola"
minimum_version = "0.14.0"
license = "MIT"
demo = "https://archie-zola.vercel.app/"

[extra.author]
name = "Keith"
homepage = "https://github.com/XXXMrG"
+++        

# archie-zola


A zola theme forked from [https://github.com/athul/archie](https://github.com/athul/archie)

## Demo


The Main branch source code hosted on [https://archie-zola.vercel.app](https://archie-zola.vercel.app)

### ScreenShot

![screenshot-light](https://archie-zola.vercel.app/screenshot/screenshot-light.png)

![screenshot-dark](https://archie-zola.vercel.app/screenshot/screenshot-dark.png)


## Installation


First download this theme to your themes directory:

```bash
cd themes
git clone https://github.com/XXXMrG/archie-zola.git
```

or add as a git submodule:

```bash
git submodule add https://github.com/XXXMrG/archie-zola.git  themes/archie-zola
```

and then enable it in your config.toml:
```toml
theme = "archie-zola"
```

## Feature

* Pagination
* Tags
* Auto Dark Mode(based on system theme)
* Dark/Light Mode toggle
* Google Analytics Script

in the planning stage：

- [ ] Custom CSS & JS
- [ ] Twitter Cards & Youtube video


## Config


### Theme config

Cause Zola limited custom config must under the `extra` field, so there are some different with the origin theme:

Demo website config.toml:

```toml
# control dark mode: auto | dark | toggle
mode = "toggle"

# subtitle will show under the title in index page
subtitle = "A zola theme forked from [archie](https://github.com/athul/archie)"

# if set true, will use external CDN resource to load font and js file
useCDN = false

favicon = "/icon/favicon.png"

# show in the footer
copyright = "keith"

# config your Google Analysis ID
ga = "XXXX-XXXXX"


# config menu path
[[extra.menu]]
name = "Home"
url = "/"
weight = 1

[[extra.menu]]
name = "All posts"
url = "/posts"
weight = 2

[[extra.menu]]
name = "About"
url = "/about"
weight = 3

[[extra.menu]]
name = "Tags"
url = "/tags"
weight = 4

# config social icon info in the footer
[[extra.social]]
icon = "github"
name = "GitHub"
url = "https://github.com/XXXMrG/archie-zola"

[[extra.social]]
icon = "twitter"
name = "Twitter"
url = "https://github.com/your-name/"

[[extra.social]]
icon = "gitlab"
name = "GitLab"
url = "https://gitlab.com/your-name/"

```

### Content config

**In content/posts/_index.md. I use Zola config: transparent = true to implement the pagination**

In Zola, you can use config in the _index.md to control pagination and sort post list:

```toml
paginate_by = 3
sort_by = "date"
```

## Extension

Follow this [doc](https://www.getzola.org/documentation/themes/extending-a-theme/) to extend theme.

## Contributing

Thank you very much for considering contributing to this project!

We appreciate any form of contribution:

* New issues (feature requests, bug reports, questions, ideas, ...)
* Pull requests (documentation improvements, code improvements, new features, ...)
        