mod config;
pub mod highlighting;
mod theme;

use std::path::Path;

pub use crate::config::{
    languages::LanguageOptions, link_checker::LinkChecker, search::Search, slugify::Slugify,
    taxonomies::TaxonomyConfig, Config,
};
use errors::Result;

/// Get and parse the config.
/// If it doesn't succeed, exit
pub fn get_config(filename: &Path) -> Result<Config> {
    Config::from_file(filename)
}
