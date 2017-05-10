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
#[cfg(test)]
extern crate tempdir;

mod utils;
mod config;
pub mod errors;
mod page;
mod front_matter;
mod site;
mod markdown;
mod section;
mod pagination;
/// Additional filters for Tera
mod filters;
/// Global fns for Tera
mod global_fns;

pub use site::{Site, GUTENBERG_TERA};
pub use config::{Config, get_config};
pub use front_matter::{FrontMatter, split_content, SortBy};
pub use page::{Page, populate_previous_and_next_pages};
pub use section::{Section};
pub use utils::{create_file};
pub use markdown::markdown_to_html;
