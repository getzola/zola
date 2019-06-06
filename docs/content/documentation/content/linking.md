+++
title = "Internal links & deep linking"
weight = 50
+++

## Heading id and anchor insertion
While rendering the markdown content, a unique id will automatically be assigned to each heading. This id is created
by converting the heading text to a [slug](https://en.wikipedia.org/wiki/Semantic_URL#Slug), appending numbers at the end
if the slug already exists for that article. For example:

```md
# Something exciting! <- something-exciting
## Example code <- example-code

# Something else <- something-else
## Example code <- example-code-1
```

You can also manually specify an id with a `{#…}` suffix on the heading line:

```md
# Something manual! {#manual}
```

This is useful for making deep links robust, either proactively (so that you can later change the text of a heading without breaking links to it) or retroactively (keeping the slug of the old header text, when changing the text). It can also be useful for migration of existing sites with different header id schemes, so that you can keep deep links working.

## Anchor insertion
It is possible to have Zola automatically insert anchor links next to the heading, as you can see on the site you are currently
reading if you hover a title.

This option is set at the section level: the `insert_anchor_links` variable on the
[Section front-matter page](@/documentation/content/section.md#front-matter).

The default template is very basic and will need CSS tweaks in your project to look decent.
If you want to change the anchor template, it can easily be overwritten by
creating a `anchor-link.html` file in the `templates` directory.

## Internal links
Linking to other pages and their headings is so common that Zola adds a
special syntax to Markdown links to handle them: start the link with `@/` and point to the `.md` file you want
to link to. The path to the file starts from the `content` directory.

For example, linking to a file located at `content/pages/about.md` would be `[my link](@/pages/about.md)`.
You can still link to an anchor directly: `[my link](@/pages/about.md#example)` will work as expected.
