#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;
extern crate walkdir;
extern crate pulldown_cmark;
extern crate regex;
#[macro_use]
extern crate tera;
extern crate glob;
extern crate syntect;
extern crate slug;
extern crate chrono;
extern crate base64;
extern crate rayon;
#[cfg(test)]
extern crate tempdir;

mod fs;
mod config;
pub mod errors;
mod front_matter;
mod content;
mod site;
mod rendering;
// Filters, Global Fns and default instance of Tera
mod templates;

pub use site::{Site};
pub use config::{Config, get_config};
pub use front_matter::{PageFrontMatter, SectionFrontMatter, InsertAnchor, split_page_content, split_section_content};
pub use content::{Page, Section, SortBy, sort_pages, populate_previous_and_next_pages};
pub use fs::{create_file};
