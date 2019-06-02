use std::collections::HashMap;

use tera::Value;
use toml;

use errors::Result;

use super::{InsertAnchor, SortBy};

static DEFAULT_PAGINATE_PATH: &'static str = "page";

/// The front matter of every section
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SectionFrontMatter {
    /// <title> of the page
    pub title: Option<String>,
    /// Description in <meta> that appears when linked, e.g. on twitter
    pub description: Option<String>,
    /// Whether to sort by "date", "order", "weight" or "none". Defaults to `none`.
    #[serde(skip_serializing)]
    pub sort_by: SortBy,
    /// Used by the parent section to order its subsections.
    /// Higher values means it will be at the end. Defaults to `0`
    #[serde(skip_serializing)]
    pub weight: usize,
    /// Optional template, if we want to specify which template to render for that section
    #[serde(skip_serializing)]
    pub template: Option<String>,
    /// How many pages to be displayed per paginated page. No pagination will happen if this isn't set
    #[serde(skip_serializing)]
    pub paginate_by: Option<usize>,
    /// Path to be used by pagination: the page number will be appended after it. Defaults to `page`.
    #[serde(skip_serializing)]
    pub paginate_path: String,
    /// Whether to insert a link for each header like the ones you can see in this site if you hover one
    /// The default template can be overridden by creating a `anchor-link.html` in the `templates` directory
    pub insert_anchor_links: InsertAnchor,
    /// Whether to render that section or not. Defaults to `true`.
    /// Useful when the section is only there to organize things but is not meant
    /// to be used directly, like a posts section in a personal site
    #[serde(skip_serializing)]
    pub render: bool,
    /// Whether to redirect when landing on that section. Defaults to `None`.
    /// Useful for the same reason as `render` but when you don't want a 404 when
    /// landing on the root section page
    #[serde(skip_serializing)]
    pub redirect_to: Option<String>,
    /// Whether the section content and its pages/subsections are included in the index.
    /// Defaults to `true` but is only used if search if explicitly enabled in the config.
    #[serde(skip_serializing)]
    pub in_search_index: bool,
    /// Whether the section should pass its pages on to the parent section. Defaults to `false`.
    /// Useful when the section shouldn't split up the parent section, like
    /// sections for each year under a posts section.
    #[serde(skip_serializing)]
    pub transparent: bool,
    /// Optional template for all pages in this section (including the pages of children section)
    #[serde(skip_serializing)]
    pub page_template: Option<String>,
    /// All aliases for that page. Zola will create HTML templates that will
    /// redirect to this
    #[serde(skip_serializing)]
    pub aliases: Vec<String>,
    /// Any extra parameter present in the front matter
    pub extra: HashMap<String, Value>,
}

impl SectionFrontMatter {
    pub fn parse(toml: &str) -> Result<SectionFrontMatter> {
        let f: SectionFrontMatter = match toml::from_str(toml) {
            Ok(d) => d,
            Err(e) => bail!(e),
        };

        Ok(f)
    }

    /// Only applies to section, whether it is paginated or not.
    pub fn is_paginated(&self) -> bool {
        match self.paginate_by {
            Some(v) => v > 0,
            None => false,
        }
    }
}

impl Default for SectionFrontMatter {
    fn default() -> SectionFrontMatter {
        SectionFrontMatter {
            title: None,
            description: None,
            sort_by: SortBy::None,
            weight: 0,
            template: None,
            paginate_by: None,
            paginate_path: DEFAULT_PAGINATE_PATH.to_string(),
            render: true,
            redirect_to: None,
            insert_anchor_links: InsertAnchor::None,
            in_search_index: true,
            transparent: false,
            page_template: None,
            aliases: Vec::new(),
            extra: HashMap::new(),
        }
    }
}
