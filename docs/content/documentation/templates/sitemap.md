+++
title = "Sitemap"
weight = 60
+++

Gutenberg will look for a `sitemap.xml` file in the `templates` directory or 
use the built-in one.


The sitemap template gets four variables in addition of the config:

- `pages`: all pages of the site
- `sections`: all sections of the site, including an index section
- `tags`: links the tags page and individual tag page, empty if no tags
- `categories`: links the categories page and individual category page, empty if no categories

As the sitemap only requires a link and an optional date for the `lastmod` field,
all the variables above are arrays of `SitemapEntry` with the following type:

```ts
permalink: String;
date: String?;
```

All `SitemapEntry` are sorted in each variable by their permalink.
