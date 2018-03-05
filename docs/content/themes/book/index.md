
+++
title = "book"
description = "A book theme inspired from GitBook/mdBook"
template = "theme.html"
date = 2018-01-28T10:53:19+01:00

[extra]
created = 2018-02-22T19:13:36+01:00
updated = 2018-01-28T10:53:19+01:00
repository = "https://github.com/Keats/book"
homepage = "https://github.com/Keats/book"
minimum_version = "0.2"
license = "MIT"

[extra.author]
name = "Vincent Prouillet"
homepage = "https://vincent.is"
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
$ git clone https://github.com/Keats/book.git
```
and then enable it in your `config.toml`:

```toml
theme = "book"
```

## Options

### Numbered chapters
By default, the `book` theme will number the chapters and pages in the left menu.
You can disable that by setting the `book_numbered_chapters` in `extra`:

```toml
book_numbered_chapters = false
```

        