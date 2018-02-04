extern crate pulldown_cmark;
extern crate rendering;

use pulldown_cmark::{Parser, Options, OPTION_ENABLE_FOOTNOTES};
use rendering::ast::{Content};
use rendering::ast_html::{into_html};

fn print_parser(parser: Parser) {
    for event in parser {
        println!("{:?}", event);
    }
    println!("\n\n");
}

#[test]
fn renders_text() {
    let original = r##"Hello"##;

    let p = Parser::new(&original);
    print_parser(p);
    let p = Parser::new(&original);

    let mut content = Content::new(p);
    let mut buf = String::new();
    into_html(&mut content, &mut buf);
    assert_eq!("<p>Hello</p>\n", buf);
}

#[test]
fn renders_multiple_text() {
    let original = r##"Hello

World
"##;

    let p = Parser::new(&original);
    print_parser(p);
    let p = Parser::new(&original);

    let mut content = Content::new(p);
    let mut buf = String::new();
    into_html(&mut content, &mut buf);
    assert_eq!("<p>Hello</p>\n<p>World</p>\n", buf);
}

#[test]
fn renders_html() {
    let original = r#"<article>
<header>This is an article header.</header>
</article>"#;

    let p = Parser::new(&original);
    print_parser(p);
    let p = Parser::new(&original);

    let mut content = Content::new(p);
    let mut buf = String::new();
    into_html(&mut content, &mut buf);
    assert_eq!(original, buf);

}

#[test]
fn renders_inline_html() {
    let original = r#"Hello, <em>World</em>!"#;

    let p = Parser::new(&original);
    print_parser(p);
    let p = Parser::new(&original);

    let mut content = Content::new(p);
    let mut buf = String::new();
    into_html(&mut content, &mut buf);
    assert_eq!("<p>Hello, <em>World</em>!</p>\n", buf);

}

#[test]
fn renders_footnotes() {
    let original = r#"Footnote reference.[^a]

[^a]: Footnote definition."#;

    let expected = r##"<p>Footnote reference.<sup class="footnote-reference"><a href="#a">1</a></sup></p>
<div class="footnote-definition" id="a"><sup class="footnote-definition-label">1</sup>
<p>Footnote definition.</p>
</div>
"##;

    let mut options = Options::empty();
    options.insert(OPTION_ENABLE_FOOTNOTES);

    let p = Parser::new_ext(&original, options);
    print_parser(p);
    let p = Parser::new_ext(&original, options);

    let mut content = Content::new(p);
    let mut buf = String::new();
    into_html(&mut content, &mut buf);
    assert_eq!(expected, buf);
}

#[test]
fn renders_headings() {
    let original = r##"# Hello

Hello Again
===========
"##;

    let expected = r#"<h1>Hello</h1>
<h1>Hello Again</h1>
"#;

    let p = Parser::new(&original);
    print_parser(p);
    let p = Parser::new(&original);

    let mut content = Content::new(p);
    let mut buf = String::new();
    into_html(&mut content, &mut buf);
    assert_eq!(expected, buf);
}

#[test]
fn renders_h2_headings() {
    let original = r##"## Hello

Hello Again
-----------
"##;

    let expected = r#"<h2>Hello</h2>
<h2>Hello Again</h2>
"#;

    let p = Parser::new(&original);
    print_parser(p);
    let p = Parser::new(&original);

    let mut content = Content::new(p);
    let mut buf = String::new();
    into_html(&mut content, &mut buf);
    assert_eq!(expected, buf);
}

#[test]
fn renders_code() {
    let original = r#"```javascript
let wat = 5 + '3';
```"#;

    let p = Parser::new(&original);
    print_parser(p);
    let p = Parser::new(&original);

    let mut content = Content::new(p);
    let mut buf = String::new();
    into_html(&mut content, &mut buf);
    assert_eq!("<pre><code>let wat = 5 + '3';\n</code></pre>\n", buf);
}
