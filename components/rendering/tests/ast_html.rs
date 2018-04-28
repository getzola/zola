extern crate pulldown_cmark;
extern crate rendering;

use pulldown_cmark::{Parser, Options, OPTION_ENABLE_FOOTNOTES, OPTION_ENABLE_TABLES};
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
    let original = "Hello";
    let expected = "<p>Hello</p>";

    let p = Parser::new(&original);
    print_parser(p);
    let p = Parser::new(&original);

    let mut content = Content::new(p);
    let mut buf = String::new();
    into_html(&mut content, &mut buf);
    assert_eq!(expected, buf);
}

#[test]
fn renders_multiple_text() {
    let original = r#"Hello

World
"#;
    let expected = "<p>Hello</p><p>World</p>";

    let p = Parser::new(&original);
    print_parser(p);
    let p = Parser::new(&original);

    let mut content = Content::new(p);
    let mut buf = String::new();
    into_html(&mut content, &mut buf);
    assert_eq!(expected, buf);
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
    let original = "Hello, <em>World</em>!";
    let expected = "<p>Hello, <em>World</em>!</p>";

    let p = Parser::new(&original);
    print_parser(p);
    let p = Parser::new(&original);

    let mut content = Content::new(p);
    let mut buf = String::new();
    into_html(&mut content, &mut buf);
    assert_eq!(expected, buf);

}

#[test]
fn renders_footnotes() {
    let original = r#"Footnote reference.[^a]

[^a]: Footnote definition."#;
    let expected = r##"<p>Footnote reference.<sup class="footnote-reference"><a href="#a">1</a></sup></p><div class="footnote-definition" id="a"><sup class="footnote-definition-label">1</sup><p>Footnote definition.</p></div>"##;

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
fn renders_table_with_only_thead() {
    let original = r##"Test|Table
----|-----
"##;
    let expected = r#"<table><thead><tr><th>Test</th><th>Table</th></tr></thead><tbody></tbody></table>"#;
    let mut options = Options::empty();
    options.insert(OPTION_ENABLE_TABLES);

    let p = Parser::new_ext(&original, options);
    print_parser(p);
    let p = Parser::new_ext(&original, options);

    let mut content = Content::new(p);
    let mut buf = String::new();
    into_html(&mut content, &mut buf);
    assert_eq!(expected, buf);
}

#[test]
fn renders_table_with_body_rows() {
    let original = r#"Test|Table
----|-----
Test|row 1
Test|row 2

"#;
    let expected = "<table><thead><tr><th>Test</th><th>Table</th></tr></thead><tbody><tr><td>Test</td><td>row 1</td></tr><tr><td>Test</td><td>row 2</td></tr></tbody></table>";
    let mut options = Options::empty();
    options.insert(OPTION_ENABLE_TABLES);

    let p = Parser::new_ext(&original, options);
    print_parser(p);
    let p = Parser::new_ext(&original, options);

    let mut content = Content::new(p);
    let mut buf = String::new();
    into_html(&mut content, &mut buf);
    assert_eq!(expected, buf);

}

#[test]
fn renders_table_with_body_row_spanning_columns() {
    let original = r#"Test|Table
----|-----
Test row

"#;
    let expected = "<table><thead><tr><th>Test</th><th>Table</th></tr></thead><tbody><tr><td>Test row</td></tr></tbody></table>";
    let mut options = Options::empty();
    options.insert(OPTION_ENABLE_TABLES);

    let p = Parser::new_ext(&original, options);
    print_parser(p);
    let p = Parser::new_ext(&original, options);

    let mut content = Content::new(p);
    let mut buf = String::new();
    into_html(&mut content, &mut buf);
    assert_eq!(expected, buf);
}

#[test]
fn renders_table_with_alignments() {
    let original = r#"Four|Types|Of|Columns
:---|-----|::|------:
Left|None|Center|Right
"#;
    let expected = r#"<table><thead><tr><th style="text-align: left;">Four</th><th>Types</th><th style="text-align: center;">Of</th><th style="text-align: right;">Columns</th></tr></thead><tbody><tr><td style="text-align: left;">Left</td><td>None</td><td style="text-align: center;">Center</td><td style="text-align: right;">Right</td></tr></tbody></table>"#;
    let mut options = Options::empty();
    options.insert(OPTION_ENABLE_TABLES);

    let p = Parser::new_ext(&original, options);
    print_parser(p);
    let p = Parser::new_ext(&original, options);

    let mut content = Content::new(p);
    let mut buf = String::new();
    into_html(&mut content, &mut buf);
    assert_eq!(expected, buf);

}

#[test]
fn renders_table_with_unspecified_alignments() {
    let original = r#"Two|Types
:---|----:
Of|Alignment|More|Columns
"#;
    let expected = r#"<table><thead><tr><th style="text-align: left;">Two</th><th style="text-align: right;">Types</th></tr></thead><tbody><tr><td style="text-align: left;">Of</td><td style="text-align: right;">Alignment</td><td>More</td><td>Columns</td></tr></tbody></table>"#;
    let mut options = Options::empty();
    options.insert(OPTION_ENABLE_TABLES);

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
    let expected = "<h1>Hello</h1><h1>Hello Again</h1>";

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
    let expected = "<h2>Hello</h2><h2>Hello Again</h2>";

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
    let expected = "<pre><code>let wat = 5 + '3';\n</code></pre>";

    let p = Parser::new(&original);
    print_parser(p);
    let p = Parser::new(&original);

    let mut content = Content::new(p);
    let mut buf = String::new();
    into_html(&mut content, &mut buf);
    assert_eq!(expected, buf);
}
