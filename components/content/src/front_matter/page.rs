use std::collections::BTreeMap;

use serde::Deserialize;
use tera::{Map, Value};
use time::format_description::well_known::Rfc3339;
use time::macros::{format_description, time};
use time::{Date, OffsetDateTime, PrimitiveDateTime};

use super::datetime::from_unknown_datetime;
use errors::{Result, bail};

use super::extra::{default_extra, deserialize_extra, merge_maps};

use crate::front_matter::split::RawFrontMatter;

/// The front matter of every page
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct PageFrontMatter {
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
    pub draft: Option<bool>,
    /// Prevent generation of a folder for current page
    /// Defaults to `true`
    #[serde(skip_serializing)]
    pub render: Option<bool>,
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
    pub authors: Option<Vec<String>>,
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
    pub in_search_index: Option<bool>,
    /// Whether the page is included in feeds (RSS/Atom)
    /// Defaults to `true`
    #[serde(skip_serializing)]
    pub include_in_feeds: Option<bool>,
    /// Whether to hide this page everywhere but still render it
    #[serde(skip_serializing)]
    pub hidden: Option<bool>,
    /// Any extra parameter present in the front matter
    #[serde(default = "default_extra", deserialize_with = "deserialize_extra")]
    pub extra: Value,
}

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

impl PageFrontMatter {
    pub fn parse(raw: &RawFrontMatter) -> Result<PageFrontMatter> {
        let mut f: PageFrontMatter = raw.deserialize()?;

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

    /// Converts the TOML datetime to a time::OffsetDateTime
    /// Also grabs the year/month/day tuple that will be used in serialization
    pub fn date_to_datetime(&mut self) {
        self.datetime = self.date.as_ref().map(|s| s.as_ref()).and_then(parse_datetime);
        self.datetime_tuple = self.datetime.map(|dt| (dt.year(), dt.month().into(), dt.day()));

        self.updated_datetime = self.updated.as_ref().map(|s| s.as_ref()).and_then(parse_datetime);
        self.updated_datetime_tuple =
            self.updated_datetime.map(|dt| (dt.year(), dt.month().into(), dt.day()));
    }

    pub fn weight(&self) -> usize {
        self.weight.unwrap()
    }

    pub fn draft(&self) -> bool {
        self.draft.unwrap_or(false)
    }

    pub fn render(&self) -> bool {
        self.render.unwrap_or(true)
    }

    pub fn in_search_index(&self) -> bool {
        self.in_search_index.unwrap_or(true)
    }

    pub fn include_in_feeds(&self) -> bool {
        self.include_in_feeds.unwrap_or(true)
    }

    pub fn authors(&self) -> &[String] {
        self.authors.as_deref().unwrap_or_default()
    }

    pub fn inherit_from(self, default: PageFrontMatter) -> PageFrontMatter {
        let PageFrontMatter {
            title,
            description,
            updated,
            updated_datetime: _,
            updated_datetime_tuple: _,
            date,
            datetime: _,
            datetime_tuple: _,
            draft,
            render,
            slug,
            path,
            taxonomies,
            weight,
            authors,
            aliases,
            template,
            in_search_index,
            include_in_feeds,
            hidden,
            extra,
        } = self;

        let mut merged_taxonomies = default.taxonomies;
        merged_taxonomies.extend(taxonomies);
        let merged_extra = match (default.extra.as_map(), extra.as_map()) {
            (Some(base), Some(overlay)) => Value::from(merge_maps(base, overlay)),
            _ => extra,
        };

        let mut merged = PageFrontMatter {
            title: title.or(default.title),
            description: description.or(default.description),
            updated: updated.or(default.updated),
            date: date.or(default.date),
            draft: draft.or(default.draft),
            render: render.or(default.render),
            weight: weight.or(default.weight),
            authors: authors.or(default.authors),
            template: template.or(default.template),
            in_search_index: in_search_index.or(default.in_search_index),
            include_in_feeds: include_in_feeds.or(default.include_in_feeds),
            hidden: hidden.or(default.hidden),
            taxonomies: merged_taxonomies,
            extra: merged_extra,
            slug,
            path,
            aliases,
            updated_datetime: None,
            updated_datetime_tuple: None,
            datetime: None,
            datetime_tuple: None,
        };
        merged.date_to_datetime();
        merged
    }
}

impl Default for PageFrontMatter {
    fn default() -> PageFrontMatter {
        PageFrontMatter {
            in_search_index: None,
            include_in_feeds: None,
            title: None,
            description: None,
            updated: None,
            updated_datetime: None,
            updated_datetime_tuple: None,
            date: None,
            datetime: None,
            datetime_tuple: None,
            draft: None,
            render: None,
            slug: None,
            path: None,
            taxonomies: BTreeMap::new(),
            weight: None,
            authors: None,
            aliases: Vec::new(),
            template: None,
            hidden: None,
            extra: Value::from(Map::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::front_matter::page::PageFrontMatter;
    use crate::front_matter::split::RawFrontMatter;
    use tera::Value;
    use tera::value::Key;
    use test_case::test_case;
    use time::macros::datetime;

    #[test_case(&RawFrontMatter::Toml(r#"  "#); "toml")]
    #[test_case(&RawFrontMatter::Toml(r#"  "#); "yaml")]
    fn can_have_empty_front_matter(content: &RawFrontMatter) {
        let res = PageFrontMatter::parse(content);
        println!("{:?}", res);
        assert!(res.is_ok());
    }

    #[test]
    fn include_in_feeds_defaults_to_true() {
        let content = &RawFrontMatter::Toml(r#"title = "Hello""#);
        let res = PageFrontMatter::parse(content).unwrap();
        assert!(res.include_in_feeds());
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
        let res = PageFrontMatter::parse(content).unwrap();
        assert!(!res.include_in_feeds());
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
        let res = PageFrontMatter::parse(content);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res.title.unwrap(), "Hello".to_string());
        assert_eq!(res.description.unwrap(), "hey there".to_string())
    }

    #[test_case(&RawFrontMatter::Toml(r#"title = |\n"#); "toml")]
    #[test_case(&RawFrontMatter::Yaml(r#"title: |\n"#); "yaml")]
    fn errors_with_invalid_front_matter(content: &RawFrontMatter) {
        let res = PageFrontMatter::parse(content);
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
        let res = PageFrontMatter::parse(content);
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
        let res = PageFrontMatter::parse(content);
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
        let res = PageFrontMatter::parse(content).unwrap();
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
        let res = PageFrontMatter::parse(content).unwrap();
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
        let res = PageFrontMatter::parse(content).unwrap();
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
        let res = PageFrontMatter::parse(content).unwrap();
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
        let res = PageFrontMatter::parse(content).unwrap();
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
        let res = PageFrontMatter::parse(content).unwrap();
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
        let res = PageFrontMatter::parse(content).unwrap();
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
        let res = PageFrontMatter::parse(content);
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
        let res = PageFrontMatter::parse(content);
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
        let res = PageFrontMatter::parse(content).unwrap();
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
        let res = PageFrontMatter::parse(content);
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
        let res = PageFrontMatter::parse(content);
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
        let res = PageFrontMatter::parse(content);
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
        let res = PageFrontMatter::parse(content);
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
        let res = PageFrontMatter::parse(content);
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
        let res = PageFrontMatter::parse(content);
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
        let res = PageFrontMatter::parse(content);
        assert!(res.is_ok());
        let res2 = res.unwrap();
        assert_eq!(res2.authors().len(), 2);
        assert_eq!(
            vec!(
                "person1@example.com (Person One)".to_owned(),
                "person2@example.com (Person Two)".to_owned()
            ),
            res2.authors()
        );
    }

    fn toml(content: &str) -> PageFrontMatter {
        PageFrontMatter::parse(&RawFrontMatter::Toml(content)).unwrap()
    }

    #[test]
    fn inherit_missing_keys_and_translated_wins() {
        let default = toml("title = \"Default\"\nweight = 10\n");
        let merged = toml("title = \"Traduit\"\n").inherit_from(default);
        assert_eq!(merged.title.as_deref(), Some("Traduit"));
        assert_eq!(merged.weight, Some(10));
    }

    #[test]
    fn inherit_merges_extra_tables_and_replaces_arrays() {
        let default = toml(
            "[extra]\ncover = \"cover.png\"\nflavor = \"default\"\ntypes = [\"a\", \"b\"]\n\
             [extra.abilities]\n\"Fire walk\" = \"yes\"\n\"Swim\" = \"no\"\n",
        );
        let translated = toml(
            "[extra]\nflavor = \"traduit\"\ntypes = []\n[extra.abilities]\n\"Fire walk\" = \"oui\"\n",
        );
        let merged = translated.inherit_from(default);
        let extra = merged.extra.as_map().unwrap();
        assert_eq!(extra.get(&Key::from("cover")).unwrap(), &Value::from("cover.png"));
        assert_eq!(extra.get(&Key::from("flavor")).unwrap(), &Value::from("traduit"));
        assert!(extra.get(&Key::from("types")).unwrap().as_array().unwrap().is_empty());
        let abilities = extra.get(&Key::from("abilities")).unwrap().as_map().unwrap();
        assert_eq!(abilities.get(&Key::from("Fire walk")).unwrap(), &Value::from("oui"));
        assert_eq!(abilities.get(&Key::from("Swim")).unwrap(), &Value::from("no"));
    }

    #[test]
    fn inherit_skips_slug_path_and_aliases() {
        let default =
            toml("slug = \"default-slug\"\npath = \"/somewhere\"\naliases = [\"/old\"]\n");
        let merged = toml("title = \"x\"\n").inherit_from(default);
        assert_eq!(merged.slug, None);
        assert_eq!(merged.path, None);
        assert!(merged.aliases.is_empty());
    }

    #[test]
    fn inherit_draft_and_date_and_derives_datetime() {
        let default = toml("draft = true\ndate = 2002-10-12\n");
        let merged = toml("title = \"x\"\n").inherit_from(default);
        assert!(merged.draft());
        assert_eq!(merged.datetime.unwrap(), datetime!(2002 - 10 - 12 0:00 UTC));
        assert_eq!(merged.datetime_tuple, Some((2002, 10, 12)));
    }

    #[test]
    fn inherit_explicit_false_wins() {
        let merged = toml("draft = false\n").inherit_from(toml("draft = true\n"));
        assert!(!merged.draft());
    }

    #[test]
    fn inherit_merges_taxonomies_per_key() {
        let default = toml("[taxonomies]\ntags = [\"hello\"]\nrelated = [\"a\"]\n");
        let translated = toml("[taxonomies]\ntags = [\"bonjour\"]\n");
        let merged = translated.inherit_from(default);
        assert_eq!(merged.taxonomies["tags"], vec!["bonjour".to_string()]);
        assert_eq!(merged.taxonomies["related"], vec!["a".to_string()]);
    }

    #[test]
    fn inherit_empty_authors_in_translation_win() {
        let merged = toml("authors = []\n").inherit_from(toml("authors = [\"a\"]\n"));
        assert!(merged.authors().is_empty());
        let merged = toml("title = \"x\"\n").inherit_from(toml("authors = [\"a\"]\n"));
        assert_eq!(merged.authors(), ["a".to_string()]);
    }
}
