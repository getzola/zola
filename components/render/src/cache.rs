use std::path::PathBuf;

use ahash::AHashMap;
use tera::{Tera, Value};

use config::Config;
use content::{Library, Taxonomy};

/// Cached page/section: serialized value + canonical path for translation lookups
#[derive(Debug, Clone)]
pub struct CachedContent {
    pub value: Value,
    pub canonical: PathBuf,
}

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
    pub pages: AHashMap<PathBuf, CachedContent>,
    pub sections: AHashMap<PathBuf, CachedContent>,
    /// canonical path -> (lang -> file path)
    pub pages_by_canonical: AHashMap<PathBuf, AHashMap<String, PathBuf>>,
    /// canonical path -> (lang -> file path)
    pub sections_by_canonical: AHashMap<PathBuf, AHashMap<String, PathBuf>>,
    /// Serialized config per language
    pub configs: AHashMap<String, Value>,
    /// Cached taxonomies: lang -> (taxonomy_slug -> CachedTaxonomy)
    pub taxonomies: AHashMap<String, AHashMap<String, CachedTaxonomy>>,
}

impl RenderCache {
    pub fn build(config: &Config, library: &Library, taxonomies: &[Taxonomy], tera: &Tera) -> Self {
        let (pages, pages_by_canonical) = library.pages.iter().fold(
            (
                AHashMap::with_capacity(library.pages.len()),
                AHashMap::<PathBuf, AHashMap<String, PathBuf>>::new(),
            ),
            |(mut pages, mut pages_by_canonical), (path, page)| {
                pages.insert(
                    path.clone(),
                    CachedContent {
                        value: Value::from_serializable(&page.serialize(library)),
                        canonical: page.file.canonical.clone(),
                    },
                );
                pages_by_canonical
                    .entry(page.file.canonical.clone())
                    .or_default()
                    .insert(page.lang.clone(), path.clone());
                (pages, pages_by_canonical)
            },
        );

        let (sections, sections_by_canonical) = library.sections.iter().fold(
            (
                AHashMap::with_capacity(library.sections.len()),
                AHashMap::<PathBuf, AHashMap<String, PathBuf>>::new(),
            ),
            |(mut sections, mut sections_by_canonical), (path, section)| {
                sections.insert(
                    path.clone(),
                    CachedContent {
                        value: Value::from_serializable(&section.serialize(library)),
                        canonical: section.file.canonical.clone(),
                    },
                );
                sections_by_canonical
                    .entry(section.file.canonical.clone())
                    .or_default()
                    .insert(section.lang.clone(), path.clone());
                (sections, sections_by_canonical)
            },
        );

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

        Self { pages, sections, pages_by_canonical, sections_by_canonical, configs, taxonomies }
    }

    pub fn get_taxonomy(&self, lang: &str, slug: &str) -> Option<&CachedTaxonomy> {
        self.taxonomies.get(lang)?.get(slug)
    }
}
