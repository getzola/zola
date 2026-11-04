use std::collections::BTreeMap;

use serde::Deserialize;
use tera::{Map, Value};
use time::format_description::well_known::Rfc3339;
use time::macros::{format_description, time};
use time::{Date, OffsetDateTime, PrimitiveDateTime};

mod datetime;
mod extra;
mod split;

use crate::SortBy;
use crate::file_info::Kind;
use crate::front_matter::split::RawFrontMatter;
use datetime::from_unknown_datetime;
use errors::{Result, bail};
use extra::{default_extra, deserialize_extra};
pub use split::split_content;
use utils::types::InsertAnchor;

const DEFAULT_PAGINATE_PATH: &str = "page";

/// Parse a string for a datetime coming from one of the supported TOML format
/// There are three alternatives:
/// 1. an offset datetime (plain RFC3339)
/// 2. a local datetime (RFC3339 with timezone omitted)
/// 3. a local date (YYYY-MM-DD).
///
/// This tries each in order.
fn parse_datetime(d: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(d, &Rfc3339)
        .or_else(|_| OffsetDateTime::parse(format!("{}Z", d).as_ref(), &Rfc3339))
        .or_else(|_| match Date::parse(d, &format_description!("[year]-[month]-[day]")) {
            Ok(date) => Ok(PrimitiveDateTime::new(date, time!(0:00)).assume_utc()),
            Err(e) => Err(e),
        })
        .ok()
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FrontMatter {
    /// <title> of the page
    pub title: Option<String>,
    /// Description in <meta> that appears when linked, e.g. on twitter
    pub description: Option<String>,
    /// Updated date
    #[serde(default, deserialize_with = "from_unknown_datetime")]
    pub updated: Option<String>,
    /// Datetime content was last updated
    #[serde(default, skip_deserializing)]
    pub updated_datetime: Option<OffsetDateTime>,
    /// The converted update datetime into a (year, month, day) tuple
    #[serde(default, skip_deserializing)]
    pub updated_datetime_tuple: Option<(i32, u8, u8)>,
    /// Date if we want to order pages (ie blog post)
    #[serde(default, deserialize_with = "from_unknown_datetime")]
    pub date: Option<String>,
    /// Datetime content was created
    #[serde(default, skip_deserializing)]
    pub datetime: Option<OffsetDateTime>,
    /// The converted date into a (year, month, day) tuple
    #[serde(default, skip_deserializing)]
    pub datetime_tuple: Option<(i32, u8, u8)>,
    /// Whether this page is a draft
    pub draft: bool,
    /// Prevent generation of a folder for current page
    /// Defaults to `true`
    pub render: bool,
    /// The page slug. Will be used instead of the filename if present
    /// Can't be an empty string if present
    pub slug: Option<String>,
    /// The path the page appears at, overrides the slug if set in the front-matter
    /// otherwise is set after parsing front matter and sections
    /// Can't be an empty string if present
    pub path: Option<String>,
    pub taxonomies: BTreeMap<String, Vec<String>>,
    /// Integer to use to order content. Highest is at the bottom, lowest first
    pub weight: Option<usize>,
    /// The authors of the page.
    pub authors: Vec<String>,
    /// All aliases for that page. Zola will create HTML templates that will
    /// redirect to this
    pub aliases: Vec<String>,
    /// Specify a template different from `page.html` to use for that page
    pub template: Option<String>,
    /// Whether the page is included in the search index
    /// Defaults to `true` but is only used if search if explicitly enabled in the config.
    pub in_search_index: bool,
    /// Whether the page is included in feeds (RSS/Atom)
    /// Defaults to `true`
    pub include_in_feeds: bool,
    /// Whether to hide this page everywhere but still render it
    pub hidden: Option<bool>,
    /// Any extra parameter present in the front matter
    #[serde(default = "default_extra", deserialize_with = "deserialize_extra")]
    pub extra: Value,

    // Section specific fields
    /// Whether to sort by "date", "order", "weight" or "none". Defaults to `none`.
    pub sort_by: SortBy,
    /// How many pages to be displayed per paginated page. No pagination will happen if this isn't set
    pub paginate_by: Option<usize>,
    /// Whether to reverse the order of the pages before segmenting into pagers
    pub paginate_reversed: bool,
    /// Path to be used by pagination: the page number will be appended after it. Defaults to `page`.
    pub paginate_path: String,
    /// Whether the section should pass its pages on to the parent section. Defaults to `false`.
    /// Useful when the section shouldn't split up the parent section, like
    /// sections for each year under a posts section.
    pub transparent: bool,
    /// Optional template for all pages in this section (including the pages of children section)
    pub page_template: Option<String>,
    /// Whether to generate a feed for the current section
    pub generate_feeds: bool,
    /// Whether to insert a link for each header like the ones you can see in this site if you hover one
    /// The default template can be overridden by creating a `anchor-link.html` in the `templates` directory
    pub insert_anchor_links: Option<InsertAnchor>,
    /// Whether to redirect when landing on that section. Defaults to `None`.
    /// Useful for the same reason as `render` but when you don't want a 404 when
    /// landing on the root section page
    pub redirect_to: Option<String>,
}

impl Default for FrontMatter {
    fn default() -> Self {
        Self {
            title: None,
            description: None,
            updated: None,
            updated_datetime: None,
            updated_datetime_tuple: None,
            date: None,
            datetime: None,
            datetime_tuple: None,
            draft: false,
            render: true,
            slug: None,
            path: None,
            taxonomies: BTreeMap::new(),
            weight: None,
            authors: Vec::new(),
            aliases: Vec::new(),
            template: None,
            in_search_index: true,
            include_in_feeds: true,
            hidden: None,
            extra: Value::from(Map::new()),
            sort_by: SortBy::None,
            paginate_by: None,
            paginate_reversed: false,
            paginate_path: DEFAULT_PAGINATE_PATH.to_string(),
            transparent: false,
            page_template: None,
            generate_feeds: false,
            insert_anchor_links: None,
            redirect_to: None,
        }
    }
}

impl FrontMatter {
    /// Converts the TOML datetime to a time::OffsetDateTime
    /// Also grabs the year/month/day tuple that will be used in serialization
    pub fn date_to_datetime(&mut self) {
        self.datetime = self.date.as_ref().map(|s| s.as_ref()).and_then(parse_datetime);
        self.datetime_tuple = self.datetime.map(|dt| (dt.year(), dt.month().into(), dt.day()));

        self.updated_datetime = self.updated.as_ref().map(|s| s.as_ref()).and_then(parse_datetime);
        self.updated_datetime_tuple =
            self.updated_datetime.map(|dt| (dt.year(), dt.month().into(), dt.day()));
    }

    pub fn parse(raw: &RawFrontMatter, kind: Kind) -> Result<Self> {
        let mut f: FrontMatter = raw.deserialize()?;

        // We want to error if users put something like paginate on a page because it's mostly
        // likely a mistake and we have unified them so no more deny_unknown_fields for that.
        if kind == Kind::Page {
            if f.sort_by != SortBy::None {
                bail!("`sort_by` can only be set on sections");
            }
            if f.paginate_by.is_some() {
                bail!("`paginate_by` can only be set on sections");
            }
            if f.paginate_reversed {
                bail!("`paginate_reversed` can only be set on sections");
            }
            if f.paginate_path != DEFAULT_PAGINATE_PATH {
                bail!("`paginate_path` can only be set on sections");
            }
            if f.redirect_to.is_some() {
                bail!("`redirect_to` can only be set on sections");
            }
            if f.transparent {
                bail!("`transparent` can only be set on sections");
            }
            if f.generate_feeds {
                bail!("`generate_feeds` can only be set on sections");
            }
            if f.page_template.is_some() {
                bail!("`page_template` can only be set on sections");
            }
            if f.insert_anchor_links.is_some() {
                bail!("`insert_anchor_links` can only be set on sections");
            }
        } else {
            if f.slug.is_some() {
                bail!("`slug` can only be set on pages");
            }
            if f.path.is_some() {
                bail!("`path` can only be set on pages");
            }
        }

        // Generic validation
        if let Some(ref slug) = f.slug
            && slug.is_empty()
        {
            bail!("`slug` can't be empty if present")
        }

        if let Some(ref path) = f.path
            && path.is_empty()
        {
            bail!("`path` can't be empty if present")
        }

        f.date_to_datetime();

        for terms in f.taxonomies.values() {
            for term in terms {
                if term.trim().is_empty() {
                    bail!("A taxonomy term cannot be an empty string");
                }
            }
        }

        if let Some(ref date) = f.date
            && f.datetime.is_none()
        {
            bail!("`date` could not be parsed: {}.", date);
        }

        Ok(f)
    }

    pub fn weight(&self) -> usize {
        self.weight.unwrap_or_default()
    }

    pub fn is_paginated(&self) -> bool {
        self.paginate_by.is_some_and(|v| v > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::FrontMatter;
    use crate::file_info::Kind;
    use crate::front_matter::split::RawFrontMatter;
    use tera::Value;
    use tera::value::Key;
    use test_case::test_case;
    use time::macros::datetime;

    #[test_case(&RawFrontMatter::Toml(r#"  "#); "toml")]
    #[test_case(&RawFrontMatter::Toml(r#"  "#); "yaml")]
    fn can_have_empty_front_matter(content: &RawFrontMatter) {
        let res = FrontMatter::parse(content, Kind::Page);
        println!("{:?}", res);
        assert!(res.is_ok());
    }

    #[test]
    fn include_in_feeds_defaults_to_true() {
        let content = &RawFrontMatter::Toml(r#"title = "Hello""#);
        let res = FrontMatter::parse(content, Kind::Page).unwrap();
        assert!(res.include_in_feeds);
    }

    #[test_case(&RawFrontMatter::Toml(r#"
title = "Hello"
include_in_feeds = false
"#); "toml")]
    #[test_case(&RawFrontMatter::Yaml(r#"
title: Hello
include_in_feeds: false
"#); "yaml")]
    fn can_disable_include_in_feeds(content: &RawFrontMatter) {
        let res = FrontMatter::parse(content, Kind::Page).unwrap();
        assert!(!res.include_in_feeds);
    }

    #[test_case(&RawFrontMatter::Toml(r#"
title = "Hello"
description = "hey there"
"#); "toml")]
    #[test_case(&RawFrontMatter::Yaml(r#"
title: Hello
description: hey there
"#); "yaml")]
    fn can_parse_valid_front_matter(content: &RawFrontMatter) {
        let res = FrontMatter::parse(content, Kind::Page);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res.title.unwrap(), "Hello".to_string());
        assert_eq!(res.description.unwrap(), "hey there".to_string())
    }

    #[test_case(&RawFrontMatter::Toml(r#"title = |\n"#); "toml")]
    #[test_case(&RawFrontMatter::Yaml(r#"title: |\n"#); "yaml")]
    fn errors_with_invalid_front_matter(content: &RawFrontMatter) {
        let res = FrontMatter::parse(content, Kind::Page);
        assert!(res.is_err());
    }

    #[test_case(&RawFrontMatter::Toml(r#"
title = "Hello"
description = "hey there"
slug = ""
"#); "toml")]
    #[test_case(&RawFrontMatter::Yaml(r#"
title: Hello
description: hey there
slug: ""
"#); "yaml")]
    fn errors_on_present_but_empty_slug(content: &RawFrontMatter) {
        let res = FrontMatter::parse(content, Kind::Page);
        assert!(res.is_err());
    }

    #[test_case(&RawFrontMatter::Toml(r#"
title = "Hello"
description = "hey there"
path = ""
"#); "toml")]
    #[test_case(&RawFrontMatter::Yaml(r#"
title: Hello
description: hey there
path: ""
"#); "yaml")]
    fn errors_on_present_but_empty_path(content: &RawFrontMatter) {
        let res = FrontMatter::parse(content, Kind::Page);
        assert!(res.is_err());
    }

    #[test_case(&RawFrontMatter::Toml(r#"
title = "Hello"
description = "hey there"
date = 2016-10-10
"#); "toml")]
    #[test_case(&RawFrontMatter::Yaml(r#"
title: Hello
description: hey there
date: 2016-10-10
"#); "yaml")]
    fn can_parse_date_yyyy_mm_dd(content: &RawFrontMatter) {
        let res = FrontMatter::parse(content, Kind::Page).unwrap();
        assert!(res.datetime.is_some());
        assert_eq!(res.datetime.unwrap(), datetime!(2016 - 10 - 10 0:00 UTC));
    }

    #[test_case(&RawFrontMatter::Toml(r#"
title = "Hello"
description = "hey there"
date = 2002-10-02T15:00:00Z
"#); "toml")]
    #[test_case(&RawFrontMatter::Yaml(r#"
title: Hello
description: hey there
date: 2002-10-02T15:00:00Z
"#); "yaml")]
    fn can_parse_date_rfc3339(content: &RawFrontMatter) {
        let res = FrontMatter::parse(content, Kind::Page).unwrap();
        assert!(res.datetime.is_some());
        assert_eq!(res.datetime.unwrap(), datetime!(2002 - 10 - 02 15:00:00 UTC));
    }

    #[test_case(&RawFrontMatter::Toml(r#"
title = "Hello"
description = "hey there"
date = 2002-10-02T15:00:00
"#); "toml")]
    #[test_case(&RawFrontMatter::Yaml(r#"
title: Hello
description: hey there
date: 2002-10-02T15:00:00
"#); "yaml")]
    fn can_parse_date_rfc3339_without_timezone(content: &RawFrontMatter) {
        let res = FrontMatter::parse(content, Kind::Page).unwrap();
        assert!(res.datetime.is_some());
        assert_eq!(res.datetime.unwrap(), datetime!(2002 - 10 - 02 15:00:00 UTC));
    }

    #[test_case(&RawFrontMatter::Toml(r#"
title = "Hello"
description = "hey there"
date = 2002-10-02 15:00:00+02:00
"#); "toml")]
    #[test_case(&RawFrontMatter::Yaml(r#"
title: Hello
description: hey there
date: 2002-10-02 15:00:00+02:00
"#); "yaml")]
    fn can_parse_date_rfc3339_with_space(content: &RawFrontMatter) {
        let res = FrontMatter::parse(content, Kind::Page).unwrap();
        assert!(res.datetime.is_some());
        assert_eq!(res.datetime.unwrap(), datetime!(2002 - 10 - 02 15:00:00+02:00));
    }

    #[test_case(&RawFrontMatter::Toml(r#"
title = "Hello"
description = "hey there"
date = 2002-10-02 15:00:00
"#); "toml")]
    #[test_case(&RawFrontMatter::Yaml(r#"
title: Hello
description: hey there
date: 2002-10-02 15:00:00
"#); "yaml")]
    fn can_parse_date_rfc3339_with_space_without_timezone(content: &RawFrontMatter) {
        let res = FrontMatter::parse(content, Kind::Page).unwrap();
        assert!(res.datetime.is_some());
        assert_eq!(res.datetime.unwrap(), datetime!(2002 - 10 - 02 15:00:00 UTC));
    }

    #[test_case(&RawFrontMatter::Toml(r#"
title = "Hello"
description = "hey there"
date = 2002-10-02T15:00:00.123456Z
"#); "toml")]
    #[test_case(&RawFrontMatter::Yaml(r#"
title: Hello
description: hey there
date: 2002-10-02T15:00:00.123456Z
"#); "yaml")]
    fn can_parse_date_rfc3339_with_microseconds(content: &RawFrontMatter) {
        let res = FrontMatter::parse(content, Kind::Page).unwrap();
        assert!(res.datetime.is_some());
        assert_eq!(res.datetime.unwrap(), datetime!(2002 - 10 - 02 15:00:00.123456 UTC));
    }

    #[test_case(&RawFrontMatter::Yaml(r#"
title: Hello
description: hey there
date: 2001-12-15T02:59:43.1Z
"#); "canonical")]
    #[test_case(&RawFrontMatter::Yaml(r#"
title: Hello
description: hey there
date: 2001-12-14t21:59:43.10-05:00
"#); "iso8601")]
    #[test_case(&RawFrontMatter::Yaml(r#"
title: Hello
description: hey there
date: 2001-12-14 21:59:43.10 -5
"#); "space separated")]
    #[test_case(&RawFrontMatter::Yaml(r#"
title: Hello
description: hey there
date: 2001-12-15 2:59:43.10
"#); "no time zone")]
    #[test_case(&RawFrontMatter::Yaml(r#"
title: Hello
description: hey there
date: 2001-12-15
"#); "date only")]
    fn can_parse_yaml_dates(content: &RawFrontMatter) {
        let res = FrontMatter::parse(content, Kind::Page).unwrap();
        assert!(res.datetime.is_some());
    }

    #[test_case(&RawFrontMatter::Toml(r#"
title = "Hello"
description = "hey there"
date = 2002/10/12
"#); "toml")]
    #[test_case(&RawFrontMatter::Yaml(r#"
title: Hello
description: hey there
date: 2002/10/12
"#); "yaml")]
    fn cannot_parse_random_date_format(content: &RawFrontMatter) {
        let res = FrontMatter::parse(content, Kind::Page);
        assert!(res.is_err());
    }

    #[test_case(&RawFrontMatter::Toml(r#"
title = "Hello"
description = "hey there"
date = 2002-14-01
"#); "toml")]
    #[test_case(&RawFrontMatter::Yaml(r#"
title: Hello
description: hey there
date: 2002-14-01
"#); "yaml")]
    fn cannot_parse_invalid_date_format(content: &RawFrontMatter) {
        let res = FrontMatter::parse(content, Kind::Page);
        assert!(res.is_err());
    }

    #[test_case(&RawFrontMatter::Toml(r#"
title = "Hello"
description = "hey there"
date = "2016-10-10"
"#); "toml")]
    #[test_case(&RawFrontMatter::Yaml(r#"
title: Hello
description: hey there
date: "2016-10-10"
"#); "yaml")]
    fn can_parse_valid_date_as_string(content: &RawFrontMatter) {
        let res = FrontMatter::parse(content, Kind::Page).unwrap();
        assert!(res.date.is_some());
        assert!(res.datetime.is_some());
        assert_eq!(res.datetime.unwrap(), datetime!(2016 - 10 - 10 0:00 UTC));
    }

    #[test_case(&RawFrontMatter::Toml(r#"
title = "Hello"
description = "hey there"
date = "2002-14-01"
"#); "toml")]
    #[test_case(&RawFrontMatter::Yaml(r#"
title: Hello
description: hey there
date: "2002-14-01"
"#); "yaml")]
    fn cannot_parse_invalid_date_as_string(content: &RawFrontMatter) {
        let res = FrontMatter::parse(content, Kind::Page);
        assert!(res.is_err());
    }

    #[test_case(&RawFrontMatter::Toml(r#"
title = "Hello"
description = "hey there"

[extra]
some-date = 2002-11-01
"#); "toml")]
    #[test_case(&RawFrontMatter::Yaml(r#"
title: Hello
description: hey there

extra:
    some-date: 2002-11-01
"#); "yaml")]
    fn can_parse_dates_in_extra(content: &RawFrontMatter) {
        let res = FrontMatter::parse(content, Kind::Page);
        println!("{:?}", res);
        assert!(res.is_ok());
        assert_eq!(
            res.unwrap().extra.as_map().unwrap().get(&Key::from("some-date")).unwrap(),
            &Value::from("2002-11-01")
        );
    }

    #[test_case(&RawFrontMatter::Toml(r#"
title = "Hello"
description = "hey there"

[extra.something]
some-date = 2002-11-01
"#); "toml")]
    #[test_case(&RawFrontMatter::Yaml(r#"
title: Hello
description: hey there

extra:
    something:
        some-date: 2002-11-01
"#); "yaml")]
    fn can_parse_nested_dates_in_extra(content: &RawFrontMatter) {
        let res = FrontMatter::parse(content, Kind::Page);
        println!("{:?}", res);
        assert!(res.is_ok());
        assert_eq!(
            res.unwrap()
                .extra
                .as_map()
                .unwrap()
                .get(&Key::from("something"))
                .unwrap()
                .as_map()
                .unwrap()
                .get(&Key::from("some-date"))
                .unwrap(),
            &Value::from("2002-11-01")
        );
    }

    #[test_case(&RawFrontMatter::Toml(r#"
title = "Hello"
description = "hey there"

[extra]
date_example = 2020-05-04
[[extra.questions]]
date = 2020-05-03
name = "Who is the prime minister of Uganda?"
"#); "toml")]
    #[test_case(&RawFrontMatter::Yaml(r#"
title: Hello
description: hey there

extra:
    date_example: 2020-05-04
    questions:
        - date: 2020-05-03
          name: "Who is the prime minister of Uganda?"
"#); "yaml")]
    fn can_parse_fully_nested_dates_in_extra(content: &RawFrontMatter) {
        let res = FrontMatter::parse(content, Kind::Page);
        println!("{:?}", res);
        assert!(res.is_ok());
        assert_eq!(
            res.unwrap()
                .extra
                .as_map()
                .unwrap()
                .get(&Key::from("questions"))
                .unwrap()
                .as_array()
                .unwrap()[0]
                .as_map()
                .unwrap()
                .get(&Key::from("date"))
                .unwrap(),
            &Value::from("2020-05-03")
        );
    }

    #[test_case(&RawFrontMatter::Toml(r#"
title = "Hello World"

[taxonomies]
tags = ["Rust", "JavaScript"]
categories = ["Dev"]
"#); "toml")]
    #[test_case(&RawFrontMatter::Yaml(r#"
title: Hello World

taxonomies:
    tags:
        - Rust
        - JavaScript
    categories:
        - Dev
"#); "yaml")]
    fn can_parse_taxonomies(content: &RawFrontMatter) {
        let res = FrontMatter::parse(content, Kind::Page);
        println!("{:?}", res);
        assert!(res.is_ok());
        let res2 = res.unwrap();
        assert_eq!(res2.taxonomies["categories"], vec!["Dev"]);
        assert_eq!(res2.taxonomies["tags"], vec!["Rust", "JavaScript"]);
    }

    #[test_case(&RawFrontMatter::Toml(r#"
title = "Hello World"

[taxonomies]
tags = [""]
"#); "toml")]
    #[test_case(&RawFrontMatter::Yaml(r#"
title: Hello World

taxonomies:
    tags:
        -
"#); "yaml")]
    fn errors_on_empty_taxonomy_term(content: &RawFrontMatter) {
        // https://github.com/getzola/zola/issues/2085
        let res = FrontMatter::parse(content, Kind::Page);
        println!("{:?}", res);
        assert!(res.is_err());
    }

    #[test_case(&RawFrontMatter::Toml(r#"
authors = ["person1@example.com (Person One)", "person2@example.com (Person Two)"]
"#); "toml")]
    #[test_case(&RawFrontMatter::Yaml(r#"
title: Hello World
authors:
    - person1@example.com (Person One)
    - person2@example.com (Person Two)
"#); "yaml")]
    fn can_parse_authors(content: &RawFrontMatter) {
        let res = FrontMatter::parse(content, Kind::Page);
        assert!(res.is_ok());
        let res2 = res.unwrap();
        assert_eq!(res2.authors.len(), 2);
        assert_eq!(
            vec!(
                "person1@example.com (Person One)".to_owned(),
                "person2@example.com (Person Two)".to_owned()
            ),
            res2.authors
        );
    }

    #[test_case(r#"sort_by = "date""#, "sort_by"; "sort_by")]
    #[test_case(r#"paginate_by = 5"#, "paginate_by"; "paginate_by")]
    #[test_case(r#"paginate_reversed = true"#, "paginate_reversed"; "paginate_reversed")]
    #[test_case(r#"paginate_path = "p""#, "paginate_path"; "paginate_path")]
    #[test_case(r#"redirect_to = "/hey""#, "redirect_to"; "redirect_to")]
    #[test_case(r#"transparent = true"#, "transparent"; "transparent")]
    #[test_case(r#"generate_feeds = true"#, "generate_feeds"; "generate_feeds")]
    #[test_case(r#"page_template = "a.html""#, "page_template"; "page_template")]
    #[test_case(r#"insert_anchor_links = "left""#, "insert_anchor_links"; "insert_anchor_links")]
    fn errors_on_section_field_on_page(content: &str, field: &str) {
        let res = FrontMatter::parse(&RawFrontMatter::Toml(content), Kind::Page);
        let err = res.unwrap_err().to_string();
        assert!(err.contains(field), "{err}");
        assert!(err.contains("sections"), "{err}");
        // but it is fine on a section
        assert!(FrontMatter::parse(&RawFrontMatter::Toml(content), Kind::Section).is_ok());
    }

    #[test_case(r#"slug = "hey""#, "slug"; "slug")]
    #[test_case(r#"path = "/hey""#, "path"; "path")]
    fn errors_on_page_field_on_section(content: &str, field: &str) {
        let res = FrontMatter::parse(&RawFrontMatter::Toml(content), Kind::Section);
        let err = res.unwrap_err().to_string();
        assert!(err.contains(field), "{err}");
        assert!(err.contains("pages"), "{err}");
        assert!(FrontMatter::parse(&RawFrontMatter::Toml(content), Kind::Page).is_ok());
    }

    #[test]
    fn section_can_have_date_authors_and_taxonomies() {
        let content = RawFrontMatter::Toml(
            r#"
date = 2021-01-01
updated = 2021-02-01
authors = ["me"]
[taxonomies]
tags = ["rust"]
"#,
        );
        let res = FrontMatter::parse(&content, Kind::Section).unwrap();
        assert!(res.datetime.is_some());
        assert!(res.updated_datetime.is_some());
        assert_eq!(res.authors, vec!["me".to_string()]);
        assert_eq!(res.taxonomies["tags"], vec!["rust".to_string()]);
    }

    #[test_case(Kind::Page; "page")]
    #[test_case(Kind::Section; "section")]
    fn errors_on_unknown_field(kind: Kind) {
        let res = FrontMatter::parse(&RawFrontMatter::Toml(r#"titel = "typo""#), kind);
        assert!(res.is_err());
    }
}
