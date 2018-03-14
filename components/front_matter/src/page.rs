use std::result::{Result as StdResult};

use chrono::prelude::*;
use tera::{Map, Value};
use serde::{Deserialize, Deserializer};
use toml;

use errors::Result;


fn from_toml_datetime<'de, D>(deserializer: D) -> StdResult<Option<String>, D::Error>
    where
        D: Deserializer<'de>,
{
    toml::value::Datetime::deserialize(deserializer)
        .map(|s| Some(s.to_string()))
}

/// Returns key/value for a converted date from TOML.
/// If the table itself is the TOML struct, only return its value without the key
fn convert_toml_date(table: Map<String, Value>) -> Value {
    let mut new = Map::new();

    for (k, v) in table.into_iter() {
        if k == "$__toml_private_datetime" {
            return v;
        }

        match v {
            Value::Object(mut o) => {
                // that was a toml datetime object, just return the date
                if let Some(toml_date) = o.remove("$__toml_private_datetime") {
                    new.insert(k, toml_date);
                    return Value::Object(new);
                }
                new.insert(k, convert_toml_date(o));
            },
            _ => { new.insert(k, v); }
        }
    }

    Value::Object(new)
}

/// TOML datetimes will be serialized as a struct but we want the
/// stringified version for json, otherwise they are going to be weird
fn fix_toml_dates(table: Map<String, Value>) -> Value {
    let mut new = Map::new();

    for (key, value) in table {
        match value {
            Value::Object(mut o) => {
                new.insert(key, convert_toml_date(o));
            },
            _ => { new.insert(key, value); },
        }
    }

    Value::Object(new)
}


/// The front matter of every page
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageFrontMatter {
    /// <title> of the page
    pub title: Option<String>,
    /// Description in <meta> that appears when linked, e.g. on twitter
    pub description: Option<String>,
    /// Date if we want to order pages (ie blog post)
    #[serde(default, deserialize_with = "from_toml_datetime")]
    pub date: Option<String>,
    /// Whether this page is a draft and should be ignored for pagination etc
    pub draft: Option<bool>,
    /// The page slug. Will be used instead of the filename if present
    /// Can't be an empty string if present
    pub slug: Option<String>,
    /// The path the page appears at, overrides the slug if set in the front-matter
    /// otherwise is set after parsing front matter and sections
    /// Can't be an empty string if present
    pub path: Option<String>,
    /// Tags, not to be confused with categories
    pub tags: Option<Vec<String>>,
    /// Only one category allowed. Can't be an empty string if present
    pub category: Option<String>,
    /// Integer to use to order content. Lowest is at the bottom, highest first
    pub order: Option<usize>,
    /// Integer to use to order content. Highest is at the bottom, lowest first
    pub weight: Option<usize>,
    /// All aliases for that page. Gutenberg will create HTML templates that will
    /// redirect to this
    #[serde(skip_serializing)]
    pub aliases: Option<Vec<String>>,
    /// Specify a template different from `page.html` to use for that page
    #[serde(skip_serializing)]
    pub template: Option<String>,
    /// Whether the page is included in the search index
    /// Defaults to `true` but is only used if search if explicitly enabled in the config.
    #[serde(default, skip_serializing)]
    pub in_search_index: bool,
    /// Any extra parameter present in the front matter
    #[serde(default)]
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

        if let Some(ref category) = f.category {
            if category == "" {
                bail!("`category` can't be empty if present")
            }
        }

        f.extra = match fix_toml_dates(f.extra) {
            Value::Object(o) => o,
            _ => unreachable!("Got something other than a table in page extra"),
        };
        Ok(f)
    }

    /// Converts the TOML datetime to a Chrono naive datetime
    pub fn date(&self) -> Option<NaiveDateTime> {
        if let Some(ref d) = self.date {
            if d.contains('T') {
                DateTime::parse_from_rfc3339(&d).ok().and_then(|s| Some(s.naive_local()))
            } else {
                NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok().and_then(|s| Some(s.and_hms(0, 0, 0)))
            }
        } else {
            None
        }
    }

    pub fn order(&self) -> usize {
        self.order.unwrap()
    }

    pub fn weight(&self) -> usize {
        self.weight.unwrap()
    }

    pub fn has_tags(&self) -> bool {
        match self.tags {
            Some(ref t) => !t.is_empty(),
            None => false
        }
    }
}

impl Default for PageFrontMatter {
    fn default() -> PageFrontMatter {
        PageFrontMatter {
            title: None,
            description: None,
            date: None,
            draft: None,
            slug: None,
            path: None,
            tags: None,
            category: None,
            order: None,
            weight: None,
            aliases: None,
            in_search_index: true,
            template: None,
            extra: Map::new(),
        }
    }
}


#[cfg(test)]
mod tests {
    use tera::to_value;
    use super::PageFrontMatter;

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
    fn can_parse_tags() {
        let content = r#"
    title = "Hello"
    description = "hey there"
    slug = "hello-world"
    tags = ["rust", "html"]"#;
        let res = PageFrontMatter::parse(content);
        assert!(res.is_ok());
        let res = res.unwrap();

        assert_eq!(res.title.unwrap(), "Hello".to_string());
        assert_eq!(res.slug.unwrap(), "hello-world".to_string());
        assert_eq!(res.tags.unwrap(), ["rust".to_string(), "html".to_string()]);
    }

    #[test]
    fn errors_with_invalid_front_matter() {
        let content = r#"title = 1\n"#;
        let res = PageFrontMatter::parse(content);
        assert!(res.is_err());
    }

    #[test]
    fn errors_on_non_string_tag() {
        let content = r#"
    title = "Hello"
    description = "hey there"
    slug = "hello-world"
    tags = ["rust", 1]"#;
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
}
