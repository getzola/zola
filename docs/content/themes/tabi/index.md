
+++
title = "tabi"
description = "A fast, lightweight, and modern Zola theme with optional JavaScript, and a perfect Lighthouse score."
template = "theme.html"
date = 2023-04-30T21:01:54+02:00

[extra]
created = 2023-04-30T21:01:54+02:00
updated = 2023-04-30T21:01:54+02:00
repository = "https://github.com/welpo/tabi.git"
homepage = "https://github.com/welpo/tabi"
minimum_version = "0.9.0"
license = "MIT"
demo = "https://welpo.github.io/tabi"

[extra.author]
name = "Óscar Fernández"
homepage = "https://welpo.ooo"
+++        

# tabi

A fast, lightweight, and modern [Zola](https://getzola.org) theme. It aims to be a personal page and home to blog posts.

See a live preview [here](https://welpo.github.io/tabi).

> tabi (旅): Journey.

![tabi](light_dark_screenshot.png)

tabi has a perfect score on Google's Lighthouse audit:

![lighthouse](lighthouse_score.png)

## Features

- [X] Dark and light themes. Defaults to the OS setting, with a switcher in the navigation bar.
- [X] Perfect Lighthouse score (Performance, Accessibility, Best Practices and SEO).
- [X] [KaTeX](https://katex.org/) support.
- [X] All JavaScript (theme switcher and KaTeX) can be fully disabled.
- [X] Responsive design.
- [X] Projects page.
- [X] Archive page.
- [x] Tags.
- [x] Social links.
- [X] Code syntax highlighting.
- [X] [Custom shortcodes](./templates/shortcodes/).
- [X] Customizable secure headers.

See the project's roadmap [here](https://github.com/users/welpo/projects/1).

## Quick start

```bash
git clone https://github.com/welpo/tabi.git
cd tabi
zola serve
```

Open http://127.0.0.1:1111/ in the browser.

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
header = {title = "Hello! I'm tabi~", img = "$BASE_URL/img/main.webp" }
```

The content outside the front matter will be rendered between the header title and the posts listing. In the screenshot above, it's the text that reads "tabi is a fast, lightweight, and modern Zola theme…".

## Inspiration

This theme was inspired by:
- [shadharon](https://github.com/syedzayyan/shadharon). tabi started as a fork of [syedzayyan](https://github.com/syedzayyan)'s theme.
- [tailwind-nextjs-starter-blog](https://github.com/timlrx/tailwind-nextjs-starter-blog)
- [tale-zola](https://github.com/aaranxu/tale-zola)
- [internetVin's blog](https://internetvin.ghost.io)

## Contributing

Please do! Take a look at the [Contributing Guidelines](/CONTRIBUTING.md) to learn more.

## License

The code is available under the [MIT license](./LICENSE).

        