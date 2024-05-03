
+++
title = "Mabuya"
description = "🦎 Mabuya—a minimal Zola theme with the goal of helping you build a light and SEO-ready blog."
template = "theme.html"
date = 2024-05-03T12:02:37-07:00

[extra]
created = 2024-05-03T12:02:37-07:00
updated = 2024-05-03T12:02:37-07:00
repository = "https://github.com/semanticdata/mabuya.git"
homepage = "https://github.com/semanticdata/mabuya"
minimum_version = "0.18.0"
license = "MIT"
demo = "https://mabuya.vercel.app/"

[extra.author]
name = "Miguel Pimentel"
homepage = "https://miguelpimentel.do/"
+++        

# 🦎 Mabuya

<p align="">
  <img src="https://img.shields.io/github/languages/code-size/semanticdata/mabuya" />
  <img src="https://img.shields.io/github/repo-size/semanticdata/mabuya" />
  <img src="https://img.shields.io/github/commit-activity/t/semanticdata/mabuya" />
  <img src="https://img.shields.io/github/last-commit/semanticdata/mabuya" />
  <img src="https://img.shields.io/website/https/mabuya.vercel.app.svg" />
</p>

[Mabuya](https://mabuya.vercel.app/) is a minimal [Zola](https://www.getzola.org/) theme focused on helping you build an elegant, fast, lightweight, and SEO-ready blog. Put your work front and center with Mabuya as the base of your project.

Check out the live [demo](https://mabuya.vercel.app/) of the theme.

## Table of Contents

- 🦎 Mabuya
  - Table of Contents
  - Screenshot
  - Background
  - Features and Improvements
  - Running the Site Locally
  - Customization
  - Reporting Issues
  - Contributing
  - Attributions
  - License

## Screenshot

<img alt="Website Screenshot" src="screenshot.png" width="720px" />

## Background

While searching for themes, I came across [Zola Tale](https://github.com/aaranxu/tale-zola). Sadly, the project's last update was on Dec 4, 2021. Shortly after, I decided to fork the project and add my own touches to it.

The name Mabuya comes from the [Mabuya hispaniolae](https://en.wikipedia.org/wiki/Mabuya_hispaniolae?useskin=vector), a possibly extinct[^1] species of skink endemic to the Dominican Republic, my home country.

## Features and Improvements

While working on the theme, I have added new functionality and made many quality of life improvements. Here's a short list:

- Refactored stylesheets.
- Added Dark theme and color theme toggle.
- Added new footer navigation.
- Created a custom GitHub Action to deploy Zola sites faster than any other GitHub Actions using Docker.
- Refined page transitions from desktop to mobile and viceversa.
- Centralized custom variables–made it easier to customize the site's colors.
- Addressed PR [#7](https://github.com/aaranxu/tale-zola/pull/7) fixing the pagination problem present in the original Zola theme.
- Addressed Issue [#4](https://github.com/aaranxu/tale-zola/issues/4) fixing custom text not being used correctly.
- Addressed (temporarily) Issue [#1](https://github.com/aaranxu/tale-zola/issues/1) by removing the erroneous pinned marker.
- Optimized for speed and accessibility. Subtle color changes to make the text more readable, etc.
- Many other small improvements...

## Running the Site Locally

Tips that will help you develop and preview the site locally.

### Requirements

Before using the theme, you need to install [Zola](https://www.getzola.org/documentation/getting-started/installation/) ≥ 0.18.0.

### Quick Start

```bash
# clone the repo
git clone git@github.com:semanticdata/mabuya.git
# change directory into the cloned folder
cd mabuya
# serve the site locally
zola serve
# open http://127.0.0.1:1111/ in the browser
```

For more detailed instructions, visit the [Documentation](https://www.getzola.org/documentation/themes/installing-and-using-themes/) page about installing and using themes.

### Useful Commands

A short list of commands that will help you develop your own version of the theme.

| Command                    | Description                |
| -------------------------- | -------------------------- |
| `zola init <repo-name>`    | Initiate new Zola site     |
| `zola build`               | Build only                 |
| `zola serve`               | Build and Serve            |

## Customization

You can changed the configuration, templates and content yourself. Refer to the `config.toml`, and templates files for ideas. In most cases you only need to modify the contents of `config.toml` to customize the appearance of your blog. Make sure to visit tyhe [Zola Documentation](https://www.getzola.org/documentation/getting-started/overview/).

### Custom CSS Styles

Adding custom CSS is as easy as adding your styles to `sass/_custom.scss`. This is made possible because SCSS files are backwards compatible with CSS. This means you can type normal CSS code into a SCSS file and it will be valid.

## Reporting Issues

We use GitHub Issues as the official bug tracker for **Mabuya**. Please
search [existing issues](https://github.com/semanticdata/mabuya/issues). It’s
possible someone has already reported the same problem.

If your problem or idea is not addressed yet, [open a new issue](https://github.com/semanticdata/mabuya/issues/new).

## Contributing

We'd love your help! Please see [CONTRIBUTING](./CONTRIBUTING.md) to learn about the kinds of contributions we're looking for.

Please read and be aware of the [Code of Conduct](.github/CODE_OF_CONDUCT.md) before contributing.

## Attributions

Mabuya is a fork of [Tale](https://github.com/aaranxu/tale-zola), which itself is a port of the Jekyll theme [Tale](https://github.com/chesterhow/tale) which is now archived.

The icons used throughout the site are kindly provided by [UXWing](https://uxwing.com/license/). Read their [license](https://uxwing.com/license/).

## License

Source code in this repository is available under the [MIT License](LICENSE).

[^1]: *Mabuya hispaniolae*'s conservation status is *Critically endangered, possibly extinct*.  

        