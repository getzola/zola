
+++
title = "terminus"
description = "A dark duotone retro theme for Zola"
template = "theme.html"
date = 2025-12-28T23:40:32-05:00

[taxonomies]
theme-tags = ['dark', 'blog', 'minimal', 'personal', 'responsive', 'seo']

[extra]
created = 2025-12-28T23:40:32-05:00
updated = 2025-12-28T23:40:32-05:00
repository = "https://github.com/ebkalderon/terminus.git"
homepage = "https://github.com/ebkalderon/terminus"
minimum_version = "0.20.0"
license = "MIT"
demo = "https://ebkalderon.github.io/terminus/"

[extra.author]
name = "Eyal Kalderon"
homepage = "https://eyalkalderon.com"
+++        

# Terminus

An accessible [Zola](https://github.com/getzola/zola) theme with a dark color
scheme and retro computer terminal-like vibe, multi-language support, zero
required JavaScript, pretty font ligatures, and a perfect baseline Lighthouse
score.

**Try the demo now:** https://ebkalderon.github.io/terminus/

![Screenshot of the Terminus demo website on a desktop browser](https://github.com/user-attachments/assets/ae7c378b-2987-4dbd-a84e-7d272e8856bc)

Terminus is largely a port of Radek Kozieł's [Terminal Theme for
Hugo](https://github.com/panr/hugo-theme-terminal) but with several key
differences. Credit to the [zerm](https://github.com/ejmg/zerm) and
[tabi](https://github.com/welpo/tabi) themes for inspiring some of these
changes.

* Better accessibility (WCAG 2.2 Level AA minimum target)
* Mobile-first design with improved responsiveness
* Social media icons in footer
* Support for [GitHub-style alerts]
* SEO friendly (better OpenGraph support, will add Schema.org eventually)
* No post image previews for a cleaner look

[GitHub-style alerts]: https://ebkalderon.github.io/terminus/blog/shortcodes/#alert-shortcode

## Features

- [x] Perfect baseline Lighthouse score (Performance, Accessibility, Best Practices and SEO).
- [x] [Social media icons in footer](./theme.toml#L70-L73)
- [x] [Custom shortcodes](https://ebkalderon.github.io/terminus/blog/shortcodes/)
- [x] Copy button on code blocks
- [ ] [Comprehensive documentation] (still working on it!)
- [ ] Searchable archive page
- [ ] Projects portfolio page
- [ ] Site navigation submenus
- [x] Customizable [color schemes](./theme.toml#L22-L27)
- [x] [KaTeX](https://katex.org/) support for mathematical notation

[Comprehensive documentation]: https://ebkalderon.github.io/terminus/

## Getting Started

### Manual Installation

1. Initialize a Git repository in your [Zola project directory], if you haven't
   already:
   ```bash
   git init
   ```
2. Add the theme as a Git submodule:
   ```
   git submodule add https://github.com/ebkalderon/terminus.git themes/terminus
   ```
3. Enable the theme in your `config.toml`:
   ```toml
   theme = "terminus"
   ```
4. Set a website `title` in your  `config.toml`:
   ```toml
   title = "Your Site Title"
   ```
5. Create a text file named `content/_index.md`. This file controls how your
   home page looks and behaves. Choose _exactly one_ of the following options:
   1. **Serve blog posts from `/`:**
      ```markdown
      +++
      title = "Home"
      paginate_by = 5  # Show 5 posts per page.
      +++
      ```
   2. **Serve posts from a different path, e.g. `blog/`:**
      ```markdown
      +++
      title = "Home"

      [extra]
      section_path = "blog/_index.md"  # Where to find your posts.
      max_posts = 5  # Show 5 posts and a link to blog section on home page.
      +++
      ```

[Zola project directory]: https://www.getzola.org/documentation/getting-started/cli-usage/#init

### Updating Terminus

To update the Terminus theme as a Git submodule, run:

```bash
git submodule update --remote themes/terminus
```

## License

This project is licensed under the terms of the [MIT license](./LICENSE).

        