use std::collections::HashMap;

use tera::Value;
use toml;

use errors::Result;

use super::{SortBy, InsertAnchor};

static DEFAULT_PAGINATE_PATH: &'static str = "page";


/// The front matter of every section
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionFrontMatter {
    /// <title> of the page
    pub title: Option<String>,
    /// Description in <meta> that appears when linked, e.g. on twitter
    pub description: Option<String>,
    /// Whether to sort by "date", "order" or "none". Defaults to `none`.
    #[serde(skip_serializing)]
    pub sort_by: Option<SortBy>,
    /// Optional template, if we want to specify which template to render for that page
    #[serde(skip_serializing)]
    pub template: Option<String>,
    /// How many pages to be displayed per paginated page. No pagination will happen if this isn't set
    #[serde(skip_serializing)]
    pub paginate_by: Option<usize>,
    /// Path to be used by pagination: the page number will be appended after it. Defaults to `page`.
    #[serde(skip_serializing)]
    pub paginate_path: Option<String>,
    /// Whether to insert a link for each header like in Github READMEs. Defaults to false
    /// The default template can be overridden by creating a `anchor-link.html` template and CSS will need to be
    /// written if you turn that on.
    pub insert_anchor: Option<InsertAnchor>,
    /// Whether to render that section or not. Defaults to `true`.
    /// Useful when the section is only there to organize things but is not meant
    /// to be used directly, like a posts section in a personal site
    #[serde(skip_serializing)]
    pub render: Option<bool>,
    /// Any extra parameter present in the front matter
    pub extra: Option<HashMap<String, Value>>,
}

impl SectionFrontMatter {
    pub fn parse(toml: &str) -> Result<SectionFrontMatter> {
        let mut f: SectionFrontMatter = match toml::from_str(toml) {
            Ok(d) => d,
            Err(e) => bail!(e),
        };

        if f.paginate_path.is_none() {
            f.paginate_path = Some(DEFAULT_PAGINATE_PATH.to_string());
        }

        if f.render.is_none() {
            f.render = Some(true);
        }

        if f.sort_by.is_none() {
            f.sort_by = Some(SortBy::None);
        }

        if f.insert_anchor.is_none() {
            f.insert_anchor = Some(InsertAnchor::None);
        }

        Ok(f)
    }

    /// Returns the current sorting method, defaults to `None` (== no sorting)
    pub fn sort_by(&self) -> SortBy {
        self.sort_by.unwrap()
    }

    /// Only applies to section, whether it is paginated or not.
    pub fn is_paginated(&self) -> bool {
        match self.paginate_by {
            Some(v) => v > 0,
            None => false
        }
    }

    pub fn should_render(&self) -> bool {
        self.render.unwrap()
    }
}

impl Default for SectionFrontMatter {
    fn default() -> SectionFrontMatter {
        SectionFrontMatter {
            title: None,
            description: None,
            sort_by: Some(SortBy::None),
            template: None,
            paginate_by: None,
            paginate_path: Some(DEFAULT_PAGINATE_PATH.to_string()),
            render: Some(true),
            insert_anchor: Some(InsertAnchor::None),
            extra: None,
        }
    }
}
