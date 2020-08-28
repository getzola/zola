use std::collections::HashMap;
use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};
use unic_langid::LanguageIdentifier;

use crate::config::languages::*;
use errors::Result;
use utils::fs::read_file_with_error;

/// Holds the data from a `theme.toml` file.
///
/// There are other fields, but Zola only cares about `[extra]` and `[languages]`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Theme {
    /// The primary/default language of the theme
    ///
    /// MUST be a valid language, specified using the [BCP 47] syntax. To retain backwards
    /// compatibility with other language naming systems, the [language_alias] option was added.
    ///
    /// If not specified, [default_language_options] will be merged into
    /// [config.default_language_options] regardless of [config.default_language].
    pub default_language: Option<LanguageIdentifier>,
    /// Theme-wide defaults for locale-specific options
    ///
    /// All values except those in `[extra]` are ignored.
    ///
    /// This is flattened when serializing/deserializing, i.e. its fields will appear as if they
    /// were at the top level alongside base_url. These values are used for the default language,
    /// and other languages will fall back to these values if they are not overridden in
    /// `languages`
    #[serde(flatten, default = "LocaleOptions::default_for_default_lang")]
    pub default_language_options: LocaleOptions,

    /// Language-specific options for translations of this site
    ///
    /// All values except those in `[extra]` are ignored.
    ///
    /// The key of the TOML table (represented as a HashMap in Rust) is a valid language code, and
    /// its values can be used for overriding some of the site-wide settings.
    ///
    /// See [LocaleOptions] for what options can be specified here and what their default values
    /// are.
    ///
    /// Having an entry for the default language is treated as an error.
    pub languages: HashMap<LanguageIdentifier, LocaleOptions>,
}

impl Theme {
    /// Parses a TOML string to our Theme struct
    pub fn parse(content: &str) -> Result<Theme> {
        toml::from_str::<Theme>(content).map_err(|e| e.into())
    }

    /// Parses a theme file from the given path
    pub fn from_file(path: &PathBuf) -> Result<Theme> {
        let content = read_file_with_error(
            path,
            "No `theme.toml` file found. \
             Is the `theme` defined in your `config.toml` present in the `themes` directory \
             and does it have a `theme.toml` inside?",
        )?;
        Theme::parse(&content)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            default_language: None,
            default_language_options: LocaleOptions::default_for_default_lang(),
            languages: HashMap::new(),
        }
    }
}
