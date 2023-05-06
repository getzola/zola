mod common;
use config::Config;

#[test]
fn can_transform_image() {
    let cases = vec![
        "![haha](https://example.com/abc.jpg)",
        "![](https://example.com/abc.jpg)",
        "![ha\"h>a](https://example.com/abc.jpg)",
        "![__ha__*ha*](https://example.com/abc.jpg)",
        "![ha[ha](https://example.com)](https://example.com/abc.jpg)",
    ];

    let body = common::render(&cases.join("\n")).unwrap().body;
    insta::assert_snapshot!(body);
}

#[test]
fn can_add_lazy_loading_and_async_decoding() {
    let cases = vec![
        "![haha](https://example.com/abc.jpg)",
        "![](https://example.com/abc.jpg)",
        "![ha\"h>a](https://example.com/abc.jpg)",
        "![__ha__*ha*](https://example.com/abc.jpg)",
        "![ha[ha](https://example.com)](https://example.com/abc.jpg)",
    ];

    let mut config = Config::default_for_test();
    config.markdown.lazy_async_image = true;

    let body = common::render_with_config(&cases.join("\n"), config).unwrap().body;
    insta::assert_snapshot!(body);
}
