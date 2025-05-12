
+++
title = "anemone"
description = "A minimalist Zola theme that prioritizes clean CSS and avoids heavy JavaScript. Enjoy a seamless user experience with lightning-fast load times. Let your content take center stage in a clutter-free, elegant design that enhances readability. Responsive and efficient, anemone brings focus to your ideas."
template = "theme.html"
date = 2025-04-25T01:55:45+01:00

[taxonomies]
theme-tags = []

[extra]
created = 2025-04-25T01:55:45+01:00
updated = 2025-04-25T01:55:45+01:00
repository = "https://github.com/Speyll/anemone.git"
homepage = "https://github.com/Speyll/anemone"
minimum_version = "0.4.0"
license = "MIT"
demo = "https://anemone.pages.dev"

[extra.author]
name = "Speyll"
homepage = "https://speyllsite.pages.dev/"
+++        

# anemone

Introducing "anemone," a minimalist [Zola](https://www.getzola.org) theme that prioritizes clean CSS and avoids heavy JavaScript. Enjoy a seamless user experience with lightning-fast load times. Let your content take center stage in a clutter-free, elegant design that enhances readability. Responsive and efficient, anemone brings focus to your ideas.

You can browse the demo website [here](https://anemone.pages.dev/)
I also use it on my own [website.](https://speyllsite.pages.dev/)

Anemone is a versatile Zola theme that comes with both light and dark variants. You can easily switch between the light and dark themes to suit your preferences.

![Anemone Light and Dark Theme](screenshot.png)

## Installation

To get started with Anemone, follow these simple steps:

1. Download the theme to your `themes` directory:

```bash
cd themes
git clone https://github.com/Speyll/anemone
```

2. Enable Anemone in your `config.toml`:

```toml
theme = "anemone"
```

## Release Notes

#### 2025-04-09

This release introduces a **complete rewrite** of the project: simplified, improved, and optimized across the board.

**If you are updating from an older release:**
1. Open your `config.toml` file and update it as needed (compare with the latest release for reference).
2. Remove the following line from `content/blog/_index.md`:
   ```toml
   page_template = "blog-page.html"
   ```

#### 2024-03-02
This release brings several improvements and enhancements, focusing mainly on optimizing performance and user experience. Here's a summary of the key changes:

- **suCSS Integration:** The core CSS now leverages the lightweight [suCSS framework](https://speyll.github.io/suCSS/) made by yours truly, providing better maintainability, robustness, and scalability. With suCSS, the theme should maintain consistent appearance across different browsers.

- **Enhanced Theme Toggle:** The dark and light theme toggle has been revamped for more consistency. Now, the website respects the user's system-wide theme settings, ensuring a seamless experience. Additionally, the toggle retains the selected theme for future visits, offering improved usability.

- **Smooth Transition and Sound Effect:** Enjoy a smoother transition between the dark and light mode accompanied by a subtle sound effect. Rest assured, the added sound effect incurs minimal performance overhead, with the file size being just 1kb.

- **Class Names and Shortcodes Update:** Some class names and shortcodes have been modified for better organization and clarity. I apologize for any inconvenience this may cause.

- **Slight change in Color Choice:** Some dark mode colors have been changed for the sake of readability, still using [veqev](https://github.com/Speyll/veqev).


## Options

Anemone provides various options to customize your website:

#### Default Taxonomies

To use tags, add the following code to a page's metadata:

```toml
[taxonomies]
tags = ["tag1", "tag2"]
```

#### Pages List in Homepage

Enable listing of pages in the homepage by adding the following code to `config.toml`:

```toml
[extra]
list_pages = true
```

#### Multilanguage

The theme has a built-in feature that allows you to use multiple languages. For detailed instructions on how to use this feature, you can refer to the [Zola Multilingual documentation](https://www.getzola.org/documentation/content/multilingual/). This documentation provides additional information on how to make the most out of this multilingual capability.

```toml
[languages.fr]
weight = 2
title = "anemone"
languageName = "Français"
languageCode = "fr"
```
#### Multilanguage-Ready Navigation Bar

Customize the header navigation links with the following code in the `extra` section of `config.toml`:

```toml
[extra]

header_nav = [
  { url = "/", name_en = "/home/", name_fr = "/accueil/" },
  { url = "/about", name_en = "/about/", name_fr = "/concernant/" },
  { url = "/journal", name_en = "/journal/", name_fr = "/journal/" },
  { url = "/blog", name_en = "/blog/", name_fr = "/blog/" }
]
```

#### Add Table of Contents (TOC) to Pages

In a page's frontmatter, set `extra.toc` to `true`:

```toml
[extra]
toc = true
```

#### Display Author Name in Blog Posts

Customize the display of the author's name in your blog posts by toggling the `display_author` variable to either `true` or `false`:

```toml
[extra]
display_author = true
```

### Webrings

Add a webring with a shortcode:

```html
{{/* webring(prev="#", webring="#", webringName="Random Webring", next="#") */}}
```

### Extra Data

- Set the `author` in both the main config and in pages metadata.
- Similarly, set `favicon` in the main config, and it will be used as the site icon.
- Set `footer_content_license` and `footer_content_license_link` if you wish to display content license information in the footer.


### License

The Anemone theme is available as open source under the terms of the [MIT License](LICENSE).

        