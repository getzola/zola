use std::collections::HashMap;


use toml;
use tera::Value;
use chrono::prelude::*;


use errors::{Result};


/// The front matter of every page
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrontMatter {
    // Mandatory fields

    /// <title> of the page
    pub title: String,
    /// Description that appears when linked, e.g. on twitter
    pub description: String,

    // Optional stuff

    /// Date if we want to order pages (ie blog post)
    pub date: Option<String>,
    /// The page slug. Will be used instead of the filename if present
    /// Can't be an empty string if present
    pub slug: Option<String>,
    /// The url the page appears at, overrides the slug if set in the front-matter
    /// otherwise is set after parsing front matter and sections
    /// Can't be an empty string if present
    pub url: Option<String>,
    /// Tags, not to be confused with categories
    pub tags: Option<Vec<String>>,
    /// Whether this page is a draft and should be published or not
    pub draft: Option<bool>,
    /// Only one category allowed
    pub category: Option<String>,
    /// Optional layout, if we want to specify which tpl to render for that page
    #[serde(skip_serializing)]
    pub layout: Option<String>,
    /// Any extra parameter present in the front matter
    pub extra: Option<HashMap<String, Value>>,
}

impl FrontMatter {
    pub fn parse(toml: &str) -> Result<FrontMatter> {
        if toml.trim() == "" {
            bail!("Front matter of file is missing");
        }

        let f: FrontMatter = match toml::from_str(toml) {
            Ok(d) => d,
            Err(e) => bail!(e),
        };

        if let Some(ref slug) = f.slug {
            if slug == "" {
                bail!("`slug` can't be empty if present")
            }
        }

        if let Some(ref url) = f.url {
            if url == "" {
                bail!("`url` can't be empty if present")
            }
        }

        Ok(f)
    }

    pub fn parse_date(&self) -> Option<NaiveDateTime> {
        match self.date {
            Some(ref d) => {
                if d.contains("T") {
                    DateTime::parse_from_rfc3339(d).ok().and_then(|s| Some(s.naive_local()))
                } else {
                    NaiveDate::parse_from_str(d, "%Y-%m-%d").ok().and_then(|s| Some(s.and_hms(0,0,0)))
                }
            },
            None => None,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::{FrontMatter};
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
        assert_eq!(res.title, "Hello".to_string());
        assert_eq!(res.description, "hey there".to_string());
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

        assert_eq!(res.title, "Hello".to_string());
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

        assert_eq!(res.title, "Hello".to_string());
        assert_eq!(res.slug.unwrap(), "hello-world".to_string());
        let extra = res.extra.unwrap();
        assert_eq!(extra.get("language").unwrap(), &to_value("en").unwrap());
        assert_eq!(
            extra.get("authors").unwrap(),
            &to_value(["Bob".to_string(), "Alice".to_string()]).unwrap()
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
    fn test_errors_with_missing_required_value_front_matter() {
        let content = r#"title = """#;
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
        assert!(res.parse_date().is_some());
    }

    #[test]
    fn test_parse_date_rfc3339() {
        let content = r#"
title = "Hello"
description = "hey there"
date = "2002-10-02T15:00:00Z""#;
        let res = FrontMatter::parse(content).unwrap();
        assert!(res.parse_date().is_some());
    }

    #[test]
    fn test_cant_parse_random_date_format() {
        let content = r#"
title = "Hello"
description = "hey there"
date = "2002/10/12""#;
        let res = FrontMatter::parse(content).unwrap();
        assert!(res.parse_date().is_none());
    }
}
