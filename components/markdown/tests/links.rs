mod common;

#[test]
fn can_detect_links() {
    // no links
    let rendered = common::render("Hello World!").unwrap();
    assert_eq!(rendered.internal_links.len(), 0);
    assert_eq!(rendered.external_links.len(), 0);

    // external
    let rendered = common::render("[abc](https://google.com/)").unwrap();
    assert_eq!(rendered.internal_links.len(), 0);
    assert_eq!(rendered.external_links.len(), 1);
    assert_eq!(rendered.external_links[0], "https://google.com/");

    // internal
    let rendered = common::render("[abc](@/pages/about.md)").unwrap();
    assert_eq!(rendered.internal_links, vec![("pages/about.md".to_owned(), None)]);
    assert_eq!(rendered.external_links.len(), 0);

    // internal with anchors
    let rendered = common::render("[abc](@/pages/about.md#hello)").unwrap();
    assert_eq!(rendered.internal_links[0], ("pages/about.md".to_owned(), Some("hello".to_owned())));
    assert_eq!(rendered.external_links.len(), 0);

    // internal link referring to self
    let rendered = common::render("[abc](#hello)").unwrap();
    assert_eq!(rendered.internal_links.len(), 1);
    assert_eq!(rendered.internal_links[0], ("my_page.md".to_owned(), Some("hello".to_owned())));
    assert_eq!(rendered.external_links.len(), 0);

    // Mixed with various protocols
    let rendered = common::render(
        "
[a link](http://google.com)
[a link](http://google.fr)
Email: [foo@bar.baz](mailto:foo@bar.baz)
Email: <foo@bar.baz>",
    )
    .unwrap();
    assert_eq!(rendered.internal_links.len(), 0);
    assert_eq!(
        rendered.external_links,
        &["http://google.com".to_owned(), "http://google.fr".to_owned()]
    );

    // Not pointing to anything known so that's an error
    let res = common::render("[abc](@/def/123.md)");
    assert!(res.is_err());

    // Empty link is an error as well
    let res = common::render("[abc]()");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "There is a link that is missing a URL");
}
