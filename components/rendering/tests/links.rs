use std::collections::HashMap;

use errors::Result;
use rendering::Rendered;

mod common;

fn render_content(content: &str, permalinks: HashMap<String, String>) -> Result<Rendered> {
    let config = config::Config::default_for_test();
    let tera = tera::Tera::default();
    let mut context = rendering::RenderContext::new(
        &tera,
        &config,
        &config.default_language,
        "http://mypage.com",
        &permalinks,
        front_matter::InsertAnchor::None,
    );
    context.set_current_page_path("mine.md");

    rendering::render_content(content, &context)
}

#[test]
fn can_detect_links() {
    // no links
    let rendered = render_content("Hello World!", HashMap::new()).unwrap();
    assert_eq!(rendered.internal_links.len(), 0);
    assert_eq!(rendered.external_links.len(), 0);

    // external
    let rendered = render_content("[abc](https://google.com/)", HashMap::new()).unwrap();
    assert_eq!(rendered.internal_links.len(), 0);
    assert_eq!(rendered.external_links.len(), 1);
    assert_eq!(rendered.external_links[0], "https://google.com/");

    // internal
    let mut permalinks = HashMap::new();
    permalinks.insert("def/123.md".to_owned(), "https://xyz.com/def/123".to_owned());
    let rendered = render_content("[abc](@/def/123.md)", permalinks).unwrap();
    assert_eq!(rendered.internal_links.len(), 1);
    assert_eq!(rendered.internal_links[0], ("def/123.md".to_owned(), None));
    assert_eq!(rendered.external_links.len(), 0);

    // internal with anchors
    let mut permalinks = HashMap::new();
    permalinks.insert("def/123.md".to_owned(), "https://xyz.com/def/123".to_owned());
    let rendered = render_content("[abc](@/def/123.md#hello)", permalinks).unwrap();
    assert_eq!(rendered.internal_links.len(), 1);
    assert_eq!(rendered.internal_links[0], ("def/123.md".to_owned(), Some("hello".to_owned())));
    assert_eq!(rendered.external_links.len(), 0);

    // internal link referring to self
    let rendered = render_content("[abc](#hello)", HashMap::new()).unwrap();
    assert_eq!(rendered.internal_links.len(), 1);
    assert_eq!(rendered.internal_links[0], ("mine.md".to_owned(), Some("hello".to_owned())));
    assert_eq!(rendered.external_links.len(), 0);

    // Not pointing to anything so that's an error
    let res = render_content("[abc](@/def/123.md)", HashMap::new());
    assert!(res.is_err());
}
