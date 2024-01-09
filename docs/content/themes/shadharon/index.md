
+++
title = "shadharon"
description = "Simple blog theme powered by Zola"
template = "theme.html"
date = 2024-01-09T08:33:22+01:00

[extra]
created = 2024-01-09T08:33:22+01:00
updated = 2024-01-09T08:33:22+01:00
repository = "https://github.com/syedzayyan/shadharon"
homepage = "https://github.com/syedzayyan/shadharon"
minimum_version = "0.4.0"
license = "MIT"
demo = "https://syedzayyan.github.io/shadharon"

[extra.author]
name = "Syed Zayyan Masud"
homepage = "https://syedzayyan.com"
+++        

# Shadharon

Simple blog theme powered by [Zola](getzola.org). See a live preview [here](https://shadharon.syedzayyan.com/).

> Name derived from the Bengali Word - সাধারণ which translates to "generic"

<details open>
  <summary>Dark theme</summary>

  ![blog-dark](https://raw.githubusercontent.com/syedzayyan/shadharon/main/screenshot.png)
</details>

<details close>
  <summary>Light theme</summary>
  
  ![light-dark](https://raw.githubusercontent.com/syedzayyan/shadharon/main/screenshot-light.png)
</details>

## Features

- [X] Themes (light, dark). Default theme is dark with a switcher in the navbar
- [X] Projects page
- [x] Social Links
- [x] Tags

## Installation

0. Initialize Git Repo if not initialized

1. Download the theme
```
git submodule add https://github.com/syedzayyan/shadharon themes/shadharon
```

2. Add `theme = "shadharon"` to your `config.toml`

3. Copy the example content

```
cp -R themes/shadharon/content/. content
```

## Customization

1. For customization refer to config.toml files, which has comments.

2. For customizing the banner on the homepage the content/posts/_index.md needs modification. The desc variable under `extra`, specifically. You could delete this as well to remove banner. For an about page or any aditional page an .md file in the "content" directory will do.

You can add stylesheets to override the theme:

```toml
[extra]
stylesheets = [
    "override.css",
]
```

These filenames are relative to the root of the site. In this example, the two CSS files would be in the `static` folder.


## References

This theme takes inspiration from 
- [apollo](https://github.com/not-matthias/apollo).  
- [Tania's Website](https://tania.dev/)
- [Anpu Zola Theme](https://github.com/zbrox/anpu-zola-theme)

        