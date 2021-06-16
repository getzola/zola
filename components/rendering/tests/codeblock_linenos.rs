use std::collections::HashMap;

use tera::Tera;

use config::Config;
use front_matter::InsertAnchor;
use rendering::{render_content, RenderContext};

#[test]
fn can_add_line_numbers() {
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
```linenos
foo
bar
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
        "<pre data-linenos style=\"background-color:#2b303b;color:#c0c5ce;\"><code><table><tbody><tr><td>1</td><td><span>foo\n</span><tr><td>2</td><td><span>bar\n</span></tr></tbody></table></code></pre>\n"
    );
}

#[test]
fn can_add_line_numbers_with_linenostart() {
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
```linenos, linenostart=40
foo
bar
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
        "<pre data-linenos style=\"background-color:#2b303b;color:#c0c5ce;\"><code><table><tbody><tr><td>40</td><td><span>foo\n</span><tr><td>41</td><td><span>bar\n</span></tr></tbody></table></code></pre>\n"
    );
}

#[test]
fn can_add_line_numbers_with_highlight() {
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
```linenos, hl_lines=2
foo
bar
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
        "<pre data-linenos style=\"background-color:#2b303b;color:#c0c5ce;\"><code><table><tbody><tr><td>1</td><td><span>foo\n</span><tr><td><mark style=\"background-color:#65737e30;\">2</mark></td><td><mark style=\"background-color:#65737e30;\"><span>bar\n</span></mark></tr></tbody></table></code></pre>\n"
    );
}
