+++
title = "Index, Sections and Pages"
weight = 20
+++

First off, it is important to know that in Gutenberg the index 
page is actually a section like any other: you can add metadata
and content by adding `_index.md` at the root of the `content` folder.

Pages and sections are actually very similar.

## Page variables
By default, Gutenberg will try to load `templates/page.html`. If there isn't
one, it will render the built-in template: a blank page.

Whichever template you decide to render, you will get a `page` variable in your template
with the following fields:


```ts
content: String;
title: String?;
description: String?;
date: String?;
slug: String;
path: String;
permalink: String;
summary: String?;
tags: Array<String>;
category: String?;
extra: HashMap<String, Any>;
// Naive word count, will not work for languages without whitespace
word_count: Number;
// Based on https://help.medium.com/hc/en-us/articles/214991667-Read-time
reading_time: Number;
// `previous` and `next` are only filled if the content can be sorted
previous: Page?;
next: Page?;
// See the Table of contents section below for more details
toc: Array<Header>;
```

## Section variables
By default, Gutenberg will try to load `templates/section.html`. If there isn't
one, it will render the built-in template: a blank page.

Whichever template you decide to render, you will get a `section` variable in your template
with the following fields:


```ts
content: String;
title: String?;
description: String?;
date: String?;
slug: String;
path: String;
permalink: String;
extra: HashMap<String, Any>;
// Pages directly in this section, sorted if asked
pages: Array<Pages>;
// Direct subsections to this section, sorted by subsections weight
subsections: Array<Section>;
// Naive word count, will not work for languages without whitespace
word_count: Number;
// Based on https://help.medium.com/hc/en-us/articles/214991667-Read-time
reading_time: Number;
// See the Table of contents section below for more details
toc: Array<Header>;
```

## Table of contents

Both page and section have a `toc` field which corresponds to an array of `Header`.
A `Header` has the following fields:

```ts
// the hx level
level: 1 | 2 | 3 | 4 | 5 | 6;
// the generated slug id
id: String;
// the text of the header
title: String;
// a link pointing directly to the header, using the inserted anchor
permalink: String;
// all lower level headers below this header
children: Array<Header>;
```
