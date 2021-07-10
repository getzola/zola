use std::collections::HashMap;

use tera::Tera;

use config::Config;
use front_matter::InsertAnchor;
use rendering::{render_content, RenderContext};

macro_rules! colored_html_line {
    ( @no $s:expr ) => {{
        let mut result = "<span>".to_string();
        result.push_str($s);
        result.push_str("\n</span>");
        result
    }};
    ( @hl $s:expr ) => {{
        let mut result = "<mark style=\"background-color:#65737e30;\">".to_string();
        result.push_str("<span>");
        result.push_str($s);
        result.push_str("\n</span>");
        result.push_str("</mark>");
        result
    }};
}

macro_rules! colored_html {
    ( $(@$kind:tt $s:expr),* $(,)* ) => {{
        let mut result = "<pre style=\"background-color:#2b303b;color:#c0c5ce;\"><code>".to_string();
        $(
            result.push_str(colored_html_line!(@$kind $s).as_str());
        )*
        result.push_str("</code></pre>\n");
        result
    }};
}

#[test]
fn hl_lines_simple() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let mut config = Config::default_for_test();
    config.markdown.highlight_code = true;
    let context = RenderContext::new(
        &tera_ctx,
        &config,
        &config.default_language,
        "",
        &permalinks_ctx,
        InsertAnchor::None,
    );
    let res = render_content(
        r#"
```hl_lines=2
foo
bar
bar
baz
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
        colored_html!(
            @no "foo",
            @hl "bar",
            @no "bar",
            @no "baz",
        )
    );
}

#[test]
fn hl_lines_in_middle() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let mut config = Config::default_for_test();
    config.markdown.highlight_code = true;
    let context = RenderContext::new(
        &tera_ctx,
        &config,
        &config.default_language,
        "",
        &permalinks_ctx,
        InsertAnchor::None,
    );
    let res = render_content(
        r#"
```hl_lines=2-3
foo
bar
bar
baz
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
        colored_html!(
            @no "foo",
            @hl "bar",
            @hl "bar",
            @no "baz",
        )
    );
}

#[test]
fn hl_lines_all() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let mut config = Config::default_for_test();
    config.markdown.highlight_code = true;
    let context = RenderContext::new(
        &tera_ctx,
        &config,
        &config.default_language,
        "",
        &permalinks_ctx,
        InsertAnchor::None,
    );
    let res = render_content(
        r#"
```hl_lines=1-4
foo
bar
bar
baz
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
        colored_html!(
            @hl "foo",
            @hl "bar",
            @hl "bar",
            @hl "baz",
        )
    );
}

#[test]
fn hl_lines_start_from_one() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let mut config = Config::default_for_test();
    config.markdown.highlight_code = true;
    let context = RenderContext::new(
        &tera_ctx,
        &config,
        &config.default_language,
        "",
        &permalinks_ctx,
        InsertAnchor::None,
    );
    let res = render_content(
        r#"
```hl_lines=1-3
foo
bar
bar
baz
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
        colored_html!(
            @hl "foo",
            @hl "bar",
            @hl "bar",
            @no "baz",
        )
    );
}

#[test]
fn hl_lines_start_from_zero() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let mut config = Config::default_for_test();
    config.markdown.highlight_code = true;
    let context = RenderContext::new(
        &tera_ctx,
        &config,
        &config.default_language,
        "",
        &permalinks_ctx,
        InsertAnchor::None,
    );
    let res = render_content(
        r#"
```hl_lines=0-3
foo
bar
bar
baz
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
        colored_html!(
            @hl "foo",
            @hl "bar",
            @hl "bar",
            @no "baz",
        )
    );
}

#[test]
fn hl_lines_end() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let mut config = Config::default_for_test();
    config.markdown.highlight_code = true;
    let context = RenderContext::new(
        &tera_ctx,
        &config,
        &config.default_language,
        "",
        &permalinks_ctx,
        InsertAnchor::None,
    );
    let res = render_content(
        r#"
```hl_lines=3-4
foo
bar
bar
baz
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
        colored_html!(
            @no "foo",
            @no "bar",
            @hl "bar",
            @hl "baz",
        )
    );
}

#[test]
fn hl_lines_end_out_of_bounds() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let mut config = Config::default_for_test();
    config.markdown.highlight_code = true;
    let context = RenderContext::new(
        &tera_ctx,
        &config,
        &config.default_language,
        "",
        &permalinks_ctx,
        InsertAnchor::None,
    );
    let res = render_content(
        r#"
```hl_lines=3-4294967295
foo
bar
bar
baz
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
        colored_html!(
            @no "foo",
            @no "bar",
            @hl "bar",
            @hl "baz",
        )
    );
}

#[test]
fn hl_lines_overlap() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let mut config = Config::default_for_test();
    config.markdown.highlight_code = true;
    let context = RenderContext::new(
        &tera_ctx,
        &config,
        &config.default_language,
        "",
        &permalinks_ctx,
        InsertAnchor::None,
    );
    let res = render_content(
        r#"
```hl_lines=2-3 1-2
foo
bar
bar
baz
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
        colored_html!(
            @hl "foo",
            @hl "bar",
            @hl "bar",
            @no "baz",
        )
    );
}
#[test]
fn hl_lines_multiple() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let mut config = Config::default_for_test();
    config.markdown.highlight_code = true;
    let context = RenderContext::new(
        &tera_ctx,
        &config,
        &config.default_language,
        "",
        &permalinks_ctx,
        InsertAnchor::None,
    );
    let res = render_content(
        r#"
```hl_lines=2-3,hl_lines=1-2
foo
bar
bar
baz
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
        colored_html!(
            @hl "foo",
            @hl "bar",
            @hl "bar",
            @no "baz",
        )
    );
}

#[test]
fn hl_lines_extra_spaces() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let mut config = Config::default_for_test();
    config.markdown.highlight_code = true;
    let context = RenderContext::new(
        &tera_ctx,
        &config,
        &config.default_language,
        "",
        &permalinks_ctx,
        InsertAnchor::None,
    );
    let res = render_content(
        r#"
```     hl_lines     =       2 - 3      1    -       2
foo
bar
bar
baz
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
        colored_html!(
            @hl "foo",
            @hl "bar",
            @hl "bar",
            @no "baz",
        )
    );
}

#[test]
fn hl_lines_int_and_range() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let mut config = Config::default_for_test();
    config.markdown.highlight_code = true;
    let context = RenderContext::new(
        &tera_ctx,
        &config,
        &config.default_language,
        "",
        &permalinks_ctx,
        InsertAnchor::None,
    );
    let res = render_content(
        r#"
```hl_lines=1 3-4
foo
bar
bar
baz
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
        colored_html!(
            @hl "foo",
            @no "bar",
            @hl "bar",
            @hl "baz",
        )
    );
}

#[test]
fn hl_lines_single_line_range() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let mut config = Config::default_for_test();
    config.markdown.highlight_code = true;
    let context = RenderContext::new(
        &tera_ctx,
        &config,
        &config.default_language,
        "",
        &permalinks_ctx,
        InsertAnchor::None,
    );
    let res = render_content(
        r#"
```hl_lines=2-2
foo
bar
bar
baz
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
        colored_html!(
            @no "foo",
            @hl "bar",
            @no "bar",
            @no "baz",
        )
    );
}

#[test]
fn hl_lines_reverse_range() {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let mut config = Config::default_for_test();
    config.markdown.highlight_code = true;
    let context = RenderContext::new(
        &tera_ctx,
        &config,
        &config.default_language,
        "",
        &permalinks_ctx,
        InsertAnchor::None,
    );
    let res = render_content(
        r#"
```hl_lines=3-2
foo
bar
bar
baz
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
        colored_html!(
            @no "foo",
            @hl "bar",
            @hl "bar",
            @no "baz",
        )
    );
}
