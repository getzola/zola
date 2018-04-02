+++
title = "Overview"
weight = 10
+++


Gutenberg uses the folder structure to determine the site structure.
Each folder in the `content` directory represents a [section](./documentation/content/section.md)
that contains [pages](./documentation/content/page.md): your `.md` files.

```bash
.
└── content
    ├── content
    │   └── something.md // -> https://mywebsite.com/content/something/
    ├── blog
    │   ├── cli-usage.md // -> https://mywebsite.com/blog/cli-usage/
    │   ├── configuration.md // -> https://mywebsite.com/blog/configuration/
    │   ├── directory-structure.md // -> https://mywebsite.com/blog/directory-structure/
    │   ├── _index.md // -> https://mywebsite.com/blog/
    │   └── installation.md // -> https://mywebsite.com/blog/installation/
    └── landing
        └── _index.md // -> https://mywebsite.com/landing/
```

Each page path (the part after the `base_url`, for example `blog/cli-usage/`) can be customised by changing the `path` or `slug`
attribute of the [page front-matter](./documentation/content/page.md#front-matter).

You might have noticed a file named `_index.md` in the example above.
This file is used to store both metadata and content of the section itself and is not considered a page.

To make sure the terminology used in the rest of the documentation is understood, let's go over the example above.

The `content` directory in this case has three `sections`: `content`, `blog` and `landing`. The `content` section has only
one page, `something.md`, the `landing` section has no page and the `blog` section has 4 pages: `cli-usage.md`, `configuration.md`, `directory-structure.md`
and `installation.md`.

While not shown in the example, sections can be nested indefinitely.

## Assets colocation

The `content` directory is not limited to markup files though: it's natural to want to co-locate a page and some related
assets.

Gutenberg supports that pattern out of the box: create a folder, add a `index.md` file and as many non-markdown files as you want.
Those assets will be copied in the same folder when building the site which allows you to use a relative path to access them.

```bash
└── with-assets
    ├── index.md
    └── yavascript.js
```

By default, this page will get the folder name (`with-assets` in this case) as its slug.

It is possible to ignore selected asset files using the
[ignored_content](./documentation/getting-started/configuration.md) setting in the config file.
For example, say you have an Excel spreadsheet from which you are taking several screenshots and
then linking to those image files on your website. For maintainability purposes, you want to keep
the spreadsheet in the same folder as the markdown, but you don't want to copy the spreadsheet to
the public web site. You can achieve this by simply setting `ignored_content` in the config file:

```
ignored_content = ["*.xlsx"]
```
