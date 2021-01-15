mod config;
pub mod highlighting;
mod theme;
pub use crate::config::{
    languages::Language, link_checker::LinkChecker, slugify::Slugify, taxonomies::Taxonomy, Config,
};
use errors::Result;

use std::path::Path;

/// Get and parse the config.
/// If it doesn't succeed, exit
pub fn get_config(filename: &Path) -> Result<Config> {
    Config::from_file(filename)
        .map_err(|e| errors::Error::chain(&format!("Failed to load config from file {}", filename.display()), e))
}
