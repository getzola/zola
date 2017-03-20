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
extern crate tera;
extern crate glob;
extern crate syntect;
extern crate slug;
extern crate chrono;
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

pub use site::Site;
pub use config::Config;
pub use front_matter::{FrontMatter, split_content};
pub use page::{Page};
pub use section::{Section};
pub use utils::create_file;
