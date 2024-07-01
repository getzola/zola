
+++
title = "Zplit"
description = "A single page theme for a professional online presence."
template = "theme.html"
date = 2024-07-01T05:58:26Z

[extra]
created = 2024-07-01T05:58:26Z
updated = 2024-07-01T05:58:26Z
repository = "https://github.com/gicrisf/zplit.git"
homepage = "https://github.com/gicrisf/zplit"
minimum_version = "0.15.0"
license = "Creative Commons Attribution 3.0 License"
demo = "https://zplit.netlify.app"

[extra.author]
name = "Giovanni Crisalfi"
homepage = "https://github.com/gicrisf"
+++        

# Zplit

Zplit is a single-page, centrally-divided layout designed for a professional online presence. It features a large image or video on the left, accompanied by content on the right. Zplit is a port of [Split](//onepagelove.com/split) by [One Page Love](//onepagelove.com) for [Zola](https://www.getzola.org/).

![Zola Zplit Theme screenshot](screenshot.png)

**DEMO**: [https://zplit.netlify.app/](https://zplit.netlify.app/)

## Installation

Download the theme to your `themes` directory:

```bash
$ cd themes
$ git clone https://github.com/gicrisf/zplit.git
```

Then, enable the theme editing your `config.toml`:

```toml
theme = "zplit"
```

## Getting started

The most important file of the theme is located in the root directory and is named =config.toml=. Edit this file to customize your preferences. Look for sections like `[extra]` to set variables like `author`, or `[extra.content]` to modify intro_tagline.

If something is unclear or not obvious, you might have missed [the "configuration" section of the Zola official documentation](https://www.getzola.org/documentation/getting-started/configuration/). Even if you're new to static site generators, don't worry and take some time to go through the documentation, as it covers fundamental concepts.

Here after, we will discuss two specific sections in more detail, because those are unique for the Zplit theme:
- Background image
- Lists (of links)

### Background image

Edit the `[extra.visual]` section to set your background image of choice.

```toml
[extra.visual]

background = "<your-image-file-path-goes-here>"
```

You can find this example already written as the default:

```toml
[extra.visual]

background = "images/background.jpg"
position = "center center"
```

As you can see, you can edit the relative position of the image, which is centered by default.

### Lists

You can set up to 3 lists of links in the `[extra.lists]` section of the `config.toml` file: 
- connect
- social
- network

Manipulating them is very easy: just add/remove elements in the TOML list, as showed in this example (also already present in the default file):

``` toml
social = [
    {url = "https://t.me/zwitterio", text = "Telegram"},
    {url = "https://twitter.com/gicrisf", text = "Twitter"},
    {url = "https://github.com/gicrisf", text = "Github"},
]
```

Do you want another item? Just throw it up to the pile. You have no limits.
Remember to set the `url` field with the link itself you want to direct your user at and a `text` to show in the page for the corrisponding URL.

## Posts

To add new posts, simply place markdown files in the `content` directory. In order to sort the post index by date, you need to enable the `sort_by` option in the `content/_index.md` file within the index section.

```toml
sort_by = "date"
```


This theme was not specifically designed for blogging, but rather as a landing page for professionals. However, if you wish to blog using this theme, you certainly can. To do so, simply add a new section in the content directory and include it in the main menu through the config file. This will make it readily accessible to the user.

The theme does not offer support for taxonomies or other advanced features. It is focused on providing simple pages. If you wish to enhance the blogging functionality, you are welcome to customize the code or submit a specific request as an issue.

## Custom CSS

To make custom changes to the original stylesheets, you can create a `custom.css` file in the `static` directory. In this file, you can add any modifications or additions you desire.

## Custom colors

If you need to make adjustments to the colors or grid dimensions, it may be easier to modify the frontmatter of the `_01-content.scss` file directly. In this file, you will find variables conveniently located at the top:

``` scss
//-------------------------------------------------------------------------------
// Variables
//-------------------------------------------------------------------------------

// Colors
$color-background : #061C30;
$color-text       : #848d96;
$color-link       : #848d96;
$color-link-hover : #CA486d;
$color-maverick   : #47bec7;
$color-tagline    : #CCCCCC;

// Breakpoints
$bp-smallish      : 1200px;
$bp-tablet        : 800px;
$bp-mobile        : 500px;
```

## Features

- [x] Lightweight and minimal
- [x] Responsive (mobile support)
- [x] Social links
- [x] Deploy via Netlify (config already included)
- [x] Easily extendable menus
- [x] De-googled (local assets are faster and more secure)
- [x] Netlify support
- [x] Custom CSS
- [x] Custom colors
- [x] 404 page
- [x] Basic blogging features
- [ ] Open Graph and Twitter Cards support
- [ ] Multilanguage support

## Support me!

Do you love Zplit? Did you find it enjoyable and useful? If so, consider showing your support by making a donation. Your contribution will help fund the development of new features and improvements for this theme.

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/V7V425BFU)

## License

The original template is released under the [Creative Commons Attribution 3.0 License](//github.com/escalate/hugo-split-theme/blob/master/LICENSE.md). Please keep the original attribution link when using for your own project. If you'd like to use the template without the attribution, you can check out the license option via the template [author's website](//onepagelove.com/split).

        