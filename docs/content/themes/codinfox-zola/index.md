
+++
title = "codinfox-zola"
description = "Codinfox theme for Zola"
template = "theme.html"
date = 2021-02-18T22:27:50+01:00

[extra]
created = 2021-02-18T22:27:50+01:00
updated = 2021-02-18T22:27:50+01:00
repository = "https://github.com/svavs/codinfox-zola"
homepage = "https://github.com/svavs/codinfox-zola"
minimum_version = "0.11.0"
license = "MIT"
demo = "https://codinfox-zola.vercel.app/"

[extra.author]
name = "Silvano Sallese"
homepage = "https://svavs.github.io/"
+++        

# Codinfox-Zola

![Zola Deploy to Github Pages on push](https://github.com/svavs/codinfox-zola/workflows/Zola%20Deploy%20to%20Pages%20on%20push/badge.svg?branch=master)

This is a [Zola](https://www.getzola.com) theme inspired to [Codinfox-Lanyon](https://codinfox.github.com/), a Lanyon based theme for [Jekyll](http://jekyllrb.com). See a live demo [here](https://codinfox-zola.vercel.app/).

This theme places content first by tucking away navigation in a hidden drawer.

* Built for [Zola](https://www.getzola.com)
* Developed on GitHub and hosted for free on [GitHub Pages](https://pages.github.com) and [Vercel](https://vercel.com)
* Coded with [Spacemacs](https://www.spacemacs.org)

This theme supports:

1. Theme colors: you can choose your favorite theme color (changing in `_config.scss`)
2. Changable sidebar locations (reverse it by changing the boolean value in `_config.scss`)
3. Integration of FontAwesome, MathJax, Disqus and Google Analytics
4. Support for multilingual sites
4. and numerous improvements over original Lanyon and Codinfox-Lanyon

All the configuration variables and their meaning are inside:

- `config.toml` (for the zola config variables and some extra variables required by this theme),
- `author.toml` (for the personal informations to be displayed about the author of the site),
- `nav.toml` (for the navigation menu structure available in the site's sidebar)
- `_config.scss` (for the definition of some css customizations)

The options are fairly straightforward and described in comments.

Learn more and contribute on [GitHub](https://github.com/svavs/codinfox-zola).

Have questions or suggestions? Feel free to [open an issue on GitHub](https://github.com/svavs/codinfox-zola/issues/new) or [ask me on Twitter](https://twitter.com/svavs).

### Install and use

To use this theme you can follow the instruction required by any Zola theme.

Simply clone this repository under the `themes` folder of your site's main folder.

Then, define the required extra variables in the config.toml (take it from the config.toml file of the theme), create and define the author.toml and nav.toml configuration files in the main folder of your site (the same level of the config.toml), and that's it!

To define your own home picture, put an image file in the `static/img/` folder and set the path in the config.extra.image variable.

Now is possible to create the content inside the `content` folder as usual for Zola sites.

If you want to have a Blog with this theme, then create a folder inside the `content` folder containing all the blog posts in Markdown format. Zola automatically generate a section that you can manage as a blog. See an example in the [live demo](https://codinfox-zola.vercel.app/blog/).
 
## License

Open sourced under the [MIT license](LICENSE.md).


## TODO
 - recaptcha for hiding email address link (https://developers.google.com/recaptcha/intro)
 - hidden multilingual links in topbar for main index section pages

        