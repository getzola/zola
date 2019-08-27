#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate globset;
extern crate toml;
#[macro_use]
extern crate lazy_static;
extern crate syntect;

#[macro_use]
extern crate errors;
extern crate utils;

mod config;
pub mod highlighting;
mod theme;
pub use config::{Config, Language, Taxonomy};

use std::path::Path;

/// Get and parse the config.
/// If it doesn't succeed, exit
pub fn get_config(path: &Path, filename: &str) -> Config {
    match Config::from_file(path.join(filename)) {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to load {}", filename);
            println!("Error: {}", e);
            ::std::process::exit(1);
        }
    }
}
