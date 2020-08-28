mod config;
pub mod highlighting;
mod theme;

pub use crate::config::{
    languages::LocaleOptions, link_checker::LinkChecker, search::Search, slugify::Slugify,
    taxonomies::Taxonomy, Config, LocalizedConfig,
};
pub use crate::theme::Theme;

use std::path::Path;

/// Get and parse the config.
/// If it doesn't succeed, exit
pub fn get_config(filename: &Path) -> Config {
    match Config::from_file(filename) {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to load {}", filename.display());
            println!("Error: {}", e);
            ::std::process::exit(1);
        }
    }
}
