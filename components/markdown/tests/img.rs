mod common;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use ahash::AHashMap;
use config::Config;
use markdown::MarkdownContext;
use templates::ZOLA_TERA;
use utils::types::InsertAnchor;

const SVG_CONTENT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="320" height="240" viewBox="0 0 320 240"></svg>"#;

#[test]
fn can_transform_image() {
    let cases = [
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
    let cases = [
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

fn create_test_context<'a>(
    tera: &'a tera::Tera,
    config: &'a Config,
    permalinks: &'a HashMap<String, String>,
    colocated_assets: &'a AHashMap<String, (String, String)>,
    current_dir: Option<&'a Path>,
    base_path: Option<&'a Path>,
) -> MarkdownContext<'a> {
    MarkdownContext {
        tera,
        config,
        permalinks,
        colocated_assets,
        lang: &config.default_language,
        current_permalink: "https://www.getzola.org/test/",
        current_path: "my_page.md",
        insert_anchor: InsertAnchor::None,
        current_dir,
        base_path,
    }
}

#[test]
fn can_add_dimensions_for_relative_bundle_image() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("graphic.svg"), SVG_CONTENT).unwrap();

    let mut tera = ZOLA_TERA.clone();
    let _ = tera.set_fallback_prefixes(vec!["__zola_builtins/".to_string()]);
    let permalinks = HashMap::new();
    let config = Config::default_for_test();
    let colocated_assets = AHashMap::new();

    let context =
        create_test_context(&tera, &config, &permalinks, &colocated_assets, Some(tmp.path()), None);

    let rendered = markdown::render_content("![sample](graphic.svg)", &context).unwrap();
    assert_eq!(
        rendered.body.trim(),
        r#"<p><img src="https://www.getzola.org/test/graphic.svg" alt="sample" width="320" height="240" /></p>"#
    );
}

#[test]
fn can_add_dimensions_with_percent_encoding_and_query_fragment() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("my graphic.svg"), SVG_CONTENT).unwrap();

    let mut tera = ZOLA_TERA.clone();
    let _ = tera.set_fallback_prefixes(vec!["__zola_builtins/".to_string()]);
    let permalinks = HashMap::new();
    let config = Config::default_for_test();
    let colocated_assets = AHashMap::new();

    let context =
        create_test_context(&tera, &config, &permalinks, &colocated_assets, Some(tmp.path()), None);

    let rendered =
        markdown::render_content("![sample](my%20graphic.svg?v=1#icon)", &context).unwrap();
    assert_eq!(
        rendered.body.trim(),
        r#"<p><img src="https://www.getzola.org/test/my%20graphic.svg?v=1#icon" alt="sample" width="320" height="240" /></p>"#
    );
}

#[test]
fn can_add_dimensions_for_colocated_content_image() {
    let tmp = tempfile::tempdir().unwrap();
    let blog_dir = tmp.path().join("content").join("blog");
    fs::create_dir_all(&blog_dir).unwrap();
    fs::write(blog_dir.join("graphic.svg"), SVG_CONTENT).unwrap();

    let mut tera = ZOLA_TERA.clone();
    let _ = tera.set_fallback_prefixes(vec!["__zola_builtins/".to_string()]);
    let mut permalinks = HashMap::new();
    permalinks.insert("blog/index.md".to_string(), "https://www.getzola.org/blog/".to_string());
    let config = Config::default_for_test();
    let mut colocated_assets = AHashMap::new();
    colocated_assets.insert(
        "blog/graphic.svg".to_string(),
        ("blog/index.md".to_string(), "graphic.svg".to_string()),
    );

    let context =
        create_test_context(&tera, &config, &permalinks, &colocated_assets, None, Some(tmp.path()));

    let rendered = markdown::render_content("![sample](@/blog/graphic.svg)", &context).unwrap();
    assert_eq!(
        rendered.body.trim(),
        r#"<p><img src="https://www.getzola.org/blog/graphic.svg" alt="sample" width="320" height="240" /></p>"#
    );
}

#[test]
fn can_add_dimensions_for_colocated_content_image_with_query_and_fragment() {
    let tmp = tempfile::tempdir().unwrap();
    let blog_dir = tmp.path().join("content").join("blog");
    fs::create_dir_all(&blog_dir).unwrap();
    fs::write(blog_dir.join("my graphic.svg"), SVG_CONTENT).unwrap();

    let mut tera = ZOLA_TERA.clone();
    let _ = tera.set_fallback_prefixes(vec!["__zola_builtins/".to_string()]);
    let mut permalinks = HashMap::new();
    permalinks.insert("blog/index.md".to_string(), "https://www.getzola.org/blog/".to_string());
    let config = Config::default_for_test();
    let mut colocated_assets = AHashMap::new();
    colocated_assets.insert(
        "blog/my graphic.svg".to_string(),
        ("blog/index.md".to_string(), "my graphic.svg".to_string()),
    );

    let context =
        create_test_context(&tera, &config, &permalinks, &colocated_assets, None, Some(tmp.path()));

    let rendered =
        markdown::render_content("![sample](@/blog/my%20graphic.svg?v=1#hash)", &context).unwrap();
    assert_eq!(
        rendered.body.trim(),
        r#"<p><img src="https://www.getzola.org/blog/my%20graphic.svg?v=1#hash" alt="sample" width="320" height="240" /></p>"#
    );
}

#[test]
fn can_add_dimensions_for_static_image() {
    let tmp = tempfile::tempdir().unwrap();
    let static_dir = tmp.path().join("static");
    fs::create_dir_all(&static_dir).unwrap();
    fs::write(static_dir.join("static-img.svg"), SVG_CONTENT).unwrap();

    let mut tera = ZOLA_TERA.clone();
    let _ = tera.set_fallback_prefixes(vec!["__zola_builtins/".to_string()]);
    let permalinks = HashMap::new();
    let config = Config::default_for_test();
    let colocated_assets = AHashMap::new();

    let context =
        create_test_context(&tera, &config, &permalinks, &colocated_assets, None, Some(tmp.path()));

    let rendered = markdown::render_content("![sample](/static-img.svg)", &context).unwrap();
    assert_eq!(
        rendered.body.trim(),
        r#"<p><img src="/static-img.svg" alt="sample" width="320" height="240" /></p>"#
    );
}

#[test]
fn can_add_dimensions_for_theme_image() {
    let tmp = tempfile::tempdir().unwrap();
    let theme_static = tmp.path().join("themes").join("my-theme").join("static");
    fs::create_dir_all(&theme_static).unwrap();
    fs::write(theme_static.join("theme-img.svg"), SVG_CONTENT).unwrap();

    let mut tera = ZOLA_TERA.clone();
    let _ = tera.set_fallback_prefixes(vec!["__zola_builtins/".to_string()]);
    let permalinks = HashMap::new();
    let mut config = Config::default_for_test();
    config.theme = Some("my-theme".to_string());
    let colocated_assets = AHashMap::new();

    let context =
        create_test_context(&tera, &config, &permalinks, &colocated_assets, None, Some(tmp.path()));

    let rendered = markdown::render_content("![sample](/theme-img.svg)", &context).unwrap();
    assert_eq!(
        rendered.body.trim(),
        r#"<p><img src="/theme-img.svg" alt="sample" width="320" height="240" /></p>"#
    );
}

#[test]
fn relative_image_does_not_fallback_to_static() {
    let tmp = tempfile::tempdir().unwrap();
    let static_dir = tmp.path().join("static");
    fs::create_dir_all(&static_dir).unwrap();
    fs::write(static_dir.join("static-img.svg"), SVG_CONTENT).unwrap();

    let bundle_dir = tempfile::tempdir().unwrap();

    let mut tera = ZOLA_TERA.clone();
    let _ = tera.set_fallback_prefixes(vec!["__zola_builtins/".to_string()]);
    let permalinks = HashMap::new();
    let config = Config::default_for_test();
    let colocated_assets = AHashMap::new();

    let context = create_test_context(
        &tera,
        &config,
        &permalinks,
        &colocated_assets,
        Some(bundle_dir.path()),
        Some(tmp.path()),
    );

    let rendered = markdown::render_content("![sample](static-img.svg)", &context).unwrap();
    assert_eq!(
        rendered.body.trim(),
        r#"<p><img src="https://www.getzola.org/test/static-img.svg" alt="sample" /></p>"#
    );
}

#[test]
fn missing_image_and_external_url_have_no_dimensions() {
    let tmp = tempfile::tempdir().unwrap();

    let mut tera = ZOLA_TERA.clone();
    let _ = tera.set_fallback_prefixes(vec!["__zola_builtins/".to_string()]);
    let permalinks = HashMap::new();
    let config = Config::default_for_test();
    let colocated_assets = AHashMap::new();

    let context = create_test_context(
        &tera,
        &config,
        &permalinks,
        &colocated_assets,
        Some(tmp.path()),
        Some(tmp.path()),
    );

    let rendered = markdown::render_content("![sample](missing.svg)", &context).unwrap();
    assert_eq!(
        rendered.body.trim(),
        r#"<p><img src="https://www.getzola.org/test/missing.svg" alt="sample" /></p>"#
    );

    let rendered =
        markdown::render_content("![sample](https://example.com/pic.png)", &context).unwrap();
    assert_eq!(
        rendered.body.trim(),
        r#"<p><img src="https://example.com/pic.png" alt="sample" /></p>"#
    );
}
