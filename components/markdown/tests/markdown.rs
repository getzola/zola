use std::collections::HashMap;

use libs::tera::Tera;

use config::Config;
use markdown::{render_content, RenderContext};
use templates::ZOLA_TERA;
use utils::slugs::SlugifyStrategy;
use utils::types::InsertAnchor;

mod common;

#[test]
fn can_render_basic_markdown() {
    let cases = vec![
        "Hello world",
        "# Hello world",
        "Hello *world*",
        "Hello\n\tworld",
        "Non rendered emoji :smile:",
        "[a link](image.jpg)",
        "![alt text](image.jpg)",
        "<h1>some html</h1>",
    ];

    let body = common::render(&cases.join("\n")).unwrap().body;
    insta::assert_snapshot!(body);
}

#[test]
fn can_make_zola_internal_links() {
    let body = common::render(
        r#"
[rel link](@/pages/about.md)
[rel link with anchor](@/pages/about.md#cv)
[abs link](https://getzola.org/about/)
    "#,
    )
    .unwrap()
    .body;
    insta::assert_snapshot!(body);
}

#[test]
fn can_handle_heading_ids() {
    let mut config = Config::default_for_test();

    let cases = vec![
        // Basic
        "# Hello",
        // Same slug as first
        "# Hello",
        // not a slug because of the slugify strategy chosen
        "# L'écologie et vous",
        // Chosen slug that already exists without space
        "# Hello{#hello}",
        // Chosen slug that already exists with space
        "# Hello {#hello}",
        "# Hello     {#Something_else}",
        "# Workaround for literal {#…&#125;",
        "# Auto {#*matic*}",
        // and now some empty heading
        "# ",
        "# ",
        // zola internal links
        "# [About](@/pages/about.md)",
        // https://github.com/Keats/gutenberg/issues/297
        "# [Rust](https://rust-lang.org \"Rust homepage\")",
        // and then some markdown in them
        "# `hi`",
        "# *hi*",
        "# **hi**",
        // See https://github.com/getzola/zola/issues/569
        "# text [^1] there\n[^1]: footnote",
        // Chosen slug that already exists with space
        "# Classes {#classes .bold .another}",
    ];
    let body = common::render_with_config(&cases.join("\n"), config.clone()).unwrap().body;
    insta::assert_snapshot!(body);

    // And now test without slugifying everything
    config.slugify.anchors = SlugifyStrategy::Safe;
    let body = common::render_with_config(&cases.join("\n"), config).unwrap().body;
    insta::assert_snapshot!(body);
}

#[test]
fn can_insert_anchors() {
    let cases = vec![
        // Basic
        "# Hello\n# World",
        // https://github.com/Keats/gutenberg/issues/42
        "# Hello!",
        // https://github.com/Keats/gutenberg/issues/53
        "## [Rust](https://rust-lang.org)",
        "# Hello*_()",
    ];
    let body =
        common::render_with_insert_anchor(&cases.join("\n"), InsertAnchor::Left).unwrap().body;
    insta::assert_snapshot!(body);
    let body =
        common::render_with_insert_anchor(&cases.join("\n"), InsertAnchor::Right).unwrap().body;
    insta::assert_snapshot!(body);
    let body =
        common::render_with_insert_anchor(&cases.join("\n"), InsertAnchor::Heading).unwrap().body;
    insta::assert_snapshot!(body);
}

#[test]
fn can_customise_anchor_template() {
    let mut tera = Tera::default();
    tera.extend(&ZOLA_TERA).unwrap();
    tera.add_raw_template("anchor-link.html", " (in {{ lang }})").unwrap();
    let permalinks_ctx = HashMap::new();
    let config = Config::default_for_test();
    let context = RenderContext::new(
        &tera,
        &config,
        &config.default_language,
        "",
        &permalinks_ctx,
        InsertAnchor::Right,
    );
    let body = render_content("# Hello", &context).unwrap().body;
    insta::assert_snapshot!(body);
}

#[test]
fn can_use_smart_punctuation() {
    let mut config = Config::default_for_test();
    config.markdown.smart_punctuation = true;
    let body = common::render_with_config(r#"This -- is "it"..."#, config).unwrap().body;
    insta::assert_snapshot!(body);
}

#[test]
fn can_use_external_links_options() {
    let mut config = Config::default_for_test();

    // no options
    let body = common::render("<https://google.com>").unwrap().body;
    insta::assert_snapshot!(body);

    // target blank
    config.markdown.external_links_target_blank = true;
    let body = common::render_with_config("<https://google.com>", config.clone()).unwrap().body;
    insta::assert_snapshot!(body);

    // no follow
    config.markdown.external_links_target_blank = false;
    config.markdown.external_links_no_follow = true;
    let body = common::render_with_config("<https://google.com>", config.clone()).unwrap().body;
    insta::assert_snapshot!(body);

    // no referrer
    config.markdown.external_links_no_follow = false;
    config.markdown.external_links_no_referrer = true;
    let body = common::render_with_config("<https://google.com>", config.clone()).unwrap().body;
    insta::assert_snapshot!(body);

    // all of them
    config.markdown.external_links_no_follow = true;
    config.markdown.external_links_target_blank = true;
    config.markdown.external_links_no_referrer = true;
    let body = common::render_with_config("<https://google.com>", config).unwrap().body;
    insta::assert_snapshot!(body);
}

#[test]
fn can_render_emojis() {
    let mut config = Config::default_for_test();
    config.markdown.render_emoji = true;
    let body = common::render_with_config("Hello, World! :smile:", config).unwrap().body;
    assert_eq!(body, "<p>Hello, World! 😄</p>\n");
}

// https://github.com/getzola/zola/issues/747
// https://github.com/getzola/zola/issues/816
#[test]
fn custom_url_schemes_are_untouched() {
    let body = common::render(
        r#"
[foo@bar.tld](xmpp:foo@bar.tld)

[(123) 456-7890](tel:+11234567890)

[blank page](about:blank)
    "#,
    )
    .unwrap()
    .body;
    insta::assert_snapshot!(body);
}

#[test]
fn all_markdown_features_integration() {
    let body = common::render(
        r#"
<!-- Adapted from https://markdown-it.github.io/ -->

# h1 Heading

## h2 Heading

### h3 Heading

#### h4 Heading

##### h5 Heading

###### h6 Heading

## Horizontal Rules

___

---

***

## Emphasis

**This is bold text**

__This is bold text__

*This is italic text*

_This is italic text_

~~Strikethrough~~


## Blockquotes


> Blockquotes can also be nested...
>> ...by using additional greater-than signs right next to each other...
> > > ...or with spaces between arrows.


## Lists

Unordered

+ Create a list by starting a line with `+`, `-`, or `*`
+ Sub-lists are made by indenting 2 spaces:
  - Marker character change forces new list start:
    * Ac tristique libero volutpat at
    + Facilisis in pretium nisl aliquet
    - Nulla volutpat aliquam velit
+ Very easy!

Ordered

1. Lorem ipsum dolor sit amet
2. Consectetur adipiscing elit
3. Integer molestie lorem at massa


1. You can use sequential numbers...
1. ...or keep all the numbers as `1.`

Start numbering with offset:

57. foo
1. bar


## Code

Inline `code`

Indented code

    // Some comments
    line 1 of code
    line 2 of code
    line 3 of code


Block code "fences"

```
Sample text here...
```

Syntax highlighting

``` js
var foo = function (bar) {
  return bar++;
};

console.log(foo(5));
```

## Shortcodes

## Tables

| Option | Description |
| ------ | ----------- |
| data   | path to data files to supply the data that will be passed into templates. |
| engine | engine to be used for processing templates. Handlebars is the default. |
| ext    | extension to be used for dest files. |

Right aligned columns

| Option | Description |
| ------:| -----------:|
| data   | path to data files to supply the data that will be passed into templates. |
| engine | engine to be used for processing templates. Handlebars is the default. |
| ext    | extension to be used for dest files. |


## Links

[link text](http://duckduckgo.com)

[link with title](http://duckduckgo.com/ "Duck duck go")

## Images

![Minion](https://octodex.github.com/images/minion.png)
![Stormtroopocat](https://octodex.github.com/images/stormtroopocat.jpg "The Stormtroopocat")

Like links, Images also have a footnote style syntax

![Alt text][id]

With a reference later in the document defining the URL location:

[id]: https://octodex.github.com/images/dojocat.jpg  "The Dojocat"

## Smileys

Like :smile:, :cry:

### Footnotes

Footnote 1 link[^first].

Footnote 2 link[^second].

Duplicated footnote reference[^second].

[^first]: Footnote **can have markup**
and multiple paragraphs.

[^second]: Footnote text.
    "#,
    )
    .unwrap()
    .body;
    insta::assert_snapshot!(body);
}

#[test]
fn github_style_footnotes() {
    let mut config = Config::default_for_test();
    config.markdown.bottom_footnotes = true;

    let markdown = r#"This text has a footnote[^1]

[^1]:But it is meaningless.

This text has two[^3] footnotes[^2].

[^2]: not sorted.
[^3]: But they are

[^4]:It's before the reference.

There is footnote definition?[^4]

This text has two[^5] identical footnotes[^5]
[^5]: So one is present.
[^6]: But another in not.

This text has a footnote[^7]

[^7]: But the footnote has another footnote[^8].

[^8]: That's it.

Footnotes can also be referenced with identifiers[^first].

[^first]: Like this: `[^first]`.
"#;

    let body = common::render_with_config(&markdown, config).unwrap().body;
    insta::assert_snapshot!(body);
}
