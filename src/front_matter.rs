use std::collections::BTreeMap;


use toml::{Parser, Value as TomlValue};
use tera::{Value, to_value};


use errors::{Result};
use page::Page;


// Converts from one value (Toml) to another (Tera)
// Used to fill the Page::extra map
fn toml_to_tera(val: &TomlValue) -> Value {
    match *val {
        TomlValue::String(ref s) | TomlValue::Datetime(ref s) => to_value(s),
        TomlValue::Boolean(ref b) => to_value(b),
        TomlValue::Integer(ref n) => to_value(n),
        TomlValue::Float(ref n) => to_value(n),
        TomlValue::Array(ref arr) => to_value(&arr.into_iter().map(toml_to_tera).collect::<Vec<_>>()),
        TomlValue::Table(ref table) => {
            to_value(&table.into_iter().map(|(k, v)| {
                (k, toml_to_tera(v))
            }).collect::<BTreeMap<_, _>>())
        }
    }
}


pub fn parse_front_matter(front_matter: &str, page: &mut Page) -> Result<()> {
    if front_matter.trim() == "" {
        bail!("Front matter of file is missing");
    }

    let mut parser = Parser::new(&front_matter);

    if let Some(value) = parser.parse() {
        for (key, value) in value.iter() {
            match key.as_str() {
                "title" | "slug" | "url" | "category" | "layout" | "description" => match *value {
                    TomlValue::String(ref s) => {
                        if key == "title" {
                            page.title = s.to_string();
                        } else if key == "slug" {
                            page.slug = s.to_string();
                        } else if key == "url" {
                            page.url = s.to_string();
                        } else if key == "category" {
                            page.category = Some(s.to_string());
                        } else if key == "layout" {
                            page.layout = Some(s.to_string());
                        } else if key == "description" {
                            page.description = Some(s.to_string());
                        }
                    }
                    _ => bail!("Field {} should be a string", key)
                },
                "draft" => match *value {
                    TomlValue::Boolean(b) => page.is_draft = b,
                    _ => bail!("Field {} should be a boolean", key)
                },
                "date" => match *value {
                    TomlValue::Datetime(ref d) => page.date = Some(d.to_string()),
                    _ => bail!("Field {} should be a date", key)
                },
                "tags" => match *value {
                    TomlValue::Array(ref a) => {
                        for elem in a {
                            if key == "tags" {
                                match *elem {
                                    TomlValue::String(ref s) => page.tags.push(s.to_string()),
                                    _ => bail!("Tag `{}` should be a string")
                                }
                            }
                        }
                    },
                    _ => bail!("Field {} should be an array", key)
                },
                // extra fields
                _ => {
                    page.extra.insert(key.to_string(), toml_to_tera(value));
                }
            }
        }
    } else {
        bail!("Errors parsing front matter: {:?}", parser.errors);
    }

    if page.title == "" || (page.slug == "" && page.url == "") {
        bail!("Front matter is missing required fields (title, slug/url or both)");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{parse_front_matter};
    use tera::to_value;
    use page::Page;


    #[test]
    fn test_can_parse_a_valid_front_matter() {
        let content = r#"
title = "Hello"
slug = "hello-world""#;
        let mut page = Page::default();
        let res = parse_front_matter(content, &mut page);
        assert!(res.is_ok());

        assert_eq!(page.title, "Hello".to_string());
        assert_eq!(page.slug, "hello-world".to_string());
    }

    #[test]
    fn test_can_parse_tags() {
        let content = r#"
title = "Hello"
slug = "hello-world"
tags = ["rust", "html"]"#;
        let mut page = Page::default();
        let res = parse_front_matter(content, &mut page);
        assert!(res.is_ok());

        assert_eq!(page.title, "Hello".to_string());
        assert_eq!(page.slug, "hello-world".to_string());
        assert_eq!(page.tags, ["rust".to_string(), "html".to_string()]);
    }

    #[test]
    fn test_can_parse_extra_attributes_in_frontmatter() {
        let content = r#"
title = "Hello"
slug = "hello-world"
language = "en"
authors = ["Bob", "Alice"]"#;
        let mut page = Page::default();
        let res = parse_front_matter(content, &mut page);
        assert!(res.is_ok());

        assert_eq!(page.title, "Hello".to_string());
        assert_eq!(page.slug, "hello-world".to_string());
        assert_eq!(page.extra.get("language").unwrap(), &to_value("en"));
        assert_eq!(
            page.extra.get("authors").unwrap(),
            &to_value(["Bob".to_string(), "Alice".to_string()])
        );
    }

    #[test]
    fn test_is_ok_with_url_instead_of_slug() {
        let content = r#"
title = "Hello"
url = "hello-world""#;
        let mut page = Page::default();
        let res = parse_front_matter(content, &mut page);
        assert!(res.is_ok());
        assert_eq!(page.slug, "".to_string());
        assert_eq!(page.url, "hello-world".to_string());
    }

    #[test]
    fn test_errors_with_empty_front_matter() {
        let content = r#"  "#;
        let mut page = Page::default();
        let res = parse_front_matter(content, &mut page);
        assert!(res.is_err());
    }

    #[test]
    fn test_errors_with_invalid_front_matter() {
        let content = r#"title = 1\n"#;
        let mut page = Page::default();
        let res = parse_front_matter(content, &mut page);
        assert!(res.is_err());
    }

    #[test]
    fn test_errors_with_missing_required_value_front_matter() {
        let content = r#"title = """#;
        let mut page = Page::default();
        let res = parse_front_matter(content, &mut page);
        assert!(res.is_err());
    }

    #[test]
    fn test_errors_on_non_string_tag() {
        let content = r#"
title = "Hello"
slug = "hello-world"
tags = ["rust", 1]"#;
        let mut page = Page::default();
        let res = parse_front_matter(content, &mut page);
        assert!(res.is_err());
    }
}
