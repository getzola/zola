
+++
title = "MATbook"
description = "A zola chapter book theme inspired from book and olivine"
template = "theme.html"
date = 2025-11-30T23:49:11-08:00

[taxonomies]
theme-tags = []

[extra]
created = 2025-11-30T23:49:11-08:00
updated = 2025-11-30T23:49:11-08:00
repository = "https://github.com/srliu3264/MATbook.git"
homepage = "https://github.com/srliu3264/MATbook"
minimum_version = "v1.0"
license = "MIT"
demo = "https://srliu3264.github.io/MATbook-live-demo/"

[extra.author]
name = "Shurui Liu"
homepage = "https://shurui.people.stanford.edu/"
+++        

# MATbook

A [Zola](https://github.com/getzola/zola) theme for personal notebooks or chapter books. Based on  [Vincent Prouillet](https://www.vincentprouillet.com/)'s [Book](https://github.com/getzola/book), inspired by [Dongryul Kim](https://web.stanford.edu/~dkim04/)'s [Olivine](https://github.com/dongryul-kim/olivine).

Live Demo: [https://srliu3264.github.io/MATbook-live-demo](https://srliu3264.github.io/MATbook-live-demo/)
## Contents

- MATbook
  - Features
  - Contents
  - Installation
  - Configurations
    - Enable the theme
    - Numbered chapters
    - Current section pages only
    - Math
    - Paths
    - Example toml

  - Usage
    - File structure
    - Hotkeys

## Features

- Idle inside Matrix (MATbook is short for Matrix Book)
- Searching
- Light/dark mode
- Book structure (table of chapters and sections in left sidebar, table of contents of current section in right sidebar)
- Keyboard shortcuts
- Backlinks
- Mathjax, Tikzjax, and basic theorem enviroments included for mathematics and commutative diagrams.

## Installation

Please follow the Zola documentation on [installing and using themes](https://www.getzola.org/documentation/themes/installing-and-using-themes/) to install.

## Configurations

### Enable the theme

```toml
theme = "MATbook"
build_search_index = true
```

### Numbered chapters

By default, the `MATbook` theme will number the chapters and pages in the left menu.
You can disable that by setting the `book_number_chapters` in `extra.booktheme`:

```toml
book_number_chapters = false
```

### Current section pages only

By default, the `MATbook` theme will list all the pages in the current section.
You can disable that by setting the `book_only_current_section_pages` in `extra.booktheme`:

```toml
book_only_current_section_pages = false
```

NOTE: you need to disabe this if you want to use hotkey `v` to toggle it.

### Math

Enable mathjax and tikzjax in `extra` if you need mathematics and tikzcd diagrams in your book.

```toml
tikzjax = true
mathjax = true
```

### Paths

You need to set up two paths:

First, in `[extra]`, you need to make sure that `upload_prefix` is the path to the directory where you put all images.

For example, if you put all your images in folder `/static/upload`, then you should set

```toml
upload_prefix = "/upload"
```

Second, Home link in `[extra.booktheme]`:

```toml
home_url="https://shurui.people.stanford.edu/"
```

It may be link to your homepage, or a guide/TOC page which collects links to all your books (in this way you can organize multiple books with this theme).

### Example toml

```toml
title = "Shurui Liu's Coding Notes"
description = "Personal Notes"
base_url = "https://web.stanford.edu/~srliu/Notes"
theme = "MATbook"

compile_sass = true
taxonomies = [
  { name = "tags", paginate_by = 10, rss = true }
]
build_search_index = true

[markdown]
highlight_code = true
highlight_theme = "css"
external_links_target_blank = true
smart_punctuation = true

[extra]
upload_prefix = "https://web.stanford.edu/~srliu/Notes/upload"
tikzjax = true
mathjax = true

[extra.booktheme]
book_number_chapters = true
book_only_current_section_pages = false
home_url = "https://shurui.sites.stanford.edu/"
```

## Usage

### File structure

All content should be put in `/content` folder as general Zola projects. Each chapter should be a folder inside `/content`, which contains its sections (markdown files). Here is an example of the file structure:

```markdown
.
├── chapter1
│   ├── _index.md
│   ├── section1.md
│   └── section2.md
├── chapter2
│   ├── _index.md
│   └── section1.md
├── chapter3
│   ├── _index.md
│   ├── section1.md
│   ├── section2.md
│   └── section3.md
└── _index.md
```

In `/content` folder, there should be one `_index.md`, which contains title and welcome/preface infomation of the book. In its front matter, you should set `sort_by = "weight" ` to manually control the order of chapters (or you can use sort by slug or date for your notebook).

Each chapter(folder) must have an `_index.md` file. It should sets its `weight` front-matter variable to its chapter number, and set `sort_by = "weight"` in its front matter.

Then in each chapter(folder), each section should be a `page` and should have its `weight` variable set to its section
number.

If you don't want welcome/preface page of the book or of a chapter, you can use `redirect_to` front-matter variable in the corresponding `_index.md`.

You can write a shell scripts to automate creating new chapters or new notes. For example, you can consult my `zshconfig` project on my [github](https://github.com/srliu3264) (If you can not see it, probably it is bacasue I currently make it private. You can email me for access.)

### Hotkeys

In a browser, you can always type `?` to toggle the help page, reminding you of hotkeys. 

The rule is inspired by Vim, Zathura, and [Olivine](https://github.com/dongryul-kim/olivine). 

The list of hotkeys may keep expanding, so I will not list all here. You can use `?` the help page, or see `hotkeys.js` manually to check existing functions.

The hotkey `i` will allow you to idle inside MATRIX (inspired by [cmatrix](https://github.com/abishekvashok/cmatrix)), and since MAT is both initials for matrix and math, I lazily named this project after it.

        