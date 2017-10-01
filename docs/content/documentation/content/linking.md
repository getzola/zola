+++
title = "Internal links & deep linking"
weight = 50
+++

## Header id and anchor insertion
While rendering the markdown content, a unique id will automatically be assigned to each header. This id is created
by converting the header text to a [slug](https://en.wikipedia.org/wiki/Semantic_URL#Slug), appending numbers at the end
if the slug already exists for that article. For example:

```md
# Something exciting! <- something-exciting
## Example code <- example-code

# Something else <- something-else
## Example code <- example-code-1
```

## Anchor insertion
It is possible to have Gutenberg automatically insert anchor links next to the header, as you can see on the site you are currently 
reading if you hover a title.

This option is set at the section level, look up the `insert_anchor_links` variable on the 
[Section front-matter page](./documentation/content/section.md#front-matter).

The default template is very basic and will need CSS tweaks in your project to look decent. 
If you want to change the anchor template, it can easily be overwritten by 
creating a `anchor-link.html` file in the `templates` directory.

## Internal links
Linking to other pages and their headers is so common that Gutenberg adds a 
special syntax to Markdown links to handle them: start the link with `./` and point to the `.md` file you want
to link to. The path to the file starts from the `content` directory.

For example, linking to a file located at `content/pages/about.md` would be `[my link](./pages/about.md)`.
You can still link to a header directly: `[my link](./pages/about.md#example)` would work as expected, granted
the `example` id exists on the header.
