mod common;
use config::Config;

#[test]
fn can_transform_image() {
    // normal image
    let rendered = common::render("![haha](https://example.com/abc.jpg)").unwrap().body;
    assert_eq!(rendered, "<p><img src=\"https://example.com/abc.jpg\" alt=\"haha\" /></p>\n");

    // alt text with style
    let rendered = common::render("![__ha__*ha*](https://example.com/abc.jpg)").unwrap().body;
    assert_eq!(rendered, "<p><img src=\"https://example.com/abc.jpg\" alt=\"haha\" /></p>\n");

    // alt text with link
    let rendered =
        common::render("![ha[ha](https://example.com)](https://example.com/abc.jpg)").unwrap().body;
    assert_eq!(rendered, "<p><img src=\"https://example.com/abc.jpg\" alt=\"haha\" /></p>\n");
}

#[test]
fn can_add_lazy_loading_and_async_decoding() {
    // normal alt text
    let mut config = Config::default_for_test();
    config.markdown.lazy_async_image = true;
    let rendered =
        common::render_with_config("![haha](https://example.com/abc.jpg)", config.clone())
            .unwrap()
            .body;
    assert_eq!(rendered, "<p><img src=\"https://example.com/abc.jpg\" alt=\"haha\" loading=\"lazy\" decoding=\"async\" /></p>\n");

    // Below is acceptable, but not recommended by CommonMark

    // alt text with style
    let rendered =
        common::render_with_config("![__ha__*ha*](https://example.com/abc.jpg)", config.clone())
            .unwrap()
            .body;
    assert_eq!(rendered, "<p><img src=\"https://example.com/abc.jpg\" alt=\"<strong>ha</strong><em>ha</em>\" loading=\"lazy\" decoding=\"async\" /></p>\n");

    // alt text with link
    let rendered = common::render_with_config(
        "![ha[ha](https://example.com)](https://example.com/abc.jpg)",
        config.clone(),
    )
    .unwrap()
    .body;
    assert_eq!(rendered, "<p><img src=\"https://example.com/abc.jpg\" alt=\"ha<a href=\"https://example.com\">ha</a>\" loading=\"lazy\" decoding=\"async\" /></p>\n");
}
