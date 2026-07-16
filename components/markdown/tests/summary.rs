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
