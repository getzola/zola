use std::path::PathBuf;

use ahash::AHashMap;
use tera::{Tera, Value};

use config::Config;
use content::{Library, Taxonomy};

/// Cached taxonomy data: serialized value, terms, and resolved templates
#[derive(Debug)]
pub struct CachedTaxonomy {
    /// Serialized taxonomy (for list page, e.g., /tags/)
    pub value: Value,
    /// Serialized terms: term_slug -> Value
    pub terms: AHashMap<String, Value>,
    /// Resolved template for single term page (e.g., tags/single.html)
    pub single_template: Option<String>,
    /// Resolved template for list page (e.g., tags/list.html)
    pub list_template: Option<String>,
}

/// All pre-serialized data for template rendering.
#[derive(Debug, Default)]
pub struct RenderCache {
    pub pages: AHashMap<PathBuf, Value>,
    pub sections: AHashMap<PathBuf, Value>,
    /// Serialized config per language
    pub configs: AHashMap<String, Value>,
    /// Cached taxonomies: lang -> (taxonomy_slug -> CachedTaxonomy)
    pub taxonomies: AHashMap<String, AHashMap<String, CachedTaxonomy>>,
}

impl RenderCache {
    pub fn build(config: &Config, library: &Library, taxonomies: &[Taxonomy], tera: &Tera) -> Self {
        let pages = library
            .pages
            .iter()
            .map(|(k, v)| (k.clone(), Value::from_serializable(&v.serialize(library))))
            .collect();

        let sections = library
            .sections
            .iter()
            .map(|(k, v)| (k.clone(), Value::from_serializable(&v.serialize(library))))
            .collect();

        let configs = config
            .languages
            .keys()
            .map(|lang| (lang.clone(), Value::from_serializable(&config.serialize(lang))))
            .collect();

        let cached_taxonomies: Vec<(_, _, _)> = taxonomies
            .iter()
            .map(|t| {
                // Check for custom templates (None = use default, avoids allocation)
                let single_tpl = format!("{}/single.html", t.kind.name);
                let list_tpl = format!("{}/list.html", t.kind.name);
                let single_template =
                    tera.get_template(&single_tpl).is_some().then_some(single_tpl);
                let list_template = tera.get_template(&list_tpl).is_some().then_some(list_tpl);

                // Serialize all terms
                let mut terms = AHashMap::with_capacity(t.items.len());
                for term in &t.items {
                    terms.insert(
                        term.slug.clone(),
                        Value::from_serializable(&term.serialize(library)),
                    );
                }

                let cached = CachedTaxonomy {
                    value: Value::from_serializable(&t.to_serialized(library)),
                    terms,
                    single_template,
                    list_template,
                };

                (t.lang.clone(), t.slug.clone(), cached)
            })
            .collect();

        let taxonomies = cached_taxonomies.into_iter().fold(
            AHashMap::<String, AHashMap<String, CachedTaxonomy>>::new(),
            |mut acc, (lang, slug, cached)| {
                acc.entry(lang).or_default().insert(slug, cached);
                acc
            },
        );

        Self { pages, sections, configs, taxonomies }
    }

    pub fn get_taxonomy(&self, lang: &str, slug: &str) -> Option<&CachedTaxonomy> {
        self.taxonomies.get(lang)?.get(slug)
    }
}
