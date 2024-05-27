mod elasticlunr;
mod fuse;

use libs::ammonia;
use libs::once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

pub use elasticlunr::{build_index as build_elasticlunr, ELASTICLUNR_JS};
pub use fuse::build_index as build_fuse;

pub static AMMONIA: Lazy<ammonia::Builder<'static>> = Lazy::new(|| {
    let mut clean_content = HashSet::new();
    clean_content.insert("script");
    clean_content.insert("style");
    clean_content.insert("pre");
    let mut builder = ammonia::Builder::new();
    builder
        .tags(HashSet::new())
        .tag_attributes(HashMap::new())
        .generic_attributes(HashSet::new())
        .link_rel(None)
        .allowed_classes(HashMap::new())
        .clean_content_tags(clean_content);
    builder
});
