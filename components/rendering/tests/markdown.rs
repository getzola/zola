extern crate config;
extern crate front_matter;
extern crate rendering;
extern crate templates;
extern crate tera;

use std::collections::HashMap;

use tera::Tera;

use config::Config;
use front_matter::InsertAnchor;
use rendering::{render_content, RenderContext};
use templates::ZOLA_TERA;

#[test]
fn can_do_render_content_simple() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("hello", &context).unwrap();
    assert_eq!(res.body, "<p>hello</p>\n");
}

#[test]
fn doesnt_highlight_code_block_with_highlighting_off() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let mut config = Config::default();
    config.highlight_code = false;
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("```\n$ gutenberg server\n```", &context).unwrap();
    assert_eq!(res.body, "<pre><code>$ gutenberg server\n</code></pre>\n");
}

#[test]
fn can_highlight_code_block_no_lang() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let mut config = Config::default();
    config.highlight_code = true;
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("```\n$ gutenberg server\n$ ping\n```", &context).unwrap();
    assert_eq!(
        res.body,
        "<pre style=\"background-color:#2b303b;\">\n<span style=\"color:#c0c5ce;\">$ gutenberg server\n$ ping\n</span></pre>"
    );
}

#[test]
fn can_highlight_code_block_with_lang() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let mut config = Config::default();
    config.highlight_code = true;
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("```python\nlist.append(1)\n```", &context).unwrap();
    assert_eq!(
        res.body,
        "<pre style=\"background-color:#2b303b;\">\n<span style=\"color:#c0c5ce;\">list.</span><span style=\"color:#bf616a;\">append</span><span style=\"color:#c0c5ce;\">(</span><span style=\"color:#d08770;\">1</span><span style=\"color:#c0c5ce;\">)\n</span></pre>"
    );
}

#[test]
fn can_higlight_code_block_with_unknown_lang() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let mut config = Config::default();
    config.highlight_code = true;
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("```yolo\nlist.append(1)\n```", &context).unwrap();
    // defaults to plain text
    assert_eq!(
        res.body,
        "<pre style=\"background-color:#2b303b;\">\n<span style=\"color:#c0c5ce;\">list.append(1)\n</span></pre>"
    );
}

#[test]
fn can_render_shortcode() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&ZOLA_TERA, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content(
        r#"
Hello

{{ youtube(id="ub36ffWAqgQ") }}
    "#,
        &context,
    )
    .unwrap();
    assert!(res.body.contains("<p>Hello</p>\n<div >"));
    assert!(res.body.contains(r#"<iframe src="https://www.youtube.com/embed/ub36ffWAqgQ""#));
}

#[test]
fn can_render_shortcode_with_markdown_char_in_args_name() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&ZOLA_TERA, &config, "", &permalinks_ctx, InsertAnchor::None);
    let input = vec!["name", "na_me", "n_a_me", "n1"];
    for i in input {
        let res =
            render_content(&format!("{{{{ youtube(id=\"hey\", {}=1) }}}}", i), &context).unwrap();
        assert!(res.body.contains(r#"<iframe src="https://www.youtube.com/embed/hey""#));
    }
}

#[test]
fn can_render_shortcode_with_markdown_char_in_args_value() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&ZOLA_TERA, &config, "", &permalinks_ctx, InsertAnchor::None);
    let input = vec![
        "ub36ffWAqgQ-hey",
        "ub36ffWAqgQ_hey",
        "ub36ffWAqgQ_he_y",
        "ub36ffWAqgQ*hey",
        "ub36ffWAqgQ#hey",
    ];
    for i in input {
        let res = render_content(&format!("{{{{ youtube(id=\"{}\") }}}}", i), &context).unwrap();
        assert!(res
            .body
            .contains(&format!(r#"<iframe src="https://www.youtube.com/embed/{}""#, i)));
    }
}

#[test]
fn can_render_body_shortcode_with_markdown_char_in_name() {
    let permalinks_ctx = HashMap::new();
    let mut tera = Tera::default();
    tera.extend(&ZOLA_TERA).unwrap();
    let input = vec!["quo_te", "qu_o_te"];
    let config = Config::default();

    for i in input {
        tera.add_raw_template(
            &format!("shortcodes/{}.html", i),
            "<blockquote>{{ body }} - {{ author}}</blockquote>",
        )
        .unwrap();
        let context = RenderContext::new(&tera, &config, "", &permalinks_ctx, InsertAnchor::None);

        let res =
            render_content(&format!("{{% {}(author=\"Bob\") %}}\nhey\n{{% end %}}", i), &context)
                .unwrap();
        println!("{:?}", res);
        assert!(res.body.contains("<blockquote>hey - Bob</blockquote>"));
    }
}

#[test]
fn can_render_body_shortcode_and_paragraph_after() {
    let permalinks_ctx = HashMap::new();
    let mut tera = Tera::default();
    tera.extend(&ZOLA_TERA).unwrap();

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
    assert_eq!(res.body, expected);
}

#[test]
fn can_render_two_body_shortcode_and_paragraph_after_with_line_break_between() {
    let permalinks_ctx = HashMap::new();
    let mut tera = Tera::default();
    tera.extend(&ZOLA_TERA).unwrap();

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
    assert_eq!(res.body, expected);
}

#[test]
fn can_render_several_shortcode_in_row() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&ZOLA_TERA, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content(
        r#"
Hello

{{ youtube(id="ub36ffWAqgQ") }}

{{ youtube(id="ub36ffWAqgQ", autoplay=true) }}

{{ vimeo(id="210073083") }}

{{ streamable(id="c0ic") }}

{{ gist(url="https://gist.github.com/Keats/32d26f699dcc13ebd41b") }}

    "#,
        &context,
    )
    .unwrap();
    assert!(res.body.contains("<p>Hello</p>\n<div >"));
    assert!(res.body.contains(r#"<iframe src="https://www.youtube.com/embed/ub36ffWAqgQ""#));
    assert!(res
        .body
        .contains(r#"<iframe src="https://www.youtube.com/embed/ub36ffWAqgQ?autoplay=1""#));
    assert!(res.body.contains(r#"<iframe src="https://www.streamable.com/e/c0ic""#));
    assert!(res.body.contains(r#"//player.vimeo.com/video/210073083""#));
}

#[test]
fn doesnt_render_ignored_shortcodes() {
    let permalinks_ctx = HashMap::new();
    let mut config = Config::default();
    config.highlight_code = false;
    let context = RenderContext::new(&ZOLA_TERA, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content(r#"```{{/* youtube(id="w7Ft2ymGmfc") */}}```"#, &context).unwrap();
    assert_eq!(res.body, "<p><code>{{ youtube(id=&quot;w7Ft2ymGmfc&quot;) }}</code></p>\n");
}

#[test]
fn can_render_shortcode_with_body() {
    let mut tera = Tera::default();
    tera.extend(&ZOLA_TERA).unwrap();
    tera.add_raw_template(
        "shortcodes/quote.html",
        "<blockquote>{{ body }} - {{ author }}</blockquote>",
    )
    .unwrap();
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&tera, &config, "", &permalinks_ctx, InsertAnchor::None);

    let res = render_content(
        r#"
Hello
{% quote(author="Keats") %}
A quote
{% end %}
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(res.body, "<p>Hello</p>\n<blockquote>A quote - Keats</blockquote>\n");
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
        r#"[rel link](@/pages/about.md), [abs link](https://vincent.is/about)"#,
        &context,
    )
    .unwrap();

    assert!(
        res.body.contains(r#"<p><a href="https://vincent.is/about">rel link</a>, <a href="https://vincent.is/about">abs link</a></p>"#)
    );
}

#[test]
fn can_make_relative_links_with_anchors() {
    let mut permalinks = HashMap::new();
    permalinks.insert("pages/about.md".to_string(), "https://vincent.is/about".to_string());
    let tera_ctx = Tera::default();
    let config = Config::default();
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks, InsertAnchor::None);
    let res = render_content(r#"[rel link](@/pages/about.md#cv)"#, &context).unwrap();

    assert!(res.body.contains(r#"<p><a href="https://vincent.is/about#cv">rel link</a></p>"#));
}

#[test]
fn errors_relative_link_inexistant() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("[rel link](@/pages/about.md)", &context);
    assert!(res.is_err());
}

#[test]
fn can_add_id_to_headers() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content(r#"# Hello"#, &context).unwrap();
    assert_eq!(res.body, "<h1 id=\"hello\">Hello</h1>\n");
}

#[test]
fn can_add_id_to_headers_same_slug() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("# Hello\n# Hello", &context).unwrap();
    assert_eq!(res.body, "<h1 id=\"hello\">Hello</h1>\n<h1 id=\"hello-1\">Hello</h1>\n");
}

#[test]
fn can_handle_manual_ids_on_headers() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks_ctx, InsertAnchor::None);
    // Tested things: manual IDs; whitespace flexibility; that automatic IDs avoid collision with
    // manual IDs; that duplicates are in fact permitted among manual IDs; that any non-plain-text
    // in the middle of `{#…}` will disrupt it from being acknowledged as a manual ID (that last
    // one could reasonably be considered a bug rather than a feature, but test it either way); one
    // workaround for the improbable case where you actually want `{#…}` at the end of a header.
    let res = render_content(
        "\
         # Hello\n\
         # Hello{#hello}\n\
         # Hello {#hello}\n\
         # Hello     {#Something_else} \n\
         # Workaround for literal {#…&#125;\n\
         # Hello\n\
         # Auto {#*matic*}",
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
        "\
         <h1 id=\"hello-1\">Hello</h1>\n\
         <h1 id=\"hello\">Hello</h1>\n\
         <h1 id=\"hello\">Hello</h1>\n\
         <h1 id=\"Something_else\">Hello</h1>\n\
         <h1 id=\"workaround-for-literal\">Workaround for literal {#…}</h1>\n\
         <h1 id=\"hello-2\">Hello</h1>\n\
         <h1 id=\"auto-matic\">Auto {#<em>matic</em>}</h1>\n\
         "
    );
}

#[test]
fn blank_headers() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("# \n#\n# {#hmm} \n# {#}", &context).unwrap();
    assert_eq!(
        res.body,
        "<h1 id=\"-1\"></h1>\n<h1 id=\"-2\"></h1>\n<h1 id=\"hmm\"></h1>\n<h1 id=\"\"></h1>\n"
    );
}

#[test]
fn can_insert_anchor_left() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&ZOLA_TERA, &config, "", &permalinks_ctx, InsertAnchor::Left);
    let res = render_content("# Hello", &context).unwrap();
    assert_eq!(
        res.body,
        "<h1 id=\"hello\"><a class=\"zola-anchor\" href=\"#hello\" aria-label=\"Anchor link for: hello\">🔗</a>\nHello</h1>\n"
    );
}

#[test]
fn can_insert_anchor_right() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&ZOLA_TERA, &config, "", &permalinks_ctx, InsertAnchor::Right);
    let res = render_content("# Hello", &context).unwrap();
    assert_eq!(
        res.body,
        "<h1 id=\"hello\">Hello<a class=\"zola-anchor\" href=\"#hello\" aria-label=\"Anchor link for: hello\">🔗</a>\n</h1>\n"
    );
}

#[test]
fn can_insert_anchor_for_multi_header() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&ZOLA_TERA, &config, "", &permalinks_ctx, InsertAnchor::Right);
    let res = render_content("# Hello\n# World", &context).unwrap();
    assert_eq!(
        res.body,
        "<h1 id=\"hello\">Hello<a class=\"zola-anchor\" href=\"#hello\" aria-label=\"Anchor link for: hello\">🔗</a>\n</h1>\n\
<h1 id=\"world\">World<a class=\"zola-anchor\" href=\"#world\" aria-label=\"Anchor link for: world\">🔗</a>\n</h1>\n"
    );
}

// See https://github.com/Keats/gutenberg/issues/42
#[test]
fn can_insert_anchor_with_exclamation_mark() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&ZOLA_TERA, &config, "", &permalinks_ctx, InsertAnchor::Left);
    let res = render_content("# Hello!", &context).unwrap();
    assert_eq!(
        res.body,
        "<h1 id=\"hello\"><a class=\"zola-anchor\" href=\"#hello\" aria-label=\"Anchor link for: hello\">🔗</a>\nHello!</h1>\n"
    );
}

// See https://github.com/Keats/gutenberg/issues/53
#[test]
fn can_insert_anchor_with_link() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&ZOLA_TERA, &config, "", &permalinks_ctx, InsertAnchor::Left);
    let res = render_content("## [Rust](https://rust-lang.org)", &context).unwrap();
    assert_eq!(
        res.body,
        "<h2 id=\"rust\"><a class=\"zola-anchor\" href=\"#rust\" aria-label=\"Anchor link for: rust\">🔗</a>\n<a href=\"https://rust-lang.org\">Rust</a></h2>\n"
    );
}

#[test]
fn can_insert_anchor_with_other_special_chars() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&ZOLA_TERA, &config, "", &permalinks_ctx, InsertAnchor::Left);
    let res = render_content("# Hello*_()", &context).unwrap();
    assert_eq!(
        res.body,
        "<h1 id=\"hello\"><a class=\"zola-anchor\" href=\"#hello\" aria-label=\"Anchor link for: hello\">🔗</a>\nHello*_()</h1>\n"
    );
}

#[test]
fn can_make_toc() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(
        &ZOLA_TERA,
        &config,
        "https://mysite.com/something",
        &permalinks_ctx,
        InsertAnchor::Left,
    );

    let res = render_content(
        r#"
# Header 1

## Header 2

## Another Header 2

### Last one
    "#,
        &context,
    )
    .unwrap();

    let toc = res.toc;
    assert_eq!(toc.len(), 1);
    assert_eq!(toc[0].children.len(), 2);
    assert_eq!(toc[0].children[1].children.len(), 1);
}

#[test]
fn can_ignore_tags_in_toc() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(
        &ZOLA_TERA,
        &config,
        "https://mysite.com/something",
        &permalinks_ctx,
        InsertAnchor::Left,
    );

    let res = render_content(
        r#"
## header with `code`

## [anchor](https://duckduckgo.com/) in header

## **bold** and *italics*
    "#,
        &context,
    )
    .unwrap();

    let toc = res.toc;

    assert_eq!(toc[0].id, "header-with-code");
    assert_eq!(toc[0].title, "header with code");

    assert_eq!(toc[1].id, "anchor-in-header");
    assert_eq!(toc[1].title, "anchor in header");

    assert_eq!(toc[2].id, "bold-and-italics");
    assert_eq!(toc[2].title, "bold and italics");
}

#[test]
fn can_understand_backtick_in_titles() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&ZOLA_TERA, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("# `Hello`", &context).unwrap();
    assert_eq!(res.body, "<h1 id=\"hello\"><code>Hello</code></h1>\n");
}

#[test]
fn can_understand_backtick_in_paragraphs() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&ZOLA_TERA, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("Hello `world`", &context).unwrap();
    assert_eq!(res.body, "<p>Hello <code>world</code></p>\n");
}

// https://github.com/Keats/gutenberg/issues/297
#[test]
fn can_understand_links_in_header() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&ZOLA_TERA, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("# [Rust](https://rust-lang.org)", &context).unwrap();
    assert_eq!(res.body, "<h1 id=\"rust\"><a href=\"https://rust-lang.org\">Rust</a></h1>\n");
}

#[test]
fn can_understand_link_with_title_in_header() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&ZOLA_TERA, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res =
        render_content("# [Rust](https://rust-lang.org \"Rust homepage\")", &context).unwrap();
    assert_eq!(
        res.body,
        "<h1 id=\"rust\"><a href=\"https://rust-lang.org\" title=\"Rust homepage\">Rust</a></h1>\n"
    );
}

#[test]
fn can_understand_emphasis_in_header() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&ZOLA_TERA, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("# *Emphasis* text", &context).unwrap();
    assert_eq!(res.body, "<h1 id=\"emphasis-text\"><em>Emphasis</em> text</h1>\n");
}

#[test]
fn can_understand_strong_in_header() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&ZOLA_TERA, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("# **Strong** text", &context).unwrap();
    assert_eq!(res.body, "<h1 id=\"strong-text\"><strong>Strong</strong> text</h1>\n");
}

#[test]
fn can_understand_code_in_header() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&ZOLA_TERA, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("# `Code` text", &context).unwrap();
    assert_eq!(res.body, "<h1 id=\"code-text\"><code>Code</code> text</h1>\n");
}

// See https://github.com/getzola/zola/issues/569
#[test]
fn can_understand_footnote_in_header() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&ZOLA_TERA, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content("# text [^1] there\n[^1]: footnote", &context).unwrap();
    assert_eq!(res.body, r##"<h1 id="text-there">text <sup class="footnote-reference"><a href="#1">1</a></sup> there</h1>
<div class="footnote-definition" id="1"><sup class="footnote-definition-label">1</sup>
<p>footnote</p>
</div>
"##);
}

#[test]
fn can_make_valid_relative_link_in_header() {
    let mut permalinks = HashMap::new();
    permalinks.insert("pages/about.md".to_string(), "https://vincent.is/about/".to_string());
    let tera_ctx = Tera::default();
    let config = Config::default();
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks, InsertAnchor::None);
    let res = render_content(r#" # [rel link](@/pages/about.md)"#, &context).unwrap();

    assert_eq!(
        res.body,
        "<h1 id=\"rel-link\"><a href=\"https://vincent.is/about/\">rel link</a></h1>\n"
    );
}

#[test]
fn can_make_permalinks_with_colocated_assets_for_link() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(
        &ZOLA_TERA,
        &config,
        "https://vincent.is/about/",
        &permalinks_ctx,
        InsertAnchor::None,
    );
    let res = render_content("[an image](image.jpg)", &context).unwrap();
    assert_eq!(res.body, "<p><a href=\"https://vincent.is/about/image.jpg\">an image</a></p>\n");
}

#[test]
fn can_make_permalinks_with_colocated_assets_for_image() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(
        &ZOLA_TERA,
        &config,
        "https://vincent.is/about/",
        &permalinks_ctx,
        InsertAnchor::None,
    );
    let res = render_content("![alt text](image.jpg)", &context).unwrap();
    assert_eq!(
        res.body,
        "<p><img src=\"https://vincent.is/about/image.jpg\" alt=\"alt text\" /></p>\n"
    );
}

#[test]
fn markdown_doesnt_wrap_html_in_paragraph() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(
        &ZOLA_TERA,
        &config,
        "https://vincent.is/about/",
        &permalinks_ctx,
        InsertAnchor::None,
    );
    let res = render_content(
        r#"
Some text

<h1>Helo</h1>

<div>
<a href="mobx-flow.png">
        <img src="mobx-flow.png" alt="MobX flow">
    </a>
</div>
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
        "<p>Some text</p>\n<h1>Helo</h1>\n<div>\n<a href=\"mobx-flow.png\">\n        <img src=\"mobx-flow.png\" alt=\"MobX flow\">\n    </a>\n</div>\n"
    );
}

#[test]
fn correctly_captures_external_links() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(
        &ZOLA_TERA,
        &config,
        "https://vincent.is/about/",
        &permalinks_ctx,
        InsertAnchor::None,
    );
    let content = "
[a link](http://google.com)
[a link](http://google.comy)
Email: [foo@bar.baz](mailto:foo@bar.baz)
Email: <foo@bar.baz>
    ";
    let res = render_content(content, &context).unwrap();
    assert_eq!(
        res.external_links,
        &["http://google.com".to_owned(), "http://google.comy".to_owned()]
    );
}

#[test]
fn can_handle_summaries() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks_ctx, InsertAnchor::None);
    let res = render_content(
        r#"
Hello [My site][world]

<!-- more -->

Bla bla

[world]: https://vincentprouillet.com
"#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
        "<p>Hello <a href=\"https://vincentprouillet.com\">My site</a></p>\n<p id=\"zola-continue-reading\"><a name=\"continue-reading\"></a></p>\n<p>Bla bla</p>\n"
    );
    assert_eq!(
        res.summary_len,
        Some("<p>Hello <a href=\"https://vincentprouillet.com/\">My site</a></p>".len())
    );
}

// https://github.com/Keats/gutenberg/issues/522
#[test]
fn doesnt_try_to_highlight_content_from_shortcode() {
    let permalinks_ctx = HashMap::new();
    let mut tera = Tera::default();
    tera.extend(&ZOLA_TERA).unwrap();

    let shortcode = r#"
<figure>
     {% if width %}
     <img src="/images/{{ src }}" alt="{{ caption }}" width="{{ width }}" />
     {% else %}
     <img src="/images/{{ src }}" alt="{{ caption }}" />
     {% endif %}

     <figcaption>{{ caption }}</figcaption>
</figure>"#;

    let markdown_string = r#"{{ figure(src="spherecluster.png", caption="Some spheres.") }}"#;

    let expected = r#"<figure>
     <img src="/images/spherecluster.png" alt="Some spheres." />
     <figcaption>Some spheres.</figcaption>
</figure>"#;

    tera.add_raw_template(&format!("shortcodes/{}.html", "figure"), shortcode).unwrap();
    let config = Config::default();
    let context = RenderContext::new(&tera, &config, "", &permalinks_ctx, InsertAnchor::None);

    let res = render_content(markdown_string, &context).unwrap();
    assert_eq!(res.body, expected);
}

// TODO: re-enable once it's fixed in Tera
// https://github.com/Keats/tera/issues/373
//#[test]
//fn can_split_lines_shortcode_body() {
//    let permalinks_ctx = HashMap::new();
//    let mut tera = Tera::default();
//    tera.extend(&ZOLA_TERA).unwrap();
//
//    let shortcode = r#"{{ body | split(pat="\n") }}"#;
//
//    let markdown_string = r#"
//{% alert() %}
//multi
//ple
//lines
//{% end %}
//    "#;
//
//    let expected = r#"<p>["multi", "ple", "lines"]</p>"#;
//
//    tera.add_raw_template(&format!("shortcodes/{}.html", "alert"), shortcode).unwrap();
//    let config = Config::default();
//    let context = RenderContext::new(&tera, &config, "", &permalinks_ctx, InsertAnchor::None);
//
//    let res = render_content(markdown_string, &context).unwrap();
//    assert_eq!(res.body, expected);
//}
