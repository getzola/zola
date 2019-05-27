+++
title = "Overview"
weight = 10
+++


Zola uses the folder structure to determine the site structure.
Each folder in the `content` directory represents a [section](@/documentation/content/section.md)
that contains [pages](@/documentation/content/page.md): your `.md` files.

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
attribute of the [page front-matter](@/documentation/content/page.md#front-matter).

You might have noticed a file named `_index.md` in the example above.
This file is used to store both metadata and content of the section itself and is not considered a page.

To make sure the terminology used in the rest of the documentation is understood, let's go over the example above.

The `content` directory in this case has three `sections`: `content`, `blog` and `landing`. The `content` section has only
one page, `something.md`, the `landing` section has no page and the `blog` section has 4 pages: `cli-usage.md`, `configuration.md`, `directory-structure.md`
and `installation.md`.

While not shown in the example, sections can be nested indefinitely.

## Assets colocation

The `content` directory is not limited to markup files though: it's natural to want to co-locate a page and some related
assets, for instance images or spreadsheets. Zola supports that pattern out of the box for both sections and pages.

Any non-markdown file you add in the page/section folder will be copied alongside the generated page when building the site,
which allows us to use a relative path to access them.

For pages to use assets colocation, they should not be placed directly in their section folder (such as `latest-experiment.md`), but as an `index.md` file
in a dedicated folder (`latest-experiment/index.md`), like so:


```bash
└── research
    ├── latest-experiment
    │   ├── index.md
    │   └── yavascript.js
    ├── _index.md
    └── research.jpg
```

In this setup, you may access `research.jpg` from your 'research' section,
and `yavascript.js` from your 'latest-experiment' directly within the Markdown:

```markdown
Check out the complete program [here](yavascript.js). It's **really cool free-software**!
```

By default, this page will get the folder name as its slug. So its permalink would be in the form of `https://example.com/research/latest-experiment/`

### Excluding files from assets

It is possible to ignore selected asset files using the
[ignored_content](@/documentation/getting-started/configuration.md) setting in the config file.
For example, say you have an Excel spreadsheet from which you are taking several screenshots and
then linking to those image files on your website. For maintainability purposes, you want to keep
the spreadsheet in the same folder as the markdown, but you don't want to copy the spreadsheet to
the public web site. You can achieve this by simply setting `ignored_content` in the config file:

```
ignored_content = ["*.xlsx"]
```

## Static assets

In addition to placing content files in the `content` directory, you may also place content
files in the `static` directory.  Any files/folders that you place in the `static` directory
will be copied, without modification, to the public directory.

Typically, you might put site-wide assets (such as the site favicon, site logos or site-wide
JavaScript) in the root of the static directory.  You can also place any HTML or other files that
you wish to be included without modification (that is, without being parsed as Markdown files)
into the static directory.

Note that the static folder provides an _alternative_ to colocation.  For example, imagine that you
had the following directory structure (a simplified version of the structure presented above):

```bash
.
└── content
    └── blog
        ├── configuration
        │    └── index.md // -> https://mywebsite.com/blog/configuration/
        └── _index.md // -> https://mywebsite.com/blog/
```

If you wanted to add an image to the `https://mywebsite.com/blog/configuration` page, you would
have three options:
 *  You could save the image to the `content/blog/configuration` folder and then link it with a
 relative path from the `index.md` page.  This is the approach described under **colocation**,
 above.
 *  You could save the image to a `static/blog/configuration` folder and link it in exactly the
 same way as if you had colocated it.  If you do this, the generated files will be identical to
 if you had colocated; the only difference will be that all static files will be saved in the
 static folder rather than in the content folder.  Depending on your organizational needs, this
 may be better or worse.
 *  Or you could save the image to some arbitrary folder within the static folder.  For example,
 you could save all images to `static/images`.  Using this approach, you would no longer be able
 to use relative links, but could use an absolute link to `images/[filename]` to access your
 image.  This might be preferable for small sites or for sites that associate images with
 multiple pages (e.g., logo images that appear on every page).
