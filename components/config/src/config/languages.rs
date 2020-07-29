use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};
use unic_langid::LanguageIdentifier;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Language {
    /// The language code
    pub code: LanguageIdentifier,
    /// Whether to generate a feed for that language, defaults to `false`
    pub feed: bool,
    /// Whether to generate search index for that language, defaults to `false`
    pub search: bool,
}

impl Default for Language {
    fn default() -> Self {
        Language { code: LanguageIdentifier::default(), feed: false, search: false }
    }
}

pub type TranslateTerm = HashMap<String, String>;
