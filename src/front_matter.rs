use std::collections::HashMap;
use std::path::Path;

use toml;
use tera::Value;
use chrono::prelude::*;
use regex::Regex;


use errors::{Result, ResultExt};


lazy_static! {
    static ref PAGE_RE: Regex = Regex::new(r"^\r?\n?\+\+\+\r?\n((?s).*?(?-s))\+\+\+\r?\n?((?s).*(?-s))$").unwrap();
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortBy {
    Date,
    Order,
    None,
}

/// The front matter of every page
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrontMatter {
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
    /// Whether this page is a draft and should be published or not
    pub draft: Option<bool>,
    /// Only one category allowed
    pub category: Option<String>,
    /// Whether to sort by "date", "order" or "none"
    #[serde(skip_serializing)]
    pub sort_by: Option<SortBy>,
    /// Integer to use to order content. Lowest is at the bottom, highest first
    pub order: Option<usize>,
    /// Optional template, if we want to specify which template to render for that page
    #[serde(skip_serializing)]
    pub template: Option<String>,
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

    pub fn sort_by(&self) -> SortBy {
        match self.sort_by {
            Some(ref s) => s.clone(),
            None => SortBy::Date,
        }
    }
}

impl Default for FrontMatter {
    fn default() -> FrontMatter {
        FrontMatter {
            title: "default".to_string(),
            description: " A default front matter".to_string(),
            date: None,
            slug: None,
            url: None,
            tags: None,
            draft: None,
            category: None,
            sort_by: None,
            order: None,
            template: None,
            extra: None,
        }
    }
}

/// Split a file between the front matter and its content
/// It will parse the front matter as well and returns any error encountered
pub fn split_content(file_path: &Path, content: &str) -> Result<(FrontMatter, String)> {
    if !PAGE_RE.is_match(content) {
        bail!("Couldn't find front matter in `{}`. Did you forget to add `+++`?", file_path.to_string_lossy());
    }

    // 2. extract the front matter and the content
    let caps = PAGE_RE.captures(content).unwrap();
    // caps[0] is the full match
    let front_matter = &caps[1];
    let content = &caps[2];

    // 3. create our page, parse front matter and assign all of that
    let meta = FrontMatter::parse(front_matter)
        .chain_err(|| format!("Error when parsing front matter of file `{}`", file_path.to_string_lossy()))?;

    Ok((meta, content.to_string()))
}
