use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Taxonomy {
    /// The name used in the URL, usually the plural
    pub name: String,
    /// If this is set, the list of individual taxonomy term page will be paginated
    /// by this much
    pub paginate_by: Option<usize>,
    pub paginate_path: Option<String>,
    /// Whether to generate a feed only for each taxonomy term, defaults to false
    pub feed: bool,
    /// The language for that taxonomy, only used in multilingual sites.
    /// Defaults to the config `default_language` if not set
    pub lang: String,
}

impl Taxonomy {
    pub fn is_paginated(&self) -> bool {
        if let Some(paginate_by) = self.paginate_by {
            paginate_by > 0
        } else {
            false
        }
    }

    pub fn paginate_path(&self) -> &str {
        if let Some(ref path) = self.paginate_path {
            path
        } else {
            "page"
        }
    }
}
