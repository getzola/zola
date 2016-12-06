/// A page, can be a blog post or a basic page
use std::collections::{HashMap, BTreeMap};
use std::default::Default;

// use pulldown_cmark as cmark;
use regex::Regex;
use toml::{Parser, Value as TomlValue};
use tera::{Tera, Value, to_value, Context};

use errors::{Result};
use errors::ErrorKind::InvalidFrontMatter;
use config::Config;


lazy_static! {
    static ref DELIM_RE: Regex = Regex::new(r"\+\+\+\s*\r?\n").unwrap();
}

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
            }).collect::<BTreeMap<_,_>>())
        }
    }
}


#[derive(Debug, PartialEq)]
struct Page {
    // <title> of the page
    title: String,
    // the url the page appears at (slug form)
    url: String,
    // the actual content of the page
    content: String,
    // tags, not to be confused with categories
    tags: Vec<String>,
    // whether this page should be public or not
    is_draft: bool,
    // any extra parameter present in the front matter
    // it will be passed to the template context
    extra: HashMap<String, Value>,

    // only one category allowed
    category: Option<String>,
    // optional date if we want to order pages (ie blog post)
    date: Option<String>,
    // optional layout, if we want to specify which html to render for that page
    layout: Option<String>,
    // description that appears when linked, e.g. on twitter
    description: Option<String>,
}


impl Default for Page {
    fn default() -> Page {
        Page {
            title: "".to_string(),
            url: "".to_string(),
            content: "".to_string(),
            tags: vec![],
            is_draft: false,
            extra: HashMap::new(),

            category: None,
            date: None,
            layout: None,
            description: None,
        }
    }
}


impl Page {
    // Parse a page given the content of the .md file
    // Files without front matter or with invalid front matter are considered
    // erroneous
    pub fn from_str(filename: &str, content: &str) -> Result<Page> {
        // 1. separate front matter from content
        if !DELIM_RE.is_match(content) {
            return Err(InvalidFrontMatter(filename.to_string()).into());
        }

        // 2. extract the front matter and the content
        let splits: Vec<&str> = DELIM_RE.splitn(content, 2).collect();
        let front_matter = splits[0];
        if front_matter.trim() == "" {
            return Err(InvalidFrontMatter(filename.to_string()).into());
        }

        let content = splits[1];

        // 2. create our page, parse front matter and assign all of that
        let mut page = Page::default();
        page.content = content.to_string();

        // Keeps track of required fields: title, url
        let mut num_required_fields = 2;
        let mut parser = Parser::new(&front_matter);

        if let Some(value) = parser.parse() {
            for (key, value) in value.iter() {
                if key == "title" {
                    page.title = value
                        .as_str()
                        .ok_or(InvalidFrontMatter(filename.to_string()))?
                        .to_string();
                    num_required_fields -= 1;
                } else if key == "url" {
                    page.url = value
                        .as_str()
                        .ok_or(InvalidFrontMatter(filename.to_string()))?
                        .to_string();
                    num_required_fields -= 1;
                } else if key == "draft" {
                    page.is_draft = value
                        .as_bool()
                        .ok_or(InvalidFrontMatter(filename.to_string()))?;
                } else if key == "category" {
                    page.category = Some(
                        value
                            .as_str()
                            .ok_or(InvalidFrontMatter(filename.to_string()))?.to_string()
                    );
                } else if key == "layout" {
                    page.layout = Some(
                        value
                            .as_str()
                            .ok_or(InvalidFrontMatter(filename.to_string()))?.to_string()
                    );
                } else if key == "description" {
                    page.description = Some(
                        value
                            .as_str()
                            .ok_or(InvalidFrontMatter(filename.to_string()))?.to_string()
                    );
                } else if key == "date" {
                    page.date = Some(
                        value
                            .as_datetime()
                            .ok_or(InvalidFrontMatter(filename.to_string()))?.to_string()
                    );
                } else if key == "tags" {
                    let toml_tags = value
                        .as_slice()
                        .ok_or(InvalidFrontMatter(filename.to_string()))?;

                    for tag in toml_tags {
                        page.tags.push(
                            tag
                                .as_str()
                                .ok_or(InvalidFrontMatter(filename.to_string()))?
                                .to_string()
                        );
                    }
                } else {
                    page.extra.insert(key.to_string(), toml_to_tera(value));
                }
            }

        } else {
            // TODO: handle error in parsing TOML
            println!("parse errors: {:?}", parser.errors);
        }

        if num_required_fields > 0 {
            println!("Not all required fields");
            return Err(InvalidFrontMatter(filename.to_string()).into());
        }

        Ok(page)
    }

//    pub fn render_html(&self, tera: &Tera, config: &Config) -> Result<String> {
//
//    }
}


#[cfg(test)]
mod tests {
    use super::{Page};
    use tera::to_value;


    #[test]
    fn test_can_parse_a_valid_page() {
        let content = r#"
title = "Hello"
url = "hello-world"
+++
Hello world"#;
        let res = Page::from_str("", content);
        assert!(res.is_ok());
        let page = res.unwrap();

        assert_eq!(page.title, "Hello".to_string());
        assert_eq!(page.url, "hello-world".to_string());
        assert_eq!(page.content, "Hello world".to_string());
    }

    #[test]
    fn test_can_parse_tags() {
        let content = r#"
title = "Hello"
url = "hello-world"
tags = ["rust", "html"]
+++
Hello world"#;
        let res = Page::from_str("", content);
        assert!(res.is_ok());
        let page = res.unwrap();

        assert_eq!(page.title, "Hello".to_string());
        assert_eq!(page.url, "hello-world".to_string());
        assert_eq!(page.content, "Hello world".to_string());
        assert_eq!(page.tags, ["rust".to_string(), "html".to_string()]);
    }

    #[test]
    fn test_can_parse_extra_attributes_in_frontmatter() {
        let content = r#"
title = "Hello"
url = "hello-world"
language = "en"
authors = ["Bob", "Alice"]
+++
Hello world"#;
        let res = Page::from_str("", content);
        assert!(res.is_ok());
        let page = res.unwrap();

        assert_eq!(page.title, "Hello".to_string());
        assert_eq!(page.url, "hello-world".to_string());
        assert_eq!(page.extra.get("language").unwrap(), &to_value("en"));
        assert_eq!(
            page.extra.get("authors").unwrap(),
            &to_value(["Bob".to_string(), "Alice".to_string()])
        );
    }

    #[test]
    fn test_ignore_pages_with_no_front_matter() {
        let content = r#"Hello world"#;
        let res = Page::from_str("", content);
        assert!(res.is_err());
    }

    #[test]
    fn test_ignores_pages_with_empty_front_matter() {
        let content = r#"+++\nHello world"#;
        let res = Page::from_str("", content);
        assert!(res.is_err());
    }

    #[test]
    fn test_ignores_pages_with_invalid_front_matter() {
        let content = r#"title = 1\n+++\nHello world"#;
        let res = Page::from_str("", content);
        assert!(res.is_err());
    }

    #[test]
    fn test_ignores_pages_with_missing_required_value_front_matter() {
        let content = r#"
title = ""
+++
Hello world"#;
        let res = Page::from_str("", content);
        assert!(res.is_err());
    }

    #[test]
    fn test_errors_on_non_string_tag() {
        let content = r#"
title = "Hello"
url = "hello-world"
tags = ["rust", 1]
+++
Hello world"#;
        let res = Page::from_str("", content);
        assert!(res.is_err());
    }
}
