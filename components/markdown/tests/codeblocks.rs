use config::Config;

mod common;

enum HighlightMode {
    None,
    Inlined,
    Classed,
}

fn render_codeblock(content: &str, highlight_mode: HighlightMode) -> String {
    let mut config = Config::default_for_test();
    match highlight_mode {
        HighlightMode::None => {}
        HighlightMode::Inlined => {
            config.markdown.highlight_code = true;
        }
        HighlightMode::Classed => {
            config.markdown.highlight_code = true;
            config.markdown.highlight_theme = "css".to_owned();
        }
    }
    common::render_with_config(content, config).unwrap().body
}

#[test]
fn does_nothing_with_highlighting_disabled() {
    let body = render_codeblock(
        r#"
```
foo
bar
```
    "#,
        HighlightMode::None,
    );
    insta::assert_snapshot!(body);
}

#[test]
fn can_hide_lines() {
    let body = render_codeblock(
        r#"
```hide_lines=2
foo
bar
baz
bat
```
    "#,
        HighlightMode::Inlined,
    );
    insta::assert_snapshot!(body);
}

#[test]
fn can_highlight_single_line() {
    let body = render_codeblock(
        r#"
```hl_lines=2
foo
bar
bar
baz
```
    "#,
        HighlightMode::Inlined,
    );
    insta::assert_snapshot!(body);
}

#[test]
fn can_highlight_line_range() {
    let body = render_codeblock(
        r#"
```hl_lines=2-3
foo
bar
bar
baz
```
    "#,
        HighlightMode::Inlined,
    );
    insta::assert_snapshot!(body);
}

#[test]
fn can_highlight_all_lines() {
    let body = render_codeblock(
        r#"
```hl_lines=1-4
foo
bar
bar
baz
```
    "#,
        HighlightMode::Inlined,
    );
    insta::assert_snapshot!(body);
}

#[test]
fn can_highlight_zero_start_same_as_one() {
    let body = render_codeblock(
        r#"
```hl_lines=0-3
foo
bar
bar
baz
```
    "#,
        HighlightMode::Inlined,
    );
    let body2 = render_codeblock(
        r#"
```hl_lines=1-3
foo
bar
bar
baz
```
    "#,
        HighlightMode::Inlined,
    );
    assert_eq!(body, body2);
}

#[test]
fn can_highlight_at_end() {
    let body = render_codeblock(
        r#"
```hl_lines=3-4
foo
bar
bar
baz
```
    "#,
        HighlightMode::Inlined,
    );
    insta::assert_snapshot!(body);
}

#[test]
fn can_highlight_out_of_bounds() {
    let body = render_codeblock(
        r#"
```hl_lines=3-4567898765
foo
bar
bar
baz
```
    "#,
        HighlightMode::Inlined,
    );
    insta::assert_snapshot!(body);
}

#[test]
fn can_highlight_ranges_overlap() {
    let body = render_codeblock(
        r#"
```hl_lines=2-3 1-2
foo
bar
bar
baz
```
    "#,
        HighlightMode::Inlined,
    );
    insta::assert_snapshot!(body);
}

#[test]
fn can_highlight_weird_fence_tokens() {
    let body = render_codeblock(
        r#"
```hl_lines=2-3,   hl_lines      = 1 - 2
foo
bar
bar
baz
```
    "#,
        HighlightMode::Inlined,
    );
    insta::assert_snapshot!(body);
}

#[test]
fn can_highlight_mix_line_ranges() {
    let body = render_codeblock(
        r#"
```hl_lines=1 3-4
foo
bar
bar
baz
```
    "#,
        HighlightMode::Inlined,
    );
    insta::assert_snapshot!(body);
}

#[test]
fn can_highlight_single_line_range() {
    let body = render_codeblock(
        r#"
```hl_lines=2-2
foo
bar
bar
baz
```
    "#,
        HighlightMode::Inlined,
    );
    insta::assert_snapshot!(body);
}

#[test]
fn can_highlight_reversed_range() {
    let body = render_codeblock(
        r#"
```hl_lines=3-2
foo
bar
bar
baz
```
    "#,
        HighlightMode::Inlined,
    );
    insta::assert_snapshot!(body);
}

#[test]
fn can_highlight_with_classes() {
    let body = render_codeblock(
        r#"
```html,hl_lines=3-4
<link
    rel="stylesheet"
    type="text/css"
    href="main.css"
/>
```
    "#,
        HighlightMode::Classed,
    );
    insta::assert_snapshot!(body);
}

#[test]
fn can_add_line_numbers() {
    let body = render_codeblock(
        r#"
```linenos
foo
bar
```
    "#,
        HighlightMode::Inlined,
    );
    insta::assert_snapshot!(body);
}

#[test]
fn can_add_line_numbers_windows_eol() {
    let body = render_codeblock("```linenos\r\nfoo\r\nbar\r\n```\r\n", HighlightMode::Inlined);
    insta::assert_snapshot!(body);
}

#[test]
fn can_add_line_numbers_with_lineno_start() {
    let body = render_codeblock(
        r#"
```linenos, linenostart=40
foo
bar
```
    "#,
        HighlightMode::Inlined,
    );
    insta::assert_snapshot!(body);
}

#[test]
fn can_add_line_numbers_with_highlight() {
    let body = render_codeblock(
        r#"
```linenos, hl_lines=2
foo
bar
```
    "#,
        HighlightMode::Inlined,
    );
    insta::assert_snapshot!(body);
}

#[test]
fn can_add_line_numbers_with_classes() {
    let body = render_codeblock(
        r#"
```html,linenos
<link
    rel="stylesheet"
    type="text/css"
    href="main.css"
/>
```
    "#,
        HighlightMode::Classed,
    );
    insta::assert_snapshot!(body);
}

#[test]
fn can_render_shortcode_in_codeblock() {
    let body = render_codeblock(
        r#"
```html,linenos
<div id="custom-attr">
{{ out_put_id(id="dQw4w9WgXcQ") }}
</div>
```
    "#,
        HighlightMode::Inlined,
    );
    insta::assert_snapshot!(body);
}

#[test]
fn can_render_multiple_shortcodes_in_codeblock() {
    let body = render_codeblock(
        r#"
```linenos
text1
{{ out_put_id(id="first") }}
text2
{{ out_put_id(id="second") }}
text3
```
    "#,
        HighlightMode::Inlined,
    );
    insta::assert_snapshot!(body);
}

#[test]
fn can_render_completely_mixed_codeblock() {
    let body = render_codeblock(
        r#"
```html,linenos
<a href="javascript:void(0);">{{/* before(texts="1") */}}</a>
Normally people would not write something & like <> this：
<div id="custom-attr">
An inline {{ out_put_id(id="dQw4w9WgXcQ") }} shortcode
</div>
Plain text in-between
{%/* quote(author="Vincent") */%}
A quote
{%/* end */%}
{# A Tera comment, you should see it #}
<!-- end text goes here -->
```
    "#,
        HighlightMode::Inlined,
    );
    insta::assert_snapshot!(body);
}

#[test]
fn can_highlight_unknown_lang() {
    let body = render_codeblock(
        r#"
```rustscript
foo
bar
```
    "#,
        HighlightMode::Inlined,
    );
    insta::assert_snapshot!(body);
}
