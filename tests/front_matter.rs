extern crate gutenberg;
extern crate tera;

use std::path::Path;

use gutenberg::{FrontMatter, split_content};
use tera::to_value;


#[test]
fn test_can_parse_a_valid_front_matter() {
    let content = r#"
title = "Hello"
description = "hey there""#;
    let res = FrontMatter::parse(content);
    println!("{:?}", res);
    assert!(res.is_ok());
    let res = res.unwrap();
    assert_eq!(res.title.unwrap(), "Hello".to_string());
    assert_eq!(res.description.unwrap(), "hey there".to_string());
}

#[test]
fn test_can_parse_tags() {
    let content = r#"
title = "Hello"
description = "hey there"
slug = "hello-world"
tags = ["rust", "html"]"#;
    let res = FrontMatter::parse(content);
    assert!(res.is_ok());
    let res = res.unwrap();

    assert_eq!(res.title.unwrap(), "Hello".to_string());
    assert_eq!(res.slug.unwrap(), "hello-world".to_string());
    assert_eq!(res.tags.unwrap(), ["rust".to_string(), "html".to_string()]);
}

#[test]
fn test_can_parse_extra_attributes_in_frontmatter() {
    let content = r#"
title = "Hello"
description = "hey there"
slug = "hello-world"

[extra]
language = "en"
authors = ["Bob", "Alice"]"#;
    let res = FrontMatter::parse(content);
    assert!(res.is_ok());
    let res = res.unwrap();

    assert_eq!(res.title.unwrap(), "Hello".to_string());
    assert_eq!(res.slug.unwrap(), "hello-world".to_string());
    let extra = res.extra.unwrap();
    assert_eq!(extra["language"], to_value("en").unwrap());
    assert_eq!(
        extra["authors"],
        to_value(["Bob".to_string(), "Alice".to_string()]).unwrap()
    );
}

#[test]
fn test_is_ok_with_url_instead_of_slug() {
    let content = r#"
title = "Hello"
description = "hey there"
url = "hello-world""#;
    let res = FrontMatter::parse(content);
    assert!(res.is_ok());
    let res = res.unwrap();
    assert!(res.slug.is_none());
    assert_eq!(res.url.unwrap(), "hello-world".to_string());
}

#[test]
fn test_errors_with_empty_front_matter() {
    let content = r#"  "#;
    let res = FrontMatter::parse(content);
    assert!(res.is_err());
}

#[test]
fn test_errors_with_invalid_front_matter() {
    let content = r#"title = 1\n"#;
    let res = FrontMatter::parse(content);
    assert!(res.is_err());
}

#[test]
fn test_errors_on_non_string_tag() {
    let content = r#"
title = "Hello"
description = "hey there"
slug = "hello-world"
tags = ["rust", 1]"#;
    let res = FrontMatter::parse(content);
    assert!(res.is_err());
}

#[test]
fn test_errors_on_present_but_empty_slug() {
    let content = r#"
title = "Hello"
description = "hey there"
slug = """#;
    let res = FrontMatter::parse(content);
    assert!(res.is_err());
}

#[test]
fn test_errors_on_present_but_empty_url() {
    let content = r#"
title = "Hello"
description = "hey there"
url = """#;
    let res = FrontMatter::parse(content);
    assert!(res.is_err());
}

#[test]
fn test_parse_date_yyyy_mm_dd() {
    let content = r#"
title = "Hello"
description = "hey there"
date = "2016-10-10""#;
    let res = FrontMatter::parse(content).unwrap();
    assert!(res.date().is_some());
}

#[test]
fn test_parse_date_rfc3339() {
    let content = r#"
title = "Hello"
description = "hey there"
date = "2002-10-02T15:00:00Z""#;
    let res = FrontMatter::parse(content).unwrap();
    assert!(res.date().is_some());
}

#[test]
fn test_cant_parse_random_date_format() {
    let content = r#"
title = "Hello"
description = "hey there"
date = "2002/10/12""#;
    let res = FrontMatter::parse(content).unwrap();
    assert!(res.date().is_none());
}

#[test]
fn test_cant_parse_sort_by_date() {
    let content = r#"
title = "Hello"
description = "hey there"
sort_by = "date""#;
    let res = FrontMatter::parse(content).unwrap();
    assert!(res.sort_by.is_some());
    assert!(res.sort_by.unwrap(), SortBy::Date);
}

#[test]
fn test_cant_parse_sort_by_order() {
    let content = r#"
title = "Hello"
description = "hey there"
sort_by = "order""#;
    let res = FrontMatter::parse(content).unwrap();
    assert!(res.sort_by.is_some());
    assert!(res.sort_by.unwrap(), SortBy::Order);
}

#[test]
fn test_cant_parse_sort_by_none() {
    let content = r#"
title = "Hello"
description = "hey there"
sort_by = "none""#;
    let res = FrontMatter::parse(content).unwrap();
    assert!(res.sort_by.is_some());
    assert!(res.sort_by.unwrap(), SortBy::None);
}

#[test]
fn test_can_split_content_valid() {
    let content = r#"
+++
title = "Title"
description = "hey there"
date = "2002/10/12"
+++
Hello
"#;
    let (front_matter, content) = split_content(Path::new(""), content).unwrap();
    assert_eq!(content, "Hello\n");
    assert_eq!(front_matter.title.unwrap(), "Title");
}

#[test]
fn test_can_split_content_with_only_frontmatter_valid() {
    let content = r#"
+++
title = "Title"
description = "hey there"
date = "2002/10/12"
+++"#;
    let (front_matter, content) = split_content(Path::new(""), content).unwrap();
    assert_eq!(content, "");
    assert_eq!(front_matter.title.unwrap(), "Title");
}

#[test]
fn test_can_split_content_lazily() {
    let content = r#"
+++
title = "Title"
description = "hey there"
date = "2002-10-02T15:00:00Z"
+++
+++"#;
    let (front_matter, content) = split_content(Path::new(""), content).unwrap();
    assert_eq!(content, "+++");
    assert_eq!(front_matter.title.unwrap(), "Title");
}

#[test]
fn test_error_if_cannot_locate_frontmatter() {
    let content = r#"
+++
title = "Title"
description = "hey there"
date = "2002/10/12"
"#;
    let res = split_content(Path::new(""), content);
    assert!(res.is_err());
}
