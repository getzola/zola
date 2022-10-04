
+++
title = "apollo"
description = "Modern and minimalistic blog theme"
template = "theme.html"
date = 2022-10-04T04:04:47-05:00

[extra]
created = 2022-10-04T04:04:47-05:00
updated = 2022-10-04T04:04:47-05:00
repository = "https://github.com/not-matthias/apollo.git"
homepage = "https://github.com/not-matthias/apollo"
minimum_version = "0.14.0"
license = "MIT"
demo = "https://not-matthias.github.io/apollo"

[extra.author]
name = "not-matthias"
homepage = "https://github.com/not-matthias"
+++        

# apollo

Modern and minimalistic blog theme powered by [Zola](getzola.org). See a live preview [here](https://not-matthias.github.io/apollo).

<sub><sup>Named after the greek god of knowledge, wisdom and intellect</sup></sub>

<details open>
  <summary>Dark theme</summary>
  
  ![blog-dark](https://user-images.githubusercontent.com/26800596/168986771-4ed049e2-e123-4d0e-8a24-7bf43f47551f.png)
</details>

<details>
  <summary>Light theme</summary>
  
![blog-light](https://user-images.githubusercontent.com/26800596/168986766-72a48517-7122-465d-8108-3ae33e1e88b1.png)
</details>

## Features

- [X] Pagination
- [X] Themes (light, dark, auto)
- [X] Analytics using [GoatCounter](https://www.goatcounter.com/)
- [ ] Social Links
- [ ] Search
- [ ] Categories

## Installation

1. Download the theme
```
git submodule add https://github.com/not-matthias/apollo themes/apollo
```

2. Add `theme = "apollo"` to your `config.toml`
3. Copy the example content

```
cp themes/apollo/content content
```

## Options

### Additional stylesheets

You can add stylesheets to override the theme:

```toml
[extra]
stylesheets = [
    "override.css",
    "something_else.css"
]
```

These filenames are relative to the root of the site. In this example, the two CSS files would be in the `static` folder.

## References

This theme is based on [archie-zola](https://github.com/XXXMrG/archie-zola/).  

        