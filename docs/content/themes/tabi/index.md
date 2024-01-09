
+++
title = "tabi"
description = "A fast, lightweight, and modern Zola theme with multi-language support, optional JavaScript, and a perfect Lighthouse score"
template = "theme.html"
date = 2024-01-09T08:33:22+01:00

[extra]
created = 2024-01-09T08:33:22+01:00
updated = 2024-01-09T08:33:22+01:00
repository = "https://github.com/welpo/tabi.git"
homepage = "https://github.com/welpo/tabi"
minimum_version = "0.17.0"
license = "MIT"
demo = "https://welpo.github.io/tabi"

[extra.author]
name = "Óscar Fernández"
homepage = "https://osc.garden"
+++        

# tabi

A fast, lightweight, and modern [Zola](https://www.getzola.org) theme with multi-language support. It aims to be a personal page and home to blog posts.

See a live preview (and the theme's documentation) [here](https://welpo.github.io/tabi).

Explore the Sites Using tabi section to see real-world applications.

> tabi (旅): Journey.

![tabi](https://github.com/welpo/tabi/raw/main/light_dark_screenshot.png)

tabi has a perfect score on Google's Lighthouse audit:

![lighthouse](https://raw.githubusercontent.com/welpo/tabi/main/lighthouse_score.png)

## Features

- [X] [Set any language as default](https://welpo.github.io/tabi/blog/faq-languages/#how-do-i-set-a-default-language-for-my-site). Set your base site to Chinese, Spanish, French, Hindi… or any [other supported language](/i18n). The theme's interface will be translated accordingly.
- [X] [Integration with remote repositories](https://welpo.github.io/tabi/mastering-tabi-settings/#git-repository-integration) on GitHub, GitLab, Gitea & Codeberg for commit history and showing the site source.
- [X] Dark and light themes. Defaults to the OS setting, with a switcher in the navigation bar.
- [X] Thorough documentation. See [Mastering tabi Settings: A Comprehensive Guide](https://welpo.github.io/tabi/blog/mastering-tabi-settings/).
- [X] Perfect Lighthouse score (Performance, Accessibility, Best Practices and SEO).
- [X] [Comprehensive multi-language support](https://welpo.github.io/tabi/blog/faq-languages/#how-does-tabi-handle-multilingual-support). Add as many languages as you wish.
- [X] Support for [comments using giscus, utterances, Hyvor Talk, or Isso](https://welpo.github.io/tabi/blog/comments/).
- [X] Code syntax highlighting with colours based on [Catppuccin](https://github.com/catppuccin/catppuccin) Frappé.
- [X] [Local search](https://welpo.github.io/tabi/blog/mastering-tabi-settings/#search) with an accessible, multi-lingual interface.
- [X] [KaTeX](https://katex.org/) support for mathematical notation.
- [X] [Stylized and human readable Atom feed](https://welpo.github.io/tabi/atom.xml).
- [X] [Stylized and human readable sitemap](https://welpo.github.io/tabi/sitemap.xml).
- [X] [Mail encoding](https://welpo.github.io/tabi/blog/mastering-tabi-settings/#encoded-email) for spam protection.
- [X] All JavaScript can be [fully disabled](https://welpo.github.io/tabi/blog/javascript/).
- [X] [Customizable Table of Contents](https://welpo.github.io/tabi/blog/toc/).
- [X] [Customizable secure headers](https://welpo.github.io/tabi/blog/security/).
- [X] [Copy button for code blocks](https://welpo.github.io/tabi/blog/mastering-tabi-settings/#copy-button-on-code-blocks).
- [X] [Quick navigation buttons](https://welpo.github.io/tabi/blog/mastering-tabi-settings/#quick-navigation-buttons).
- [X] [Custom copyright notice](https://welpo.github.io/tabi/blog/mastering-tabi-settings/#copyright).
- [X] [Custom canonical URLs](https://welpo.github.io/tabi/blog/mastering-tabi-settings/#canonical-url).
- [X] [Custom shortcodes](https://welpo.github.io/tabi/blog/shortcodes/).
- [X] [Customizable skins](https://welpo.github.io/tabi/blog/customise-tabi/).
- [X] [Footnote backlinks](https://welpo.github.io/tabi/blog/mastering-tabi-settings/#footnote-backlinks).
- [X] [Social media cards](https://welpo.github.io/tabi/blog/mastering-tabi-settings/#social-media-cards).
- [X] Responsive design.
- [X] [Projects page](https://welpo.github.io/tabi/projects/).
- [X] [Archive page](https://welpo.github.io/tabi/archive/).
- [X] [Social links](https://welpo.github.io/tabi/blog/mastering-tabi-settings/#social-media-icons).
- [X] [Tags](https://welpo.github.io/tabi/blog/mastering-tabi-settings/#tags).

## Quick start

Once you have installed Zola 0.17.0 or newer:

```bash
git clone https://github.com/welpo/tabi.git
cd tabi
zola serve
```

Open http://127.0.0.1:1111 in the browser.

## Installation

To add tabi to you existing Zola site:

0. Initialize a Git repository in your project directory (if you haven't already):

```
git init
```

1. Add the theme as a git submodule:

```
git submodule add https://github.com/welpo/tabi.git themes/tabi
```

Or clone the theme into your themes directory:

```
git clone https://github.com/welpo/tabi.git themes/tabi
```

### Required configuration

2. Enable the theme in your `config.toml`:

```
theme = "tabi"
```

3. Set a `title` in your `config.toml`:

```
title = "Your Site Title"
```

4. Create a `content/_index.md` file with the following content:

```
+++
title = "Home"
paginate_by = 5 # Set the number of posts per page
template = "index.html"
+++
```

If you want to serve your blog posts from a different path, such as `blog/`, add a `section_path` in the `[extra]` section of `content/_index.md` (this file will need pagination):

```
[extra]
section_path = "blog/_index.md"
```

5. If you want an introduction section (see screenshot above), add these lines to `content/_index.md`:

```
[extra]
header = {title = "Hello! I'm tabi~", img = "img/main.webp", img_alt = "Your Name" }
```

The content outside the front matter will be rendered between the header title and the posts listing. In the screenshot above, it's the text that reads "tabi is a fast, lightweight, and modern Zola theme…".

6. If you want a multilingual site, you will need to set up each language. In `config.toml`, set the title and taxonomies for each language, like:

```toml
[languages.es]
title = "~/tabi"
taxonomies = [{name = "tags", feed = true}]
```

You will need an `_index.{language_code}.md` per language for each section (e.g. /blog or /projects) that you want to enable in that language.

The same is true for individual posts, which should have the exact same name as the default language, with an extra `.{code}` before the extension (e.g. the Spanish version of `security.md` would be `security.es.md`).

This configuration allows the language switcher to take the user to the translation of the current URL. If a translation doesn't exist, the 404 page will be displayed, with an explanation in each language set in the config.

To learn more about multilingual support, see the [Frequently Asked Questions](https://welpo.github.io/tabi/blog/faq-languages/).

## Sites Using tabi

| Website | Creator | Description  | Site Source   |
| - | - | - | - |
| [osc.garden](https://osc.garden) | Óscar Fernández ([welpo](https://github.com/welpo)) | Data science, psychology, and Zola | [Source](https://github.com/welpo/osc.garden) |
| [sandip.live](https://sandip.live) | Sandip G ([sandman](https://github.com/sandman)) | Startups, tech and the good life | [Source](https://github.com/sandman/sandman.github.io) |
| [seadve.github.io](https://seadve.github.io/) | Dave Patrick Caberto ([SeaDve](https://github.com/SeaDve/)) | Personal blog and portfolio with custom CSS | [Source](https://github.com/SeaDve/seadve.github.io) |
| [donovan.is](https://donovan.is) | [Donovan Glover](https://github.com/donovanglover) | Linux, Rust, and Full Stack Web Development | [Source](https://github.com/donovanglover/donovan.is) |
| [mikufan.page](https://mikufan.page) | [Nadia](https://github.com/nyadiia) | Personal blog | [Source](https://github.com/nyadiia/mikufan.page) |
| [tim-boettcher.online](https://tim-boettcher.online/) | [Tim Böttcher](https://codeberg.org/Tim-Boettcher/) | Insights and ramblings of a deafblind programmer | [Source](https://codeberg.org/Tim-Boettcher/tim-boettcher-online/) |

Using tabi? Feel free to create a PR and add your site to this list.

## Inspiration

This theme was inspired by:
- [shadharon](https://github.com/syedzayyan/shadharon) — tabi started as a fork of [syedzayyan](https://github.com/syedzayyan)'s theme;
- [tailwind-nextjs-starter-blog](https://github.com/timlrx/tailwind-nextjs-starter-blog);
- [abridge](https://github.com/Jieiku/abridge);
- [internetVin's blog](https://internetvin.ghost.io).

## Contributing

Please do! We appreciate bug reports, improvements to translations or documentation (however minor), feature requests…

Take a look at the [Contributing Guidelines](/CONTRIBUTING.md) to learn more.

## License

The code is available under the [MIT license](./LICENSE).

        