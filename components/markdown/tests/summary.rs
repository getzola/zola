mod common;

fn get_summary(content: &str) -> String {
    get_rendered(content).summary.expect("had no summary")
}

fn get_rendered(content: &str) -> markdown::Rendered {
    common::render(content).expect("couldn't render")
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

#[test]
fn truncated_summary() {
    let body = get_summary(
        r#"
Things to do:
* Program <!-- more --> something
* Eat
* Sleep
    "#,
    );
    insta::assert_snapshot!(body);
}

#[test]
fn footnotes_summary() {
    let body = get_summary(
        r#"
Hello world[^1].

<!-- more -->

Good bye.

[^1]: "World" is a placeholder.
    "#,
    );
    insta::assert_snapshot!(body);
}
