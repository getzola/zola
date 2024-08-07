use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TaxonomyConfig {
    /// The name used in the URL, usually the plural
    pub name: String,
    /// The slug according to the config slugification strategy
    pub slug: String,
    /// If this is set, the list of individual taxonomy term page will be paginated
    /// by this much
    pub paginate_by: Option<usize>,
    pub paginate_path: Option<String>,
    /// Whether the taxonomy will be rendered, defaults to `true`
    pub render: bool,
    /// Whether to generate a feed only for each taxonomy term, defaults to `false`
    pub feed: bool,
}

impl Default for TaxonomyConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            slug: String::new(),
            paginate_by: None,
            paginate_path: None,
            render: true,
            feed: false,
        }
    }
}

impl TaxonomyConfig {
    pub fn is_paginated(&self) -> bool {
        self.paginate_by.is_some_and(|paginate_by| paginate_by > 0)
    }

    pub fn paginate_path(&self) -> &str {
        self.paginate_path.as_deref().unwrap_or("page")
    }
}
