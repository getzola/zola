mod config;
pub mod highlighting;
mod theme;

pub use crate::config::{
    languages::LanguageOptions, link_checker::LinkChecker, search::Search, slugify::Slugify,
    taxonomies::Taxonomy, Config,
};
use errors::Result;

use std::path::Path;

/// Get and parse the config.
/// If it doesn't succeed, exit
pub fn get_config(filename: &Path) -> Result<Config> {
    Config::from_file(filename)
}
