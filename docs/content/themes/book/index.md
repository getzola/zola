
+++
title = "book"
description = "A book theme inspired from GitBook/mdBook"
template = "theme.html"
date = 2018-01-28T10:53:19+01:00

[extra]
created = 2018-11-17T18:27:11+01:00
updated = 2018-01-28T10:53:19+01:00
repository = "https://github.com/getzola/book"
homepage = "https://github.com/getzola/book"
minimum_version = "0.5.0"
license = "MIT"
demo = "https://zola-book.netlify.com"

[extra.author]
name = "Vincent Prouillet"
homepage = "https://www.vincentprouillet.com"
+++        

# book

A theme based on [Gitbook](https://www.gitbook.com), to write documentation
or books.

![book screenshot](https://github.com/Keats/book/blob/master/screenshot.png?raw=true)


## Contents

- [Installation](#installation)
- [Options](#options)
  - [Numbered chapters](#numbered-chapters)

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
You can disable that by setting the `book_numbered_chapters` in `extra`:

```toml
book_numbered_chapters = false
```

        