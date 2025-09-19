
+++
title = "boring"
description = "A minimal theme"
template = "theme.html"
date = 2025-09-05T12:48:33+05:30

[taxonomies]
theme-tags = []

[extra]
created = 2025-09-05T12:48:33+05:30
updated = 2025-09-05T12:48:33+05:30
repository = "https://github.com/ssiyad/boring.git"
homepage = "https://github.com/ssiyad/boring"
minimum_version = "0.16.0"
license = "GPLv3"
demo = "https://boring-zola.netlify.app/"

[extra.author]
name = "Sabu Siyad"
homepage = "https://ssiyad.com"
+++        

# Boring
Minimal theme for [Zola](https://www.getzola.org/), powered by
[TailwindCSS](https://tailwindcss.com/)

### Demo
https://boring-zola.netlify.app/

![sreenshot](./screenshot.png)

### Setup
In your zola site directory
- Get theme

    ```bash
    git submodule add https://github.com/ssiyad/boring themes/boring
    ```

- Build CSS

    ```bash
    cd themes/boring
    yarn install --frozen-lockfile
    yarn build
    ```

- Change theme specific variables. They are listed in `extra` section of
  [config.toml](./config.toml)

Refer [Zola Docs](https://www.getzola.org/documentation/themes/installing-and-using-themes/#using-a-theme)
for further instructions

### License
[GPLv3](./LICENSE)


        