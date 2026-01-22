use std::path::PathBuf;

use ahash::AHashMap;
use tera::Value;

use config::Config;
use content::{Library, Taxonomy, TaxonomyTerm};

/// All pre-serialized data for template rendering.
#[derive(Debug)]
pub struct RenderCache {
    pub pages: AHashMap<PathBuf, Value>,
    pub sections: AHashMap<PathBuf, Value>,
    /// Serialized config per language
    pub configs: AHashMap<String, Value>,
    /// Serialized taxonomy lists (the /tags/ page)
    /// Key: (lang, taxonomy_slug) e.g. ("en", "tags")
    pub taxonomies: AHashMap<(String, String), Value>,
    /// Serialized taxonomy terms (the /tags/rust/ page)
    /// Key: (lang, taxonomy_slug, term_slug) e.g. ("en", "tags", "rust")
    pub taxonomy_terms: AHashMap<(String, String, String), Value>,
}

impl RenderCache {
    pub fn build(
        config: &Config,
        library: &Library,
        taxonomies: &[Taxonomy],
    ) -> Self {
        let mut cache = Self {
            pages: AHashMap::with_capacity(library.pages.len()),
            sections: AHashMap::with_capacity(library.sections.len()),
            configs: AHashMap::with_capacity(config.languages.len() + 1),
            taxonomies: AHashMap::with_capacity(taxonomies.len()),
            taxonomy_terms: AHashMap::with_capacity(taxonomies.len() * 3),
        };

        for (path, page) in &library.pages {
            cache.pages.insert(
                path.clone(),
                Value::from_serializable(&page.serialize(library)),
            );
        }
        for (path, section) in &library.sections {
            cache.sections.insert(
                path.clone(),
                Value::from_serializable(&section.serialize(library)),
            );
        }


        todo!()
    }
}