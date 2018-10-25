extern crate tera;
extern crate slug;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate slotmap;
extern crate rayon;
#[macro_use]
extern crate lazy_static;
extern crate regex;

#[cfg(test)]
extern crate tempfile;
#[cfg(test)]
extern crate toml;
#[cfg(test)]
extern crate globset;

extern crate front_matter;
extern crate config;
extern crate utils;
extern crate rendering;
#[macro_use]
extern crate errors;

mod content;
mod taxonomies;
mod pagination;
mod sorting;
mod library;

pub use slotmap::{Key, DenseSlotMap};

pub use sorting::sort_actual_pages_by_date;
pub use content::{Page, SerializingPage, Section, SerializingSection};
pub use library::Library;
pub use taxonomies::{Taxonomy, TaxonomyItem, find_taxonomies};
pub use pagination::Paginator;
