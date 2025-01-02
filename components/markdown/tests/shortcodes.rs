use config::Config;

mod common;

#[test]
fn can_render_simple_text_with_shortcodes() {
    let body = common::render(
        r#"
hello {{ out_put_id(id="shortcode-id") }}

{% quote() %}
A quote
{% end %}

{{ out_put_id(id="shortcode-id2") }}

{{ out_put_id(id="shortcode-id3") }}
    "#,
    )
    .unwrap()
    .body;
    insta::assert_snapshot!(body);
}

#[test]
fn can_grab_lang_in_html_shortcodes() {
    let body = common::render(
        r#"
hello in {{ i18n() }}
    "#,
    )
    .unwrap()
    .body;
    insta::assert_snapshot!(body);
}

#[test]
fn can_grab_lang_in_md_shortcodes() {
    let body = common::render(
        r#"
{{ book() }}
    "#,
    )
    .unwrap()
    .body;
    insta::assert_snapshot!(body);
}

#[test]
fn can_render_body_shortcode_and_paragraph_after() {
    let body = common::render(
        r#"
{% quote() %}
This is a quote
{% end %}

Here is another paragraph.
    "#,
    )
    .unwrap()
    .body;
    insta::assert_snapshot!(body);
}

#[test]
fn can_render_two_body_shortcode_and_paragraph_after_with_line_break_between() {
    let body = common::render(
        r#"
{% quote() %}
This is a quote
{% end %}

{% quote() %}
This is a quote
{% end %}

Here is another paragraph.
    "#,
    )
    .unwrap()
    .body;
    insta::assert_snapshot!(body);
}

#[test]
fn doesnt_render_ignored_shortcodes() {
    let body = common::render(
        r#"
{{/* youtube(id="w7Ft2ymGmfc") */}}
    "#,
    )
    .unwrap()
    .body;
    insta::assert_snapshot!(body);
}

// https://github.com/Keats/gutenberg/issues/522
#[test]
fn doesnt_try_to_highlight_content_from_shortcode() {
    let body = common::render(
        r#"
{{ four_spaces() }}
    "#,
    )
    .unwrap()
    .body;
    insta::assert_snapshot!(body);
}

#[test]
fn can_emit_newlines_and_whitespace_with_shortcode() {
    let body = common::render(
        r#"
{% pre() %}
Hello

Zola

!

{% end %}
    "#,
    )
    .unwrap()
    .body;
    insta::assert_snapshot!(body);
}

#[test]
fn can_passthrough_markdown_from_shortcode() {
    let body = common::render(
        r#"
Hello

{% md_passthrough() %}
# Passing through

*to* **the** document
{% end %}
    "#,
    )
    .unwrap()
    .body;
    insta::assert_snapshot!(body);
}

// https://github.com/getzola/zola/issues/1172
#[test]
fn doesnt_escape_html_shortcodes() {
    let body = common::render(
        r#"
{{ image(alt="something") }}
    "#,
    )
    .unwrap()
    .body;
    insta::assert_snapshot!(body);
}

#[test]
fn errors_on_unknown_shortcodes() {
    let body = common::render(
        r#"
{{ unknown() }}
    "#,
    );
    assert!(body.is_err());
}

// https://github.com/getzola/zola/issues/1172
#[test]
fn can_render_commented_out_shortcodes() {
    let body = common::render(
        r#"
<!-- {{ image(alt="something") }} -->
    "#,
    )
    .unwrap()
    .body;
    insta::assert_snapshot!(body);
}

#[test]
fn invocation_count_increments_in_shortcode() {
    let body = common::render(
        r#"
{{ a() }}
{{ b() }}
{{ a() }}
{{ b() }}
    "#,
    )
    .unwrap()
    .body;
    insta::assert_snapshot!(body);
}

// https://github.com/getzola/zola/issues/1689
#[test]
fn html_shortcode_regression() {
    let inputs = vec![
        r#"{{ ex2(page="") }} {{ ex1(page="") }} {{ ex3(page="std") }}"#,
        r#"<p>{{ ex2(page="") }} {{ ex1(page="") }} {{ ex3(page="std") }}</p>"#, // in html
        r#"<p>\n{{ ex2(page='') }}\n</p>"#,                                      // with newlines
        r#"<span>{{ ex2(page='') }}</span>\n**The Book** {{ ex2(page='') }}"#,
        r#"a.{{ ex2(page="") }} b.{{ ex1(page="") }} c.{{ ex3(page="std") }}"#,
    ];

    for input in inputs {
        let body = common::render(input).unwrap().body;
        insta::assert_snapshot!(body);
    }
}

#[test]
fn can_split_shortcode_body_lines() {
    let body = common::render(
        r#"
{% split_lines() %}
multi
ple
lines
{% end %}
    "#,
    )
    .unwrap()
    .body;
    insta::assert_snapshot!(body);
}

#[test]
fn can_render_shortcodes_with_tabs() {
    // This can cause problems mostly because the 4 spaces sometimes used for tabs also are used
    // to indicate code-blocks
    let body = common::render(
        r#"
{{ with_tabs() }} {{ with_tabs() }}
    "#,
    )
    .unwrap()
    .body;
    insta::assert_snapshot!(body);
}

// https://github.com/getzola/zola/issues/1355
#[test]
fn can_render_list_with_shortcode() {
    let body = common::render(
        r#"
* a
* b
    {{ with_tabs() }}
* c
    {{ with_tabs() }}
    "#,
    )
    .unwrap()
    .body;
    insta::assert_snapshot!(body);
}

// https://github.com/getzola/zola/issues/1655
#[test]
fn shortcodes_do_not_generate_paragraphs() {
    let body = common::render(
        r#"
{% web_component() %}
some code;
more code;

other code here;
{% end %}
    "#,
    )
    .unwrap()
    .body;
    insta::assert_snapshot!(body);
}

#[test]
fn can_render_markdown_in_shortcodes() {
    let mut config = Config::default_for_test();
    config.markdown.highlight_code = true;
    let body = common::render_with_config(
        r#"
{% render_md() %}

```
some code;
```

{% end %}
    "#,
        config,
    )
    .unwrap()
    .body;
    insta::assert_snapshot!(body);
}

// https://github.com/getzola/zola/issues/1600
#[test]
fn can_use_shortcodes_in_quotes() {
    let body = common::render(
        r#"
> test quote
> {{ image(alt="a quote") }}
    "#,
    )
    .unwrap()
    .body;
    insta::assert_snapshot!(body);
}

#[test]
fn can_render_with_inline_html() {
    let body = common::render(
        r#"
Here is <span>{{ ex1(page="") }}</span> example.
    "#,
    )
    .unwrap()
    .body;
    insta::assert_snapshot!(body);
}

#[test]
fn can_render_markdown_in_nested_shortcodes_with_bodies() {
    let config = Config::default_for_test();
    let body = common::render_with_config(
        r#"
# Begin level 0  

{% render_md() %}

## Begin level 1

{% render_md() %}

### Begin level 2

{{ a_md() }}, {{ a_md() }}, {{ b_md() }}, {{ b_md() }}

### End level 2

{% end %}

## End level 1

{% end %}

# End level 0
    "#,
        config,
    )
    .unwrap()
    .body;
    insta::assert_snapshot!(body);
}

#[test]
fn can_render_nested_shortcodes_with_bodies_with_nth() {
    let config = Config::default_for_test();
    let body = common::render_with_config(
        r#"
{{ a_md() }}

{{ a_md() }}

{% render_md() %}

{{ a_md() }}

{{ a_md() }}

{% render_md() %}

{{ a_md() }}

{{ a_md() }}

{% end %}

{{ a_md() }}

{{ a_md() }}

{% end %}

{{ a_md() }}

{{ a_md() }}
    "#,
        config,
    )
    .unwrap()
    .body;
    insta::assert_snapshot!(body);
}
