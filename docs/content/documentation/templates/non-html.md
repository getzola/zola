+++
title = "Non-HTML output"
weight = 100
+++

While Zola is best suited for generating HTML, it can be used for other output formats as well. Note that this does not have first class support and not all features will work.

## Output filename

Filenames are affected by

* Content filename, eg. `foo.md`
* Template extension, eg. `page.html`
* URL beautification, meaning files are always output as `index.*` to support paths without file extension.

With the example combination, the output file would be `foo/index.html`.

To change the output file extension, just change to a template with the desired extension.

## Whitespace

For text-based formats, the [whitespace control](https://tera.netlify.app/docs/#whitespace-control) is quite useful. An useful pattern is removing the whitespace at the end whenever there's a Tera delimiter at the end a line (that is, use `-}}` or `-%}`).

## Markdown rendering

The main focus of Zola is in rendering markdown to HTML. That's what the `markdown` filter does and `page.content` is the resulting HTML.

For non-HTML formats it's usually better to use `page.plain_content` which has Tera expressions and statements evaluated, but no HTML conversion.

Content files still need to be `*.md` and have a front matter.

## Always created files

Zola will always create `404.html`, `robots.txt` and `sitemap.xml`. See their respective documentation for customization information.

## Examples

See [`test_site_gemini/`](https://github.com/getzola/zola/tree/master/test_site_gemini) in the source code repository for a simple test site with [text/gemini](https://gemini.circumlunar.space/) output.
