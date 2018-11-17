use std::collections::HashMap;

use chrono::prelude::*;
use tera::{Map, Value};
use toml;

use errors::Result;
use utils::de::{fix_toml_dates, from_toml_datetime};

/// The front matter of every page
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct PageFrontMatter {
    /// <title> of the page
    pub title: Option<String>,
    /// Description in <meta> that appears when linked, e.g. on twitter
    pub description: Option<String>,
    /// Date if we want to order pages (ie blog post)
    #[serde(default, deserialize_with = "from_toml_datetime")]
    pub date: Option<String>,
    /// Chrono converted datetime
    #[serde(default, skip_deserializing)]
    pub datetime: Option<NaiveDateTime>,
    /// The converted date into a (year, month, day) tuple
    #[serde(default, skip_deserializing)]
    pub datetime_tuple: Option<(i32, u32, u32)>,
    /// Whether this page is a draft and should be ignored for pagination etc
    pub draft: bool,
    /// The page slug. Will be used instead of the filename if present
    /// Can't be an empty string if present
    pub slug: Option<String>,
    /// The path the page appears at, overrides the slug if set in the front-matter
    /// otherwise is set after parsing front matter and sections
    /// Can't be an empty string if present
    pub path: Option<String>,
    pub taxonomies: HashMap<String, Vec<String>>,
    /// Integer to use to order content. Lowest is at the bottom, highest first
    pub order: Option<usize>,
    /// Integer to use to order content. Highest is at the bottom, lowest first
    pub weight: Option<usize>,
    /// All aliases for that page. Zola will create HTML templates that will
    /// redirect to this
    #[serde(skip_serializing)]
    pub aliases: Vec<String>,
    /// Specify a template different from `page.html` to use for that page
    #[serde(skip_serializing)]
    pub template: Option<String>,
    /// Whether the page is included in the search index
    /// Defaults to `true` but is only used if search if explicitly enabled in the config.
    #[serde(skip_serializing)]
    pub in_search_index: bool,
    /// Any extra parameter present in the front matter
    pub extra: Map<String, Value>,
}

impl PageFrontMatter {
    pub fn parse(toml: &str) -> Result<PageFrontMatter> {
        let mut f: PageFrontMatter = match toml::from_str(toml) {
            Ok(d) => d,
            Err(e) => bail!(e),
        };

        if let Some(ref slug) = f.slug {
            if slug == "" {
                bail!("`slug` can't be empty if present")
            }
        }

        if let Some(ref path) = f.path {
            if path == "" {
                bail!("`path` can't be empty if present")
            }
        }

        f.extra = match fix_toml_dates(f.extra) {
            Value::Object(o) => o,
            _ => unreachable!("Got something other than a table in page extra"),
        };

        f.date_to_datetime();

        Ok(f)
    }

    /// Converts the TOML datetime to a Chrono naive datetime
    /// Also grabs the year/month/day tuple that will be used in serialization
    pub fn date_to_datetime(&mut self) {
        self.datetime = if let Some(ref d) = self.date {
            if d.contains('T') {
                DateTime::parse_from_rfc3339(&d).ok().and_then(|s| Some(s.naive_local()))
            } else {
                NaiveDate::parse_from_str(&d, "%Y-%m-%d")
                    .ok()
                    .and_then(|s| Some(s.and_hms(0, 0, 0)))
            }
        } else {
            None
        };

        self.datetime_tuple = if let Some(ref dt) = self.datetime {
            Some((dt.year(), dt.month(), dt.day()))
        } else {
            None
        };
    }

    pub fn order(&self) -> usize {
        self.order.unwrap()
    }

    pub fn weight(&self) -> usize {
        self.weight.unwrap()
    }
}

impl Default for PageFrontMatter {
    fn default() -> PageFrontMatter {
        PageFrontMatter {
            title: None,
            description: None,
            date: None,
            datetime: None,
            datetime_tuple: None,
            draft: false,
            slug: None,
            path: None,
            taxonomies: HashMap::new(),
            order: None,
            weight: None,
            aliases: Vec::new(),
            in_search_index: true,
            template: None,
            extra: Map::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PageFrontMatter;
    use tera::to_value;

    #[test]
    fn can_have_empty_front_matter() {
        let content = r#"  "#;
        let res = PageFrontMatter::parse(content);
        println!("{:?}", res);
        assert!(res.is_ok());
    }

    #[test]
    fn can_parse_valid_front_matter() {
        let content = r#"
    title = "Hello"
    description = "hey there""#;
        let res = PageFrontMatter::parse(content);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res.title.unwrap(), "Hello".to_string());
        assert_eq!(res.description.unwrap(), "hey there".to_string())
    }

    #[test]
    fn errors_with_invalid_front_matter() {
        let content = r#"title = 1\n"#;
        let res = PageFrontMatter::parse(content);
        assert!(res.is_err());
    }

    #[test]
    fn errors_on_present_but_empty_slug() {
        let content = r#"
    title = "Hello"
    description = "hey there"
    slug = """#;
        let res = PageFrontMatter::parse(content);
        assert!(res.is_err());
    }

    #[test]
    fn errors_on_present_but_empty_path() {
        let content = r#"
    title = "Hello"
    description = "hey there"
    path = """#;
        let res = PageFrontMatter::parse(content);
        assert!(res.is_err());
    }

    #[test]
    fn can_parse_date_yyyy_mm_dd() {
        let content = r#"
    title = "Hello"
    description = "hey there"
    date = 2016-10-10
    "#;
        let res = PageFrontMatter::parse(content).unwrap();
        assert!(res.date.is_some());
    }

    #[test]
    fn can_parse_date_rfc3339() {
        let content = r#"
    title = "Hello"
    description = "hey there"
    date = 2002-10-02T15:00:00Z
    "#;
        let res = PageFrontMatter::parse(content).unwrap();
        assert!(res.date.is_some());
    }

    #[test]
    fn cannot_parse_random_date_format() {
        let content = r#"
    title = "Hello"
    description = "hey there"
    date = 2002/10/12"#;
        let res = PageFrontMatter::parse(content);
        assert!(res.is_err());
    }

    #[test]
    fn cannot_parse_invalid_date_format() {
        let content = r#"
    title = "Hello"
    description = "hey there"
    date = 2002-14-01"#;
        let res = PageFrontMatter::parse(content);
        assert!(res.is_err());
    }

    #[test]
    fn cannot_parse_date_as_string() {
        let content = r#"
    title = "Hello"
    description = "hey there"
    date = "2002-14-01""#;
        let res = PageFrontMatter::parse(content);
        assert!(res.is_err());
    }

    #[test]
    fn can_parse_dates_in_extra() {
        let content = r#"
    title = "Hello"
    description = "hey there"

    [extra]
    some-date = 2002-14-01"#;
        let res = PageFrontMatter::parse(content);
        println!("{:?}", res);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().extra["some-date"], to_value("2002-14-01").unwrap());
    }

    #[test]
    fn can_parse_nested_dates_in_extra() {
        let content = r#"
    title = "Hello"
    description = "hey there"

    [extra.something]
    some-date = 2002-14-01"#;
        let res = PageFrontMatter::parse(content);
        println!("{:?}", res);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().extra["something"]["some-date"], to_value("2002-14-01").unwrap());
    }

    #[test]
    fn can_parse_taxonomies() {
        let content = r#"
title = "Hello World"

[taxonomies]
tags = ["Rust", "JavaScript"]
categories = ["Dev"]
"#;
        let res = PageFrontMatter::parse(content);
        println!("{:?}", res);
        assert!(res.is_ok());
        let res2 = res.unwrap();
        assert_eq!(res2.taxonomies["categories"], vec!["Dev"]);
        assert_eq!(res2.taxonomies["tags"], vec!["Rust", "JavaScript"]);
    }
}
