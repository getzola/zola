
+++
title = "book"
description = "A book theme inspired from GitBook/mdBook"
template = "theme.html"
date = 2025-03-14T12:12:57-07:00

[taxonomies]
theme-tags = []

[extra]
created = 2025-03-14T12:12:57-07:00
updated = 2025-03-14T12:12:57-07:00
repository = "https://github.com/getzola/book.git"
homepage = "https://github.com/getzola/book"
minimum_version = "0.17.0"
license = "MIT"
demo = "https://getzola.github.io/book/"

[extra.author]
name = "Vincent Prouillet"
homepage = "https://www.vincentprouillet.com"
+++        

# book

A theme based on [Gitbook](https://www.gitbook.com), to write documentation
or books.

![book screenshot](https://github.com/Keats/book/blob/master/screenshot.png?raw=true)


## Contents

- Installation
- Options
  - Numbered chapters

## Installation
First download this theme to your `themes` directory:

```bash
$ cd themes
$ git clone https://github.com/getzola/book.git
```
and then enable it in your `config.toml`:

```toml
theme = "book"
# Optional, if you want search
build_search_index = true
```

## Usage
Book will generate a book from the files you place in the `content` directory.  Your book
can have two levels of hierarchy: chapters and subchapters.

Each chapter should be a `section` within the Gutenberg site and should have an `_index.md`
file that sets its `weight` front-matter variable to its chapter number.  For example,
chapter 2 should have `weight = 2`.  Additionally, each chapter should also set the
`sort_by = "weight"` in its front matter.

Each subchapter should be a `page` and should have its `weight` variable set to the subchapter
number.  For example, subchapter 3.4 should have `weight = 4`.

Finally, you should create an `_index.md` file and set the `redirect_to` front-matter variable
to redirect to the first section of your content.  For example, if your first section has the
slug `introduction`, then you would set `redirect_to = "introduction"`.

## Options

### Numbered chapters
By default, the `book` theme will number the chapters and pages in the left menu.
You can disable that by setting the `book_number_chapters` in `extra`:

```toml
book_number_chapters = false
```

### Current section pages only
By default, the `book` theme will list all the pages in the current section.
You can disable that by setting the `book_only_current_section_pages` in `extra`:

```toml
book_only_current_section_pages = false
```

        