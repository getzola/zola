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

#[test]
fn footnotes_in_summary_no_duplication() {
    // Test that content with footnotes before <!-- more --> is not duplicated
    let content = r#"
When Ken Thompson won the Turing Award jointly with Dennis Ritchie for their
work in UNIX, he was expected like other Turing winners to write a paper that
would be published in the ACM Computer Journal. What he ended up submitting was
a paper about "the cutest program [he] ever wrote"-- a sneaky undetectable
self-reproducing "Trojan horse" virus in the C compiler that would allow him to
log into affected machines as any user.

Thompson didn't want to write about the usual things that Turing
award winners write about— in fact, according to him,[^1] he didn't want to write a
paper at all. However, when he did finally write a paper (after putting it off
for a year past the original deadline)

<!-- more -->

The rest of the content.

[^1]: YouTube video reference.
    "#;

    let rendered = get_rendered(content);

    // Summary should not have footnote references
    let summary = rendered.summary.expect("should have summary");
    assert!(!summary.contains("footnote-reference"), "summary should not have footnote references");

    // Body should start with continue-reading marker, not duplicate the summary content
    let body = rendered.body.trim();
    assert!(body.starts_with("<span id=\"continue-reading\"></span>"), "body should start with continue marker");

    // The summary text should not appear in the body after the continue-reading marker
    assert!(!body[40..].contains("When Ken Thompson won the Turing Award"),
        "body should not duplicate summary content");
}
