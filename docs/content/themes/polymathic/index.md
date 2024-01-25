
+++
title = "polymathic"
description = "A portfolio theme for person of many talents"
template = "theme.html"
date = 2024-01-25T10:41:35+02:00

[extra]
created = 2024-01-25T10:41:35+02:00
updated = 2024-01-25T10:41:35+02:00
repository = "https://github.com/anvlkv/polymathic"
homepage = "https://github.com/anvlkv/polymathic"
minimum_version = "0.17.2"
license = "MIT"
demo = "https://main--polymathic-demo.netlify.app/"

[extra.author]
name = "Aleksandr Novolokov"
homepage = "https://a.nvlkv.xyz"
+++        

# polymathic

<a href="https://www.producthunt.com/posts/polymathic?utm_source=badge-featured&utm_medium=badge&utm_souce=badge-polymathic" target="_blank"><img src="https://api.producthunt.com/widgets/embed-image/v1/featured.svg?post_id=422530&theme=light" alt="polymathic - Zola&#0032;portfolio&#0032;theme&#0032;for&#0032;those&#0032;with&#0032;many&#0032;talents | Product Hunt" style="width: 250px; height: 54px;" width="250" height="54" /></a>

polymathic is a [Zola](https://www.getzola.org/) portfolio theme. 

I made it for my own portfolio. The theme is called `polymathic`, inspired by individuals with a wide range of talents. The theme focuses on rich and consistent navigation experience, exposing the variety of topics to chose from, yet allowing the user to focus on a single thread of your story once they've made a choice. 

Docs and theme demo are available here [main--polymathic-demo.netlify.app](https://main--polymathic-demo.netlify.app/) 

This theme uses [Bulma](https://bulma.io/) scss framework, making the theme styles highly customizable and enabling mobile first theme design.

This theme uses [Animate.css](https://animate.style) for animations.

This theme adds minimal [Open Graph](https://ogp.me/) tags to every page `head`.

You can quickly deploy the theme to [netlify](https://docs.netlify.com/site-deploys/create-deploys/), theme comes with a config file.

## Features

See all features [demonstrated in the docs](https://main--polymathic-demo.netlify.app/features). 

### Media support

The theme is friendly to wide range of screen sizes from `mobile` to `fullhd`. Theme comes with minimal styles for `print` media.

#### Dark mode

Theme includes preference based dark mode as separate stylesheet. No switch.

#### Accessibility

This theme automatically finds accessible colors when using customizations, with minimal config.

This theme supports no script environments.

This theme respects user preference for reduced motion.

### Navigation

This theme builds navigation for your site. The outcome is highly customizable via your `config.toml` and front-matter of your sections.

### Templates

The theme comes with templates for `index.html`, `page.html`, `section.html`, `taxonomy_list.html`, `taxonomy_single.html`, `404.html`. You can use them in your Zola project as is or by extending them, templates are divided in `block`s and `partials/*.html` for convenience of extending the theme.

### Brand and style

The theme is highly customizable via `config.toml` and sass variables. Your customization can start from just the primary color or extend all the way to bulma variables.

### Shortcodes

The theme comes with several shortcodes for building forms, galleries, navigation cards and banners.

## Install

Once you already have zola installed and ran `zola init`, then run from your project directory

    $ git init
    $ git submodule add https://github.com/anvlkv/polymathic themes/polymathic

You will also need [npm](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm) installed, then run

    $ npm --prefix themes/polymathic install

For those using netlify deployments config is available here

    $ cp themes/polymathic/netlify.toml netlify.toml

In your `config.toml` Set zola theme to polymathic

    theme = "polymathic"


## Contributing

Issues or contributions are welcome. Also, curious what you make with it.


        