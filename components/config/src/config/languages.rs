use std::collections::HashMap;

use errors::{bail, Result};
use serde_derive::{Deserialize, Serialize};
use unic_langid::LanguageIdentifier;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LanguageOptions {
    /// Title of the site. Defaults to None
    pub title: Option<String>,
    /// Description of the site. Defaults to None
    pub description: Option<String>,
    /// Whether to generate a feed for that language, defaults to `false`
    pub generate_feed: bool,
    /// Whether to generate search index for that language, defaults to `false`
    pub build_search_index: bool,
}

impl Default for LanguageOptions {
    fn default() -> Self {
        LanguageOptions { title: None, description: None, generate_feed: false, build_search_index: false }
    }
}

pub type TranslateTerm = HashMap<String, String>;

/// We want to ensure the language codes are valid ones
pub fn validate_code(code: &str) -> Result<()> {
    if LanguageIdentifier::from_bytes(code.as_bytes()).is_err() {
        bail!("Language `{}` is not a valid Unicode Language Identifier (see http://unicode.org/reports/tr35/#Unicode_language_identifier)", code)
    }

    Ok(())
}
