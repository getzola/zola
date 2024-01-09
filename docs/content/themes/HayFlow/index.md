
+++
title = "HayFlow"
description = "HayFlow is a minimal and completely modular Zola theme for anyone wishing to have their own landing page."
template = "theme.html"
date = 2024-01-09T08:33:22+01:00

[extra]
created = 2024-01-09T08:33:22+01:00
updated = 2024-01-09T08:33:22+01:00
repository = "https://gitlab.com/cyril-marpaud/hayflow.git"
homepage = "https://gitlab.com/cyril-marpaud/hayflow"
minimum_version = "0.4.0"
license = "CC-BY-SA 4.0"
demo = "https://cyril-marpaud.gitlab.io"

[extra.author]
name = "Cyril Marpaud"
homepage = "https://cyril-marpaud.gitlab.io"
+++        

# HayFlow - Modular Zola Theme

## About

<div align="center">
![Preview screenshot](https://gitlab.com/cyril-marpaud/hayflow/-/raw/main/screenshot.png "Preview screenshot")
</div>

[HayFlow](https://gitlab.com/cyril-marpaud/hayflow) is a modular landing page made as a theme for [Zola](https://www.getzola.org), a static site generator written in [Rust](https://www.rust-lang.org). It features a dark theme with a particles background, vertical arrows for navigation and a few card types which you are free to include to best suit your needs. Nearly all UI elements are subtly animated to convey a professional look (although I'm no designer 🤷 merely an [embedded systems engineer](https://www.linkedin.com/in/cyrilmarpaud)).

It has been designed to require only [Markdown](https://www.markdownguide.org) editing (no HTML/CSS), but feel free to do so if you need to. I'll be glad to review a [Merge Request](https://gitlab.com/cyril-marpaud/hayflow/-/merge_requests) if you implement a new card type !

[[_TOC_]]

## Live demo

See [my personal website](https://cyril-marpaud.gitlab.io) for an example of what can be accomplished in a few minutes with this theme. Its source code is also available as an example in my [Gitlab website repository](https://gitlab.com/cyril-marpaud/cyril-marpaud.gitlab.io).

## Built with

- [Zola](https://www.getzola.org)
- [Particles.js](https://vincentgarreau.com/particles.js/)
- [Font Awesome](https://fontawesome.com)
- [Modern Normalize](https://github.com/sindresorhus/modern-normalize)
- Inspiration came from [particle-zola](https://github.com/svavs/particle-zola), another theme.

## Quick start

Initialize a Zola website and install HayFlow:
```bash
zola init mywebsite
cd mywebsite
git clone git@gitlab.com:cyril-marpaud/hayflow.git themes/hayflow
```

Add `theme = "hayflow"` at the top of `config.toml` file to tell Zola to use HayFlow (as described in [the documentation](https://www.getzola.org/documentation/themes/installing-and-using-themes/)).

Finally, run...

```bash
zola serve
```
...and go to [http://localhost:1111](http://localhost:1111) to see the landing page in action with the default name displayed (John Doe).

## Landing page customization

Customizing the landing page boils down to two things:

- adding the `name` and `links` variables to the `config.toml`'s `[extra]` section (`links` is optional. So is `name` if your name is John Doe)
- adding the `roles` variable to the `content/_index.md`'s `[extra]` section (also optional)

The difference comes from the fact that you might need to translate the `roles` into other languages. For that to be possible, they must be placed in a MarkDown file. See multilingual support for more info.

- `name` speaks for itself.
- `roles` is an array of strings. Each string is displayed on a separate line.
- `links` is an array of `{icon, url}` objects. You can use any **free** icon from [Font Awesome](https://fontawesome.com/search?o=r&m=free) here, all you need is the icon's code. The [enveloppe icon](https://fontawesome.com/icons/envelope?s=solid&f=classic)'s code is `fa-solid fa-envelope`. The [pizza-slice icon](https://fontawesome.com/icons/pizza-slice?s=solid&f=classic)'s code is `fa-solid fa-pizza-slice`.

This is what the `config.toml`'s `[extra]` section might look like after customization:

```TOML
[extra]
name = { first = "ninja", last = "turtle" }

links = [
   { icon = "fa-solid fa-envelope", url = "mailto:slice@pizza.it" },
   { icon = "fa-solid fa-pizza-slice", url = "https://en.wikipedia.org/wiki/Pizza" },
]
```

And here's a customized version of `content/_index.md`:

```TOML
+++
[extra]
roles = ["Green 🟢", "Turtle 🐢", "Pizza enthusiast 🍕"]
+++
```

## Adding a section

Inside the `content` directory, create a `pizza` folder and place this `_index.md` file inside:

```TOML
+++
title = "Pizza"
+++

What a mouthful !
```

Then, add this `sections` variable (an array of strings) to the `config.toml`'s `[extra]` section:

```TOML
[extra]
sections = ["pizza"]
```

A new internal link pointing to that section will appear on the landing page. Click it and see what happens ! This is called a "simple card" section.

## Customizing sections

HayFlow currently supports three card types : `simple`, `columns` and `list`. If left unspecified, the type will default to `simple`. To change it, add a `card_type` variable to the `_index.md`'s `[extra]` section:

```TOML
+++
title = "Pizza"

[extra]
card_type = "simple"
+++

What a mouthful !
```

### Columns card

Add a new section and set its card type to `columns`. Then, alongside the `_index.md` file, create three other files: `one.md`, `two.md` and `three.md`. These will be the ingredients of your new pizza. Their content is similar to `_index.md`:

```TOML
+++
title = "Tomato"

[extra]
icons = ["fa-solid fa-tomato"]
+++

The basis of any self-respecting pizza. It is the edible berry of the plant Solanum lycopersicum.
```

The `icons` variable is optional.

### List card

Add a new section and set its card type to `list`. Then, alongside the `_index.md` file, create three other files: `one.md`, `two.md` and `three.md`. These will be your favourite pizzas. Their content is similar to `_index.md`:

```TOML
+++
title = "Margherita"

[extra]
link = "https://en.wikipedia.org/wiki/Pizza_Margherita"
+++

Margherita pizza is a typical [Neapolitan pizza](https://en.wikipedia.org/wiki/Neapolitan_pizza), made with San Marzano tomatoes, mozzarella cheese, fresh basil, salt, and extra-virgin olive oil.
```

The `link` variable is optional.

## Multilingual support

HayFlow supports multilingual websites out of the box.

### Declare more languages

In `config.toml`, add the languages you want to support like so:

```TOML
default_language = "fr"
[translations]
flag = "🇫🇷"

[languages.en]
[languages.en.translations]
flag = "🇬🇧"

[languages.italian]
[languages.italian.translations]
flag = "🇮🇹"
```

This will make the language-select block in the top-right corner visible. It consists of clickable links to the translated versions of your website.
The `flag` variable is optional and you can use simple text instead of an emoji flag. If left unspecified, it will default to the country code you chose for that language (`fr`, `en` and `italian` in this example).

### Translate the content

Each `.md` file in the `content` folder now needs to be translated into every additional language previously declared in `config.toml`.

Following the above example (three languages, french, english and italian) and given this initial filetree:

```
content/
   _index.md
   pizzas/
      _index.md
      margherita.md
      capricciosa.md
```

The final filetree should look like this for the translation to be complete:

```
content/
   _index.md
   _index.en.md
   _index.italian.md
   pizzas/
      _index.md
      _index.en.md
      _index.italian.md
      margherita.md
      margherita.en.md
      margherita.italian.md
      capricciosa.md
      capricciosa.en.md
      capricciosa.italian.md
```

### List cards

Additionally, if your website includes any "list card" sections, you might want to specify a `discover` variable in their `[extra]` sections like so:

```TOML
+++
title = "List Card Section"

[extra]
card_type = "list"
discover = "Découvrir"
+++
```

## Whoami

My name is Cyril Marpaud, I'm an embedded systems freelance engineer and a Rust enthusiast 🦀 I have nearly 10 years experience and am currently living in Lyon (France).

<div align="center">

[![LinkedIn][linkedin-shield]][linkedin-url]

[linkedin-url]: https://www.linkedin.com/in/cyrilmarpaud/
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=grey&logoColor=blue

</div>

        