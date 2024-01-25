
+++
title = "Course"
description = "A zola theme designed for online courses or tutorials"
template = "theme.html"
date = 2024-01-25T10:41:35+02:00

[extra]
created = 2024-01-25T10:41:35+02:00
updated = 2024-01-25T10:41:35+02:00
repository = "https://github.com/elegaanz/zola-theme-course.git"
homepage = "https://github.com/elegaanz/zola-theme-course"
minimum_version = "0.17.1"
license = "GPL-3.0"
demo = "https://c.gelez.xyz/"

[extra.author]
name = "Ana Gelez"
homepage = "https://ana.gelez.xyz"
+++        

# Zola Theme : Course

This theme allows you to publish only courses/tutorials structured in
parts and subparts, using Zola.

![Homepage of the demo](screenshot.png)

![Page of a course](screenshot2.png)

It automatically links pages of the course with the next and previous ones
for easy navigation. There is also a navigation bar at the top of each page
to easily navigate the whole tutorial, and for the reader to never be lost.

Each page can have an illustration.

It lets you customize some parts of the site, like the color palette.

It also features a light/dark mode switcher.

It also has some SEO features.

It was made for french courses, so a few parts of the interface may be in french.
You can easily adapt it to your language by editing the files in `themes/zola-theme-course/templates/`.

## Usage

Create your Zola site, and import this theme:

```bash
zola init NAME
cd NAME/themes
git clone https://github.com/elegaanz/zola-theme-course.git
cd ..
```

Then update your `config.toml` with this line:

```toml
theme = "zola-theme-course"
```

You can also add these lines to customize how the theme behaves:

```toml
[extra]
site_name = "My course"
icon = "image.png"
icon_desc = "Icon of the course"
description = "A great course!"
default_illus = "illus.png"
primary_color = "#FFFFFF"
accent_color = "#FFFFFF"
source_url = "https://github.com/me/my-course"
```

### File structure

For your course to be displayed correctly, it needs to follow a specific structure.

- `content/_index.md` is the text displayed on the homepage
- each part should have its own folder, with an `_index.md`
- each subpart should have its own markdown file (which can be an `index.md` in a subfolder)
- all `_index.md` files should have `sort_by = "weight"` in their frontmatter, and you can then
  order parts and subparts using the `weight` option.

With this theme, pages can also have extra options:

```toml
[extra]
# Don't display the page title, useful for the homepage
no_title = true
# The name of the image to use as a banner
illus = "illus.jpg"
# Adds JSON-LD metadata on this page, useful for the homepage
jsonld = true
```

The standard `title` and `description` fields are also taken into account.
        