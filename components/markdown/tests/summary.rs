mod common;

fn get_summary(content: &str) -> String {
    let rendered = common::render(content).unwrap();
    assert!(rendered.summary_len.is_some());
    let summary_len = rendered.summary_len.unwrap();
    rendered.body[..summary_len].to_owned()
}

#[test]
fn basic_summary() {
    let body = get_summary(
        r#"
Hello world!

# Introduction

- first
- second

<!-- more -->

And some content after
    "#,
    );
    insta::assert_snapshot!(body);
}

// https://zola.discourse.group/t/zola-12-issue-with-continue-reading/590/7
#[test]
fn summary_with_shortcodes() {
    let body = get_summary(
        r#"
{{ a() }} {{ a() }}
{% render_md() %}
# Hello world
{% end %}
```
some code;
```
<!-- more -->

And some content after
    "#,
    );
    insta::assert_snapshot!(body);
}
