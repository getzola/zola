use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Language {
    /// The language code
    pub code: String,
    /// Whether to generate a feed for that language, defaults to `false`
    pub feed: bool,
    /// Whether to generate search index for that language, defaults to `false`
    pub search: bool,
}

pub type TranslateTerm = HashMap<String, String>;
