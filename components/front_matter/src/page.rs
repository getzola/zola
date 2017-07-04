use std::collections::HashMap;

use chrono::prelude::*;
use tera::Value;
use toml;

use errors::Result;

/// The front matter of every page
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageFrontMatter {
    /// <title> of the page
    pub title: Option<String>,
    /// Description in <meta> that appears when linked, e.g. on twitter
    pub description: Option<String>,
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
    /// Only one category allowed. Can't be an empty string if present
    pub category: Option<String>,
    /// Integer to use to order content. Lowest is at the bottom, highest first
    pub order: Option<usize>,
    /// Integer to use to order content. Highest is at the bottom, lowest first
    pub weight: Option<usize>,
    /// All aliases for that page. Gutenberg will create HTML templates that will
    #[serde(skip_serializing)]
    pub aliases: Option<Vec<String>>,
    /// Specify a template different from `page.html` to use for that page
    #[serde(skip_serializing)]
    pub template: Option<String>,
    /// Any extra parameter present in the front matter
    pub extra: Option<HashMap<String, Value>>,
}

impl PageFrontMatter {
    pub fn parse(toml: &str) -> Result<PageFrontMatter> {
        let f: PageFrontMatter = match toml::from_str(toml) {
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

        if let Some(ref category) = f.category {
            if category == "" {
                bail!("`category` can't be empty if present")
            }
        }

        Ok(f)
    }

    /// Converts the date in the front matter, which can be in 2 formats, into a NaiveDateTime
    pub fn date(&self) -> Option<NaiveDateTime> {
        match self.date {
            Some(ref d) => {
                if d.contains('T') {
                    DateTime::parse_from_rfc3339(d).ok().and_then(|s| Some(s.naive_local()))
                } else {
                    NaiveDate::parse_from_str(d, "%Y-%m-%d").ok().and_then(|s| Some(s.and_hms(0,0,0)))
                }
            },
            None => None,
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
            slug: None,
            url: None,
            tags: None,
            category: None,
            order: None,
            weight: None,
            aliases: None,
            template: None,
            extra: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PageFrontMatter;

    #[test]
    fn can_have_empty_front_matter() {
        let content = r#"  "#;
        let res = PageFrontMatter::parse(content);
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
    fn errors_on_present_but_empty_url() {
        let content = r#"
    title = "Hello"
    description = "hey there"
    url = """#;
        let res = PageFrontMatter::parse(content);
        assert!(res.is_err());
    }

    #[test]
    fn can_parse_date_yyyy_mm_dd() {
        let content = r#"
    title = "Hello"
    description = "hey there"
    date = "2016-10-10""#;
        let res = PageFrontMatter::parse(content).unwrap();
        assert!(res.date().is_some());
    }

    #[test]
    fn can_parse_date_rfc3339() {
        let content = r#"
    title = "Hello"
    description = "hey there"
    date = "2002-10-02T15:00:00Z""#;
        let res = PageFrontMatter::parse(content).unwrap();
        assert!(res.date().is_some());
    }

    #[test]
    fn cannot_parse_random_date_format() {
        let content = r#"
    title = "Hello"
    description = "hey there"
    date = "2002/10/12""#;
        let res = PageFrontMatter::parse(content).unwrap();
        assert!(res.date().is_none());
    }

}
