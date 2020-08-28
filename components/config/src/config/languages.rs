use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};
use toml::Value as Toml;

/// Contains site settings that can be set differently for each language
///
/// When rendering pages, this will be merged with the options for the default language, the
/// details of which are documented for each field separately.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LocaleOptions {
    /// How to refer to the language in URLs
    ///
    /// The localization functions require a valid language identifier to work correctly. Some
    /// multilingual sites might already be using language names that do not conform to the
    /// canonical [BCP 47] syntax, and this lets us keep links to them working currently.
    ///
    /// If empty or not set, the language code will be used.
    ///
    /// [BCP 47]: https://tools.ietf.org/html/bcp47
    pub language_alias: String,

    /// Title of the site. Defaults to none
    ///
    /// This will not fall back to how it's set for the default language.
    pub title: Option<String>,
    /// Description of the site. Defaults to None.
    ///
    /// This will not fall back to how it's set for the default language.
    pub description: Option<String>,

    /// Taxonomies available for a language
    ///
    /// This will not fall back to how it's set for the default language.
    pub taxonomies: Vec<super::taxonomies::Taxonomy>,

    /// Whether to generate a feed for a language
    ///
    /// If unset in a translation, it will fall back to how it's set for the default language.
    /// Its default value for the primary language is `false`.
    pub generate_feed: Option<bool>,
    /// The number of articles to include in the feed for a language
    ///
    /// If unset in a translation, it will fall back to how it's set for the default language.
    /// Its default value for the primary language is all.
    ///
    /// All has been represented as None since the beginning, hence the awkward Option<Option<_>>.
    pub feed_limit: Option<Option<usize>>,

    /// Whether to generate search index for a language
    ///
    /// If unset in a translation, it will fall back to how it's set for the default language.
    /// Its default value for the default language is `false`.
    pub build_search_index: Option<bool>,
    /// The search config, telling what to include in the search index for a language
    ///
    /// If unset in a translation, it will fall back to how it's set for the default language.
    /// If unset for the default language, it will be set to [Search::default].
    pub search: Option<super::search::Search>,

    /// All user parameters set in `[extra]`
    ///
    /// This supersedes `[translations]` from previous versions. Serde ensures that this is a Toml
    /// table by attempting to convert it into a `HashMap<String, toml::Value>`. This would,
    /// however, make us do `.get().unwrap().get().unwrap()` for retrieveing a nested value; but we
    /// treat this as opaque data, and do not attempt to modify it.
    ///
    /// For localizations, this will be recursively merged with that of the default language.
    pub extra: HashMap<String, Toml>,
}

impl LocaleOptions {
    /// Sets the default values when set for the default language.
    ///
    /// This had to be done here, because serde does not yet support `#[serde(default)]` for newtype
    /// structs.
    pub fn default_for_default_lang() -> Self {
        LocaleOptions {
            language_alias: String::new(),
            title: None,
            description: None,
            taxonomies: Vec::new(),
            generate_feed: Some(false),
            feed_limit: Some(None),
            build_search_index: Some(false),
            search: Some(super::search::Search::default()),
            extra: HashMap::new(),
        }
    }

    pub fn default_for_additional_lang() -> Self {
        LocaleOptions::default()
    }
}
