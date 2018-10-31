extern crate serde;
extern crate slug;
extern crate tera;
#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate rayon;
extern crate slotmap;
#[macro_use]
extern crate lazy_static;
extern crate regex;

#[cfg(test)]
extern crate globset;
#[cfg(test)]
extern crate tempfile;
#[cfg(test)]
extern crate toml;

extern crate config;
extern crate front_matter;
extern crate rendering;
extern crate utils;
#[macro_use]
extern crate errors;

mod content;
mod library;
mod pagination;
mod sorting;
mod taxonomies;

pub use slotmap::{DenseSlotMap, Key};

pub use content::{Page, Section, SerializingPage, SerializingSection};
pub use library::Library;
pub use pagination::Paginator;
pub use sorting::sort_actual_pages_by_date;
pub use taxonomies::{find_taxonomies, Taxonomy, TaxonomyItem};
