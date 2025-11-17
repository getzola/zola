mod common;

#[test]
fn can_make_simple_toc() {
    let res = common::render(
        r#"
# Heading 1

## Heading 2

## Another Heading 2

### Last one
    "#,
    )
    .unwrap();

    let toc = res.toc;
    assert_eq!(toc.len(), 1);
    assert_eq!(toc[0].children.len(), 2);
    assert_eq!(toc[0].children[1].children.len(), 1);
}

#[test]
fn can_ignore_tags_in_toc() {
    let res = common::render(
        r#"
## heading with `code`

## [anchor](https://duckduckgo.com/) in heading

## **bold** and *italics*
    "#,
    )
    .unwrap();

    let toc = res.toc;
    assert_eq!(toc.len(), 3);

    assert_eq!(toc[0].id, "heading-with-code");
    assert_eq!(toc[0].title, "heading with code");

    assert_eq!(toc[1].id, "anchor-in-heading");
    assert_eq!(toc[1].title, "anchor in heading");

    assert_eq!(toc[2].id, "bold-and-italics");
    assert_eq!(toc[2].title, "bold and italics");
}

#[test]
fn can_provide_raw_content() {
    let res = common::render(
        r#"
## heading with `code`

## **bold** and *italics*

## exploring the `<details>` tag
    "#,
    )
    .unwrap();

    let toc = res.toc;
    assert_eq!(toc.len(), 3);

    assert_eq!(toc[0].id, "heading-with-code");
    assert_eq!(toc[0].title_raw, "heading with <code>code</code>");

    assert_eq!(toc[1].id, "bold-and-italics");
    assert_eq!(toc[1].title_raw, "<strong>bold</strong> and <em>italics</em>");

    assert_eq!(toc[2].id, "exploring-the-details-tag");
    assert_eq!(toc[2].title_raw, "exploring the <code>&lt;details&gt;</code> tag");
}

#[test]
fn can_make_toc_all_levels() {
    let res = common::render(
        r#"
# A

## B1

## B2

### C

#### D

##### E

###### F
"#,
    )
    .unwrap();

    let toc = res.toc;
    assert_eq!(toc.len(), 1);
    assert_eq!(toc[0].children.len(), 2);
    assert_eq!(toc[0].children[1].children.len(), 1);
    assert_eq!(toc[0].children[1].children[0].children.len(), 1);
    assert_eq!(toc[0].children[1].children[0].children[0].children.len(), 1);
}
