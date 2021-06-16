use std::collections::HashMap;

use tera::Tera;

use config::Config;
use front_matter::InsertAnchor;
use rendering::{render_content, RenderContext};

macro_rules! colored_html_line {
    ( $s:expr ) => {{
        let mut result = "<span>".to_string();
        result.push_str($s);
        result.push_str("\n</span>");
        result
    }};
}

macro_rules! colored_html {
    ( $($s:expr),* $(,)* ) => {{
        let mut result = "<pre style=\"background-color:#2b303b;color:#c0c5ce;\"><code>".to_string();
        $(
            result.push_str(colored_html_line!($s).as_str());
        )*
        result.push_str("</code></pre>\n");
        result
    }};
}

#[test]
fn hide_lines_simple() {
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
```hide_lines=2
foo
bar
baz
bat
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(res.body, colored_html!("foo", "baz", "bat"));
}
