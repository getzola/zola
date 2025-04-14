
+++
title = "Project Portfolio"
description = "Theme for a project portfolio (based on Tailwind CSS)."
template = "theme.html"
date = 2025-04-10T10:44:14+02:00

[taxonomies]
theme-tags = []

[extra]
created = 2025-04-10T10:44:14+02:00
updated = 2025-04-10T10:44:14+02:00
repository = "https://github.com/awinterstein/zola-theme-project-portfolio.git"
homepage = "https://codeberg.org/winterstein/zola-theme-project-portfolio"
minimum_version = "0.19.0"
license = "MIT"
demo = "https://awinterstein.github.io/zola-theme-project-portfolio-example/"

[extra.author]
name = "Adrian Winterstein"
homepage = "https://www.winterstein.biz"
+++        

# Project Portfolio

> You can find this theme on [Codeberg](https://codeberg.org/winterstein/zola-theme-project-portfolio) and [Github](https://github.com/awinterstein/zola-theme-project-portfolio).

A [Zola](https://www.getzola.org/) theme built with [Tailwind CSS](https://tailwindcss.com/) and [DaisyUI](https://daisyui.com) for presenting the project portfolio of a freelancer, for example. The theme is based on the [Blow](https://www.getzola.org/themes/blow/) theme that was created by Thomas Chartron and on my generic [Daisy](https://codeberg.org/winterstein/zola-theme-daisy) theme. It extends the [Daisy](https://codeberg.org/winterstein/zola-theme-daisy) with specific pages, taxonomies and shortcodes for managing a project portfolio and supports all color schemes of the [Daisy](https://codeberg.org/winterstein/zola-theme-daisy) as well.

![Screenshot of a project page example](screenshot.png)

Check out the [live demo of the example project](https://awinterstein.github.io/zola-theme-project-portfolio-example/)  or a [real-world implementation](https://www.winterstein.biz/) of this theme.

## Features

* Responsive design (looks good on desktop and mobile)
* Automatically selected dark / light modes
* 37 color schemes included
* Customizable navbar and footer (with social links)
* Project types and skills taxonomies
* Search functionality
* Multi-language support
* Pagination
* Customizable favicon
* Error 404 page

## Quick Start

For starting to create a new Zola website using this theme, the easiest approach is to just checkout / fork the [example repository](https://codeberg.org/winterstein/zola-theme-project-portfolio-example) and adapt it to your needs. That repository already contains a minimal structure and configuration for the Zola-based website and can directly be built and deployed to [Netlify](https://www.netlify.com/) and Github pages.

## Configuration

The minimal `config.toml` file for using the theme looks like this:

```toml
base_url = "https://www.example.com"

theme = "project-portfolio"

taxonomies = [
    {name = "projects", paginate_by = 5, feed = true},
    {name = "skills", paginate_by = 5, feed = true},
]
```

### Color Schemes

Set a light and dark color scheme:

```toml
daisyui_theme_light = "light"
daisyui_theme_dark = "dark"
```

See the `themes` list in the [`theme.toml`](theme.toml) for all possible identifiers. You can also set only a light or a dark color scheme, if you do not want the automatic dark mode switching based on the browser settings of your visitors.

If you want to allow your visitors to change the used color scheme, just set the following variable in the `[extra]` section of your `config.toml`:

```toml
[extra]
enable_theme_switching = true
```

There will be a dropdown in the navbar then, for the visitors to select form the color schemes.

### Languages

To enable support for multiple languages, simply set the default language and add language settings for all your additional languages:

```toml
default_language = "en"

[languages.de]
# title and description in the additional language
title = "Projekt-Portfolio"
description = "Beispiel- und Demoseite des Projekt-Portfolio-Themas für Zola."

# don't forget to enable features like search or feed
# generation for the additional language as well
build_search_index = true
generate_feeds = true

# also any taxonomies of your default language need to
# be defined for the additional language as well
taxonomies = [
    {name = "projects", paginate_by = 5, feed = true},
    {name = "skills", paginate_by = 5, feed = true},
]
```

Taxonomies should have exactly the same (not translated) name in all languages, for the language switching to work best.

You need to create an i18n file containing the translations for all theme variables for all the languages of your website, if they are not included in the theme. Right now, only [English](i18n/en.toml) and [German](i18n/de.toml) are included. You can create a the directory `i18n` in your website root directory and the language files in there will be picked up by the theme. It would be great, however, I you create a pull-request on the theme repository to add your translations to the theme.

### Search

Integrating a search into your website is as easy as adding the following to your configuration:

```toml
# enable it globally for the default language
build_search_index = true

[search]
# only this format is supported by the theme
index_format = "elasticlunr_json"

# you need to enable search at all your language sections as well
[languages.de]
build_search_index = true
```

As soon as `build_search_index` is enabled, the search indices are created for all languages that have this variable enabled in their section in the `config.toml` and the search bar is shown in the navbar of the website.

Just be aware, that you need to add a [Lunr Languages](https://github.com/MihaiValentin/lunr-languages) file to your `static` directory, if you ae using other languages than English and German. See the corresponding repository for the [`min` files](https://github.com/MihaiValentin/lunr-languages/tree/master/min). Feel free to add support for your languages to the theme as well, via a pull-request.

### Navbar

Arbitrary links can be added to the footer by defining the following list in the `[extra.navbar]` section:

```toml
[extra.navbar]
links = [
    { url = "projects", i18n_key = "projects" },
    { url = "skills", i18n_key = "skills" },
    { url = "blog", i18n_key = "blog" },
]
```

The value of the `i18n_key` must be in the `i18n` files for your languages (see [en.toml](i18n/en.toml), for example).

### Footer

All three parts of the footer can be adapted: the links, the social icons, and the copyright notice.

#### Links

Arbitrary links can be added to the footer by defining the following list in the `[extra.footer]` section:

```toml
[extra.footer]
links = [
    { url = "about", i18n_key = "about" },
    { url = "sitemap.xml", i18n_key = "sitemap", no_translation = true },
]
```

The value of the `i18n_key` must be in the `i18n` files for your languages (see [en.toml](i18n/en.toml), for example). If the parameter `no_translation` is set to true, than the URL is not adapted to contain the current language code. This is needed for external links or something like the `sitemap.xml` in the example, that is not translated within your website.

#### Social Icons

The social icons in the footer can be adapted by setting any of the following variables:

```toml
[extra.social]
codeberg = ""
github = ""
gitlab = ""
stackoverflow = ""
mastodon = ""
linkedin = ""
instagram = ""
youtube = ""
signal = ""
telegram = ""
email = ""
phone = ""
```

For every non-empty variable, the corresponding icon is shown in the footer.

#### Copyright Notice

The copyright notice in the footer can be set by adding the following variable in the configuration:

```toml
[extra.footer]
notice = "This is my <b>copyright</b> notice."
```

HTML can be used there.

### Syntax Highlighting

The theme makes use of Zola code highlighting feature and supports setting a different color scheme depending on whether a light or dark theme is active. Just enable syntax highlighting the following way:

```toml
highlight_code = true
highlight_theme = "css"
```

### Index Page

A title and text can be added to the index page by creating a file `_index.md` in the `content` directory. Additionally, a slogan and an image an be configured in the `config.toml`:

```toml
[extra.index]
slogan = "Slogan text that is shown under the title"
image = "portrait.png"
image_alt = "Placeholder text describing the index's image."
```

You can also created a completely different index page, by overwriting the `index.html` template in the template directory of your site. Just inherit from the `page.html` template of the theme.

## Details on Using the Theme

The installation of the theme works the same as for other Zola themes. As it is described in the [official documentation](https://www.getzola.org/documentation/themes/installing-and-using-themes/). Hence, it fist needs to be added as a git submodule:

```bash
cd my-zola-website
git submodule add -b main \
    https://codeberg.org/winterstein/zola-theme-project-portfolio.git \
    themes/project-portfolio
```

Please make sure to add it at the path `themes/project-portfolio` in your Zola directory. The translations and the icons won't work if added to a different directory.

In the `config.toml` file it needs to be selected then:

```toml
theme = "project-portfolio"
```

Create the files `projects.md` and `skills.md` in your `content` directory that are used to show the "Projects" and "Skills" [taxonomies](https://www.getzola.org/documentation/content/taxonomies/). They both need a title and can optionally get a descriptive text that will be shown above the terms of the taxonomy. See the following `projects.md` file as an example:

```markdown
+++
title = "Projects"
+++

The title and the text of this page can be adapted by changing the
`projects.md` file in the `content` directory.

Check out the amazing projects, by browsing through the industrial
sectors. The project do not need to be categorized by industries, but
could be distinguished by other topics instead. For example by frontend
and backend projects or by main responsibilities, like developer or
lead. Whatever makes most sense for your project portfolio.
```

It would be shown with the configured title and content like this above the terms:

![Screenshot of the projects taxonomy page](https://codeberg.org/winterstein/zola-theme-project-portfolio/raw/branch/main/screenshot-projects-taxonomy.png)

The `skills.md` file can be created the same way. The corresponding page will just show the terms of the skills taxonomy instead of the terms of the projects taxonomy then.

Finally, create the first project page in the `content` directory:

```markdown
+++
```

```toml
title = "Project Title"
description = "Here is a short description of the project."
date = 2022-05-31 # The date when the project finished

[extra]
date_start = 2021-01-01 # Optional date when the project was started
image = "water.jpg" # Optional filename to an image in the `static/images` directory
top_project = true # Optional parameter to show the project on the projects overview page as well

[taxonomies]
projects=["Consumer"] # The category of the project (could be industry, type etc.)
skills=["Thinking", "Hype Technology"] # The skills & technologies used for the project
```

```markdown
+++

The content of the project description page follows here.
```

The generated project site would then look like this:

![Screenshot of the project example page](https://codeberg.org/winterstein/zola-theme-project-portfolio/raw/branch/main/screenshot-project-example.png)

        