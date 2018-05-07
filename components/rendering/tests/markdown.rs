extern crate tera;
extern crate front_matter;
extern crate templates;
extern crate rendering;
extern crate config;

use std::collections::HashMap;

use tera::Tera;

use config::Config;
use front_matter::InsertAnchor;
use templates::GUTENBERG_TERA;
use rendering::{RenderContext, render_content};


#[test]
fn can_do_render_content_simple() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("hello", &context).unwrap();
    assert_eq!(res.0, "<p>hello</p>\n");
}

#[test]
fn doesnt_highlight_code_block_with_highlighting_off() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let mut config = Config::default();
    config.highlight_code = false;
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("```\n$ gutenberg server\n```", &context).unwrap();
    assert_eq!(
        res.0,
        "<pre><code>$ gutenberg server\n</code></pre>\n"
    );
}

#[test]
fn can_highlight_code_block_no_lang() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("```\n$ gutenberg server\n$ ping\n```", &context).unwrap();
    assert_eq!(
        res.0,
        "<pre style=\"background-color:#2b303b\">\n<span style=\"background-color:#2b303b;color:#c0c5ce;\">$ gutenberg server\n</span><span style=\"background-color:#2b303b;color:#c0c5ce;\">$ ping\n</span></pre>"
    );
}

#[test]
fn can_highlight_code_block_with_lang() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("```python\nlist.append(1)\n```", &context).unwrap();
    assert_eq!(
        res.0,
        "<pre style=\"background-color:#2b303b\">\n<span style=\"background-color:#2b303b;color:#c0c5ce;\">list.</span><span style=\"background-color:#2b303b;color:#bf616a;\">append</span><span style=\"background-color:#2b303b;color:#c0c5ce;\">(</span><span style=\"background-color:#2b303b;color:#d08770;\">1</span><span style=\"background-color:#2b303b;color:#c0c5ce;\">)\n</span></pre>"
    );
}

#[test]
fn can_higlight_code_block_with_unknown_lang() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("```yolo\nlist.append(1)\n```", &context).unwrap();
    // defaults to plain text
    assert_eq!(
        res.0,
        "<pre style=\"background-color:#2b303b\">\n<span style=\"background-color:#2b303b;color:#c0c5ce;\">list.append(1)\n</span></pre>"
    );
}

#[test]
fn can_render_shortcode() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&GUTENBERG_TERA, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content(r#"
Hello

{{ youtube(id="ub36ffWAqgQ") }}
    "#, &context).unwrap();
    assert!(res.0.contains("<p>Hello</p>\n<div >"));
    assert!(res.0.contains(r#"<iframe src="https://www.youtube.com/embed/ub36ffWAqgQ""#));
}

#[test]
fn can_render_shortcode_with_markdown_char_in_args_name() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&GUTENBERG_TERA, &config, "", &permalinks_ctx, InsertAnchor::None);
    let input = vec![
        "name",
        "na_me",
        "n_a_me",
        "n1",
    ];
    for i in input {
        let res = render_content(&format!("{{{{ youtube(id=\"hey\", {}=1) }}}}", i), &context).unwrap();
        assert!(res.0.contains(r#"<iframe src="https://www.youtube.com/embed/hey""#));
    }
}

#[test]
fn can_render_shortcode_with_markdown_char_in_args_value() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&GUTENBERG_TERA, &config, "", &permalinks_ctx, InsertAnchor::None);
    let input = vec![
        "ub36ffWAqgQ-hey",
        "ub36ffWAqgQ_hey",
        "ub36ffWAqgQ_he_y",
        "ub36ffWAqgQ*hey",
        "ub36ffWAqgQ#hey",
    ];
    for i in input {
        let res = render_content(&format!("{{{{ youtube(id=\"{}\") }}}}", i), &context).unwrap();
        assert!(res.0.contains(&format!(r#"<iframe src="https://www.youtube.com/embed/{}""#, i)));
    }
}

#[test]
fn can_render_body_shortcode_with_markdown_char_in_name() {
    let permalinks_ctx = HashMap::new();
    let mut tera = Tera::default();
    tera.extend(&GUTENBERG_TERA).unwrap();
    let input = vec![
        "quo_te",
        "qu_o_te",
    ];
    let config = Config::default();

    for i in input {
        tera.add_raw_template(&format!("shortcodes/{}.html", i), "<blockquote>{{ body }} - {{ author}}</blockquote>").unwrap();
        let context = RenderContext::new(&tera, &config, "", &permalinks_ctx, InsertAnchor::None);

        let res = render_content(&format!("{{% {}(author=\"Bob\") %}}\nhey\n{{% end %}}", i), &context).unwrap();
        println!("{:?}", res);
        assert!(res.0.contains("<blockquote>hey - Bob</blockquote>"));
    }
}

#[test]
fn can_render_body_shortcode_and_paragraph_after() {
    let permalinks_ctx = HashMap::new();
    let mut tera = Tera::default();
    tera.extend(&GUTENBERG_TERA).unwrap();

    let shortcode = "<p>{{ body }}</p>";
    let markdown_string = r#"
{% figure() %}
This is a figure caption.
{% end %}

Here is another paragraph.
"#;

    let expected = "<p>This is a figure caption.</p>
<p>Here is another paragraph.</p>
";

    tera.add_raw_template(&format!("shortcodes/{}.html", "figure"), shortcode).unwrap();
    let config = Config::default();
    let context = RenderContext::new(&tera, &config, "", &permalinks_ctx, InsertAnchor::None);

    let res = render_content(markdown_string, &context).unwrap();
    println!("{:?}", res);
    assert_eq!(res.0, expected);
}

#[test]
fn can_render_two_body_shortcode_and_paragraph_after_with_line_break_between() {
    let permalinks_ctx = HashMap::new();
    let mut tera = Tera::default();
    tera.extend(&GUTENBERG_TERA).unwrap();

    let shortcode = "<p>{{ body }}</p>";
    let markdown_string = r#"
{% figure() %}
This is a figure caption.
{% end %}

{% figure() %}
This is a figure caption.
{% end %}

Here is another paragraph.
"#;

    let expected = "<p>This is a figure caption.</p>
<p>This is a figure caption.</p>
<p>Here is another paragraph.</p>
";

    tera.add_raw_template(&format!("shortcodes/{}.html", "figure"), shortcode).unwrap();
    let config = Config::default();
    let context = RenderContext::new(&tera, &config, "", &permalinks_ctx, InsertAnchor::None);

    let res = render_content(markdown_string, &context).unwrap();
    println!("{:?}", res);
    assert_eq!(res.0, expected);
}

#[test]
fn can_render_several_shortcode_in_row() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&GUTENBERG_TERA, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content(r#"
Hello

{{ youtube(id="ub36ffWAqgQ") }}

{{ youtube(id="ub36ffWAqgQ", autoplay=true) }}

{{ vimeo(id="210073083") }}

{{ streamable(id="c0ic") }}

{{ gist(url="https://gist.github.com/Keats/32d26f699dcc13ebd41b") }}

    "#, &context).unwrap();
    assert!(res.0.contains("<p>Hello</p>\n<div >"));
    assert!(res.0.contains(r#"<iframe src="https://www.youtube.com/embed/ub36ffWAqgQ""#));
    assert!(res.0.contains(r#"<iframe src="https://www.youtube.com/embed/ub36ffWAqgQ?autoplay=1""#));
    assert!(res.0.contains(r#"<iframe src="https://www.streamable.com/e/c0ic""#));
    assert!(res.0.contains(r#"//player.vimeo.com/video/210073083""#));
}

#[test]
fn doesnt_render_ignored_shortcodes() {
    let permalinks_ctx = HashMap::new();
    let mut config = Config::default();
    config.highlight_code = false;
    let context = RenderContext::new(&GUTENBERG_TERA, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content(r#"```{{/* youtube(id="w7Ft2ymGmfc") */}}```"#, &context).unwrap();
    assert_eq!(res.0, "<p><code>{{ youtube(id=&quot;w7Ft2ymGmfc&quot;) }}</code></p>\n");
}

#[test]
fn can_render_shortcode_with_body() {
    let mut tera = Tera::default();
    tera.extend(&GUTENBERG_TERA).unwrap();
    tera.add_raw_template("shortcodes/quote.html", "<blockquote>{{ body }} - {{ author }}</blockquote>").unwrap();
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&tera, &config, "", &permalinks_ctx, InsertAnchor::None);

    let res = render_content(r#"
Hello
{% quote(author="Keats") %}
A quote
{% end %}
    "#, &context).unwrap();
    assert_eq!(res.0, "<p>Hello</p>\n<blockquote>A quote - Keats</blockquote>\n");
}

#[test]
fn errors_rendering_unknown_shortcode() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("{{ hello(flash=true) }}", &context);
    assert!(res.is_err());
}

#[test]
fn can_make_valid_relative_link() {
    let mut permalinks = HashMap::new();
    permalinks.insert("pages/about.md".to_string(), "https://vincent.is/about".to_string());
    let tera_ctx = Tera::default();
    let config = Config::default();
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks, InsertAnchor::None);
    let res = render_content(
        r#"[rel link](./pages/about.md), [abs link](https://vincent.is/about)"#,
        &context
    ).unwrap();

    assert!(
        res.0.contains(r#"<p><a href="https://vincent.is/about">rel link</a>, <a href="https://vincent.is/about">abs link</a></p>"#)
    );
}

#[test]
fn can_make_relative_links_with_anchors() {
    let mut permalinks = HashMap::new();
    permalinks.insert("pages/about.md".to_string(), "https://vincent.is/about".to_string());
    let tera_ctx = Tera::default();
    let config = Config::default();
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks, InsertAnchor::None);
    let res = render_content(r#"[rel link](./pages/about.md#cv)"#, &context).unwrap();

    assert!(
        res.0.contains(r#"<p><a href="https://vincent.is/about#cv">rel link</a></p>"#)
    );
}

#[test]
fn errors_relative_link_inexistant() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("[rel link](./pages/about.md)", &context);
    assert!(res.is_err());
}

#[test]
fn can_add_id_to_headers() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content(r#"# Hello"#, &context).unwrap();
    assert_eq!(res.0, "<h1 id=\"hello\">Hello</h1>\n");
}

#[test]
fn can_add_id_to_headers_same_slug() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("# Hello\n# Hello", &context).unwrap();
    assert_eq!(res.0, "<h1 id=\"hello\">Hello</h1>\n<h1 id=\"hello-1\">Hello</h1>\n");
}

#[test]
fn can_insert_anchor_left() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&GUTENBERG_TERA, &config, "", &permalinks_ctx, InsertAnchor::Left);
    let res = render_content("# Hello", &context).unwrap();
    assert_eq!(
        res.0,
        "<h1 id=\"hello\"><a class=\"gutenberg-anchor\" href=\"#hello\" aria-label=\"Anchor link for: hello\">🔗</a>\nHello</h1>\n"
    );
}

#[test]
fn can_insert_anchor_right() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&GUTENBERG_TERA, &config, "", &permalinks_ctx, InsertAnchor::Right);
    let res = render_content("# Hello", &context).unwrap();
    assert_eq!(
        res.0,
        "<h1 id=\"hello\">Hello<a class=\"gutenberg-anchor\" href=\"#hello\" aria-label=\"Anchor link for: hello\">🔗</a>\n</h1>\n"
    );
}

// See https://github.com/Keats/gutenberg/issues/42
#[test]
fn can_insert_anchor_with_exclamation_mark() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&GUTENBERG_TERA, &config, "", &permalinks_ctx, InsertAnchor::Left);
    let res = render_content("# Hello!", &context).unwrap();
    assert_eq!(
        res.0,
        "<h1 id=\"hello\"><a class=\"gutenberg-anchor\" href=\"#hello\" aria-label=\"Anchor link for: hello\">🔗</a>\nHello!</h1>\n"
    );
}

// See https://github.com/Keats/gutenberg/issues/53
#[test]
fn can_insert_anchor_with_link() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&GUTENBERG_TERA, &config, "", &permalinks_ctx, InsertAnchor::Left);
    let res = render_content("## [Rust](https://rust-lang.org)", &context).unwrap();
    assert_eq!(
        res.0,
        "<h2 id=\"rust\"><a class=\"gutenberg-anchor\" href=\"#rust\" aria-label=\"Anchor link for: rust\">🔗</a>\n<a href=\"https://rust-lang.org\">Rust</a></h2>\n"
    );
}

#[test]
fn can_insert_anchor_with_other_special_chars() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&GUTENBERG_TERA, &config, "", &permalinks_ctx, InsertAnchor::Left);
    let res = render_content("# Hello*_()", &context).unwrap();
    assert_eq!(
        res.0,
        "<h1 id=\"hello\"><a class=\"gutenberg-anchor\" href=\"#hello\" aria-label=\"Anchor link for: hello\">🔗</a>\nHello*_()</h1>\n"
    );
}

#[test]
fn can_make_toc() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(
        &GUTENBERG_TERA,
        &config,
        "https://mysite.com/something",
        &permalinks_ctx,
        InsertAnchor::Left
    );

    let res = render_content(r#"
# Header 1

## Header 2

## Another Header 2

### Last one
    "#, &context).unwrap();

    let toc = res.1;
    assert_eq!(toc.len(), 1);
    assert_eq!(toc[0].children.len(), 2);
    assert_eq!(toc[0].children[1].children.len(), 1);

}

#[test]
fn can_understand_backtick_in_titles() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&GUTENBERG_TERA, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("# `Hello`", &context).unwrap();
    assert_eq!(
        res.0,
        "<h1 id=\"hello\"><code>Hello</code></h1>\n"
    );
}

#[test]
fn can_understand_backtick_in_paragraphs() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&GUTENBERG_TERA, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("Hello `world`", &context).unwrap();
    assert_eq!(
        res.0,
        "<p>Hello <code>world</code></p>\n"
    );
}

// https://github.com/Keats/gutenberg/issues/297
#[test]
fn can_understand_links_in_header() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&GUTENBERG_TERA, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("# [Rust](https://rust-lang.org)", &context).unwrap();
    assert_eq!(
        res.0,
        "<h1 id=\"rust\"><a href=\"https://rust-lang.org\">Rust</a></h1>\n"
    );
}

#[test]
fn can_understand_link_with_title_in_header() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&GUTENBERG_TERA, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("# [Rust](https://rust-lang.org \"Rust homepage\")", &context).unwrap();
    assert_eq!(
        res.0,
        "<h1 id=\"rust\"><a href=\"https://rust-lang.org\" title=\"Rust homepage\">Rust</a></h1>\n"
    );
}

#[test]
fn can_make_valid_relative_link_in_header() {
    let mut permalinks = HashMap::new();
    permalinks.insert("pages/about.md".to_string(), "https://vincent.is/about/".to_string());
    let tera_ctx = Tera::default();
    let config = Config::default();
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks, InsertAnchor::None);
    let res = render_content(
        r#" # [rel link](./pages/about.md)"#,
        &context
    ).unwrap();

    assert_eq!(
        res.0,
        "<h1 id=\"rel-link\"><a href=\"https://vincent.is/about/\">rel link</a></h1>\n"
    );
}


#[test]
fn can_make_permalinks_with_colocated_assets() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&GUTENBERG_TERA, &config, "https://vincent.is/about/", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("[an image](image.jpg)", &context).unwrap();
    assert_eq!(
        res.0,
        "<p><a href=\"https://vincent.is/about/image.jpg\">an image</a></p>\n"
    );
}
