use std::collections::HashMap;

use errors::{bail, Result};
use libs::unic_langid::LanguageIdentifier;
use serde::{Deserialize, Serialize};

use crate::config::search;
use crate::config::taxonomies;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct LanguageOptions {
    /// Title of the site. Defaults to None
    pub title: Option<String>,
    /// Description of the site. Defaults to None
    pub description: Option<String>,
    /// Whether to generate a feed for that language, defaults to `false`
    pub generate_feed: bool,
    /// The filename to use for feeds. Used to find the template, too.
    /// Defaults to "atom.xml", with "rss.xml" also having a template provided out of the box.
    pub feed_filename: String,
    pub taxonomies: Vec<taxonomies::TaxonomyConfig>,
    /// Whether to generate search index for that language, defaults to `false`
    pub build_search_index: bool,
    /// The search config, telling what to include in the search index for that language
    pub search: search::Search,
    /// A toml crate `Table` with String key representing term and value
    /// another `String` representing its translation.
    /// Use `get_translation()` method for translating key into different languages.
    pub translations: HashMap<String, String>,
}

impl LanguageOptions {
    /// Merges self with another LanguageOptions, panicking if 2 equivalent fields are not None,
    /// empty or of default value.
    pub fn merge(&mut self, other: &LanguageOptions) -> Result<()> {
        match &self.title {
            None => self.title = other.title.clone(),
            Some(self_title) => match &other.title {
                Some(other_title) => bail!(
                    "`title` for default language is specified twice, as {:?} and {:?}.",
                    self_title,
                    other_title
                ),
                None => (),
            },
        };

        match &self.description {
            None => self.description = other.description.clone(),
            Some(self_description) => match &other.description {
                Some(other_description) => bail!(
                    "`description` for default language is specified twice, as {:?} and {:?}.",
                    self_description,
                    other_description
                ),
                None => (),
            },
        };

        self.generate_feed = self.generate_feed || other.generate_feed;

        match &self.feed_filename == "atom.xml" {
            // "atom.xml" is default value.
            true => self.feed_filename = other.feed_filename.clone(),
            false => match &other.feed_filename.is_empty() {
                false => bail!(
                    "`feed filename` for default language is specifiec twice, as {:?} and {:?}.",
                    self.feed_filename,
                    other.feed_filename
                ),
                true => (),
            },
        };

        match &self.taxonomies.is_empty() {
            true => self.taxonomies = other.taxonomies.clone(),
            false => match &other.taxonomies.is_empty() {
                false => bail!(
                    "`taxonomies` for default language is specifiec twice, as {:?} and {:?}.",
                    self.taxonomies,
                    other.taxonomies
                ),
                true => (),
            },
        };

        self.build_search_index = self.build_search_index || other.build_search_index;

        match self.search == search::Search::default() {
            true => self.search = other.search.clone(),
            false => match self.search == other.search {
                false => bail!(
                    "`search` for default language is specified twice, as {:?} and {:?}.",
                    self.search,
                    other.search
                ),
                true => (),
            },
        };

        match &self.translations.is_empty() {
            true => self.translations = other.translations.clone(),
            false => match &other.translations.is_empty() {
                false => bail!(
                    "`translations` for default language is specified twice, as {:?} and {:?}.",
                    self.translations,
                    other.translations
                ),
                true => (),
            },
        };

        Ok(())
    }
}

/// We want to ensure the language codes are valid ones
pub fn validate_code(code: &str) -> Result<()> {
    if LanguageIdentifier::from_bytes(code.as_bytes()).is_err() {
        bail!("Language `{}` is not a valid Unicode Language Identifier (see http://unicode.org/reports/tr35/#Unicode_language_identifier)", code)
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_without_conflict() {
        let mut base_default_language_options = LanguageOptions {
            title: Some("Site's title".to_string()),
            description: None,
            generate_feed: true,
            feed_filename: "atom.xml".to_string(),
            taxonomies: vec![],
            build_search_index: true,
            search: search::Search::default(),
            translations: HashMap::new(),
        };

        let section_default_language_options = LanguageOptions {
            title: None,
            description: Some("Site's description".to_string()),
            generate_feed: false,
            feed_filename: "rss.xml".to_string(),
            taxonomies: vec![],
            build_search_index: true,
            search: search::Search::default(),
            translations: HashMap::new(),
        };

        base_default_language_options.merge(&section_default_language_options).unwrap();
    }

    #[test]
    #[should_panic]
    fn merge_with_conflict() {
        let mut base_default_language_options = LanguageOptions {
            title: Some("Site's title".to_string()),
            description: Some("Duplicate site description".to_string()),
            generate_feed: true,
            feed_filename: "".to_string(),
            taxonomies: vec![],
            build_search_index: true,
            search: search::Search::default(),
            translations: HashMap::new(),
        };

        let section_default_language_options = LanguageOptions {
            title: None,
            description: Some("Site's description".to_string()),
            generate_feed: false,
            feed_filename: "Some feed_filename".to_string(),
            taxonomies: vec![],
            build_search_index: true,
            search: search::Search::default(),
            translations: HashMap::new(),
        };

        base_default_language_options
            .merge(&section_default_language_options)
            .expect("This should lead to panic");
    }
}
