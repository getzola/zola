extern crate gutenberg;
extern crate tera;
extern crate tempdir;

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use tempdir::TempDir;
use tera::Tera;

use gutenberg::{Page, Config};


#[test]
fn test_can_parse_a_valid_page() {
    let content = r#"
+++
title = "Hello"
description = "hey there"
slug = "hello-world"
+++
Hello world"#;
    let res = Page::parse(Path::new("post.md"), content, &Config::default());
    assert!(res.is_ok());
    let mut page = res.unwrap();
    page.render_markdown(&HashMap::default(), &Tera::default(), &Config::default()).unwrap();

    assert_eq!(page.meta.title, "Hello".to_string());
    assert_eq!(page.meta.slug.unwrap(), "hello-world".to_string());
    assert_eq!(page.raw_content, "Hello world".to_string());
    assert_eq!(page.content, "<p>Hello world</p>\n".to_string());
}

#[test]
fn test_can_find_one_parent_directory() {
    let content = r#"
+++
title = "Hello"
description = "hey there"
slug = "hello-world"
+++
Hello world"#;
    let res = Page::parse(Path::new("content/posts/intro.md"), content, &Config::default());
    assert!(res.is_ok());
    let mut page = res.unwrap();
    page.render_markdown(&HashMap::default(), &Tera::default(), &Config::default()).unwrap();
    assert_eq!(page.components, vec!["posts".to_string()]);
}

#[test]
fn test_can_find_multiple_parent_directories() {
    let content = r#"
+++
title = "Hello"
description = "hey there"
slug = "hello-world"
+++
Hello world"#;
    let res = Page::parse(Path::new("content/posts/intro/start.md"), content, &Config::default());
    assert!(res.is_ok());
    let mut page = res.unwrap();
    page.render_markdown(&HashMap::default(), &Tera::default(), &Config::default()).unwrap();
    assert_eq!(page.components, vec!["posts".to_string(), "intro".to_string()]);
}

#[test]
fn test_can_make_url_from_sections_and_slug() {
    let content = r#"
+++
title = "Hello"
description = "hey there"
slug = "hello-world"
+++
Hello world"#;
    let mut conf = Config::default();
    conf.base_url = "http://hello.com/".to_string();
    let res = Page::parse(Path::new("content/posts/intro/start.md"), content, &conf);
    assert!(res.is_ok());
    let mut page = res.unwrap();
    page.render_markdown(&HashMap::default(), &Tera::default(), &Config::default()).unwrap();
    assert_eq!(page.path, "posts/intro/hello-world");
    assert_eq!(page.permalink, "http://hello.com/posts/intro/hello-world");
}

#[test]
fn test_can_make_permalink_with_non_trailing_slash_base_url() {
    let content = r#"
+++
title = "Hello"
description = "hey there"
slug = "hello-world"
+++
Hello world"#;
    let mut conf = Config::default();
    conf.base_url = "http://hello.com".to_string();
    let res = Page::parse(Path::new("content/posts/intro/hello-world.md"), content, &conf);
    assert!(res.is_ok());
    let mut page = res.unwrap();
    page.render_markdown(&HashMap::default(), &Tera::default(), &Config::default()).unwrap();
    assert_eq!(page.path, "posts/intro/hello-world");
    assert_eq!(page.permalink, format!("{}{}", conf.base_url, "/posts/intro/hello-world"));
}

#[test]
fn test_can_make_url_from_slug_only() {
    let content = r#"
+++
title = "Hello"
description = "hey there"
slug = "hello-world"
+++
Hello world"#;
    let res = Page::parse(Path::new("start.md"), content, &Config::default());
    assert!(res.is_ok());
    let mut page = res.unwrap();
    page.render_markdown(&HashMap::default(), &Tera::default(), &Config::default()).unwrap();
    assert_eq!(page.path, "hello-world");
    assert_eq!(page.permalink, format!("{}{}", Config::default().base_url, "hello-world"));
}

#[test]
fn test_errors_on_invalid_front_matter_format() {
    let content = r#"
title = "Hello"
description = "hey there"
slug = "hello-world"
+++
Hello world"#;
    let res = Page::parse(Path::new("start.md"), content, &Config::default());
    assert!(res.is_err());
}

#[test]
fn test_can_make_slug_from_non_slug_filename() {
    let content = r#"
+++
title = "Hello"
description = "hey there"
+++
Hello world"#;
    let res = Page::parse(Path::new("file with space.md"), content, &Config::default());
    assert!(res.is_ok());
    let mut page = res.unwrap();
    page.render_markdown(&HashMap::default(), &Tera::default(), &Config::default()).unwrap();
    assert_eq!(page.slug, "file-with-space");
    assert_eq!(page.permalink, format!("{}{}", Config::default().base_url, "file-with-space"));
}

#[test]
fn test_trim_slug_if_needed() {
    let content = r#"
+++
title = "Hello"
description = "hey there"
+++
Hello world"#;
    let res = Page::parse(Path::new(" file with space.md"), content, &Config::default());
    assert!(res.is_ok());
    let mut page = res.unwrap();
    page.render_markdown(&HashMap::default(), &Tera::default(), &Config::default()).unwrap();
    assert_eq!(page.slug, "file-with-space");
    assert_eq!(page.permalink, format!("{}{}", Config::default().base_url, "file-with-space"));
}

#[test]
fn test_reading_analytics_short() {
    let content = r#"
+++
title = "Hello"
description = "hey there"
+++
Hello world"#;
    let res = Page::parse(Path::new("hello.md"), content, &Config::default());
    assert!(res.is_ok());
    let mut page = res.unwrap();
    page.render_markdown(&HashMap::default(), &Tera::default(), &Config::default()).unwrap();
    let (word_count, reading_time) = page.get_reading_analytics();
    assert_eq!(word_count, 2);
    assert_eq!(reading_time, 0);
}

#[test]
fn test_reading_analytics_long() {
    let mut content = r#"
+++
title = "Hello"
description = "hey there"
+++
Hello world"#.to_string();
    for _ in 0..1000 {
        content.push_str(" Hello world");
    }
    let res = Page::parse(Path::new("hello.md"), &content, &Config::default());
    assert!(res.is_ok());
    let mut page = res.unwrap();
    page.render_markdown(&HashMap::default(), &Tera::default(), &Config::default()).unwrap();
    let (word_count, reading_time) = page.get_reading_analytics();
    assert_eq!(word_count, 2002);
    assert_eq!(reading_time, 10);
}

#[test]
fn test_automatic_summary_is_empty_string() {
    let content = r#"
+++
title = "Hello"
description = "hey there"
+++
Hello world"#.to_string();
    let res = Page::parse(Path::new("hello.md"), &content, &Config::default());
    assert!(res.is_ok());
    let mut page = res.unwrap();
    page.render_markdown(&HashMap::default(), &Tera::default(), &Config::default()).unwrap();
    assert_eq!(page.summary, "");
}

#[test]
fn test_can_specify_summary() {
    let content = r#"
+++
title = "Hello"
description = "hey there"
+++
Hello world
<!-- more -->
"#.to_string();
    let res = Page::parse(Path::new("hello.md"), &content, &Config::default());
    assert!(res.is_ok());
    let mut page = res.unwrap();
    page.render_markdown(&HashMap::default(), &Tera::default(), &Config::default()).unwrap();
    assert_eq!(page.summary, "<p>Hello world</p>\n");
}

#[test]
fn test_can_auto_detect_when_highlighting_needed() {
    let content = r#"
+++
title = "Hello"
description = "hey there"
+++
```
Hey there
```
"#.to_string();
    let mut config = Config::default();
    config.highlight_code = Some(true);
    let res = Page::parse(Path::new("hello.md"), &content, &config);
    assert!(res.is_ok());
    let mut page = res.unwrap();
    page.render_markdown(&HashMap::default(), &Tera::default(), &Config::default()).unwrap();
    assert!(page.content.starts_with("<pre"));
}

#[test]
fn test_file_not_named_index_with_assets() {
    let tmp_dir = TempDir::new("example").expect("create temp dir");
    File::create(tmp_dir.path().join("something.md")).unwrap();
    File::create(tmp_dir.path().join("example.js")).unwrap();
    File::create(tmp_dir.path().join("graph.jpg")).unwrap();
    File::create(tmp_dir.path().join("fail.png")).unwrap();

    let page = Page::from_file(tmp_dir.path().join("something.md"), &Config::default());
    assert!(page.is_err());
}
