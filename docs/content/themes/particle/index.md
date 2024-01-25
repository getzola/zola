
+++
title = "particle"
description = "Particle theme for Zola"
template = "theme.html"
date = 2024-01-25T10:41:35+02:00

[extra]
created = 2024-01-25T10:41:35+02:00
updated = 2024-01-25T10:41:35+02:00
repository = "https://github.com/svavs/particle-zola.git"
homepage = "https://github.com/svavs/particle-zola"
minimum_version = "0.16.1"
license = "MIT"
demo = "https://particle-zola.vercel.app/"

[extra.author]
name = "Silvano Sallese"
homepage = "https://svavs.github.io/"
+++        

# Port for Zola of the Particle Jekyll theme

![](./screenshot.jpg)

This is a simple and minimalist template for Zola designed for developers that want to show of their portfolio.

The Theme features:

- Gulp
- SASS
- Sweet Scroll
- Particle.js
- BrowserSync
- Font Awesome and Devicon icons
- Google Analytics
- Info Customization

## Basic Setup

1. [Install Zola](https://getzola.com)
2. Clone the particle theme: `git clone https://github.com/svavs/particle-zola.git`
3. Edit `config.toml` to personalize your site.

## Site and User Settings

You have to fill some informations on the `[extra]` section of the `config.toml` to customize your site.

```
# Site settings
description = "A blog about lorem ipsum dolor sit amet"

# User settings
username = "Lorem Ipsum"
user_description = "Anon Developer at Lorem Ipsum Dolor"
user_title = "Anon Developer"
email = "my@email.com"
twitter_username = "lorem_ipsum"
github_username = "lorem_ipsum"
gplus_username = "lorem_ipsum"
```

## Color and Particle Customization
- Color Customization
  - Edit the sass variables (`_vars.scss`)
- Particle Customization
  - Edit the json data in particle function in app.js
  - Refer to [Particle.js](https://github.com/VincentGarreau/particles.js/) for help

To customize the project lists and the about sections, you need to edit the `templates/content.html` template file.
In future versions will be provided a simpler way.

## Questions

Having any issues file a [GitHub Issue](https://github.com/svavs/particle-zola/issues/new).

## License

This theme is free and open source software, distributed under the The MIT License. So feel free to use this Jekyll theme anyway you want.

## Credits

This theme was partially designed with the inspiration from these fine folks
- [Nathan Randecker](https://github.com/nrandecker/particle)
- [Willian Justen](https://github.com/willianjusten/will-jekyll-template)
- [Vincent Garreau](https://github.com/VincentGarreau/particles.js/)

        