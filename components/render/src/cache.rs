use std::path::PathBuf;

use ahash::AHashMap;
use tera::{Tera, Value};

use config::Config;
use content::{
    Library, SerializedTaxonomy, SerializedTaxonomyTerm, SerializingPage, SerializingSection,
    Taxonomy,
};

/// Cached page/section: serialized value + canonical path for translation lookups
#[derive(Debug, Clone)]
pub struct CachedContent {
    pub value: Value,
    pub canonical: PathBuf,
}

/// Cached taxonomy data: serialized value, terms, and resolved templates
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone, Default)]
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
    pub fn new(config: &Config) -> Self {
        let mut c = Self::default();
        let configs = config
            .languages
            .keys()
            .map(|lang| (lang.clone(), Value::from_serializable(&config.serialize(lang))))
            .collect();
        c.configs = configs;
        c
    }

    pub fn build(&mut self, library: &Library, taxonomies: &[Taxonomy], tera: &Tera) {
        // First pass: serialize all pages without siblings
        let (mut pages, pages_by_canonical) = library.pages.iter().fold(
            (
                AHashMap::with_capacity(library.pages.len()),
                AHashMap::<PathBuf, AHashMap<String, PathBuf>>::new(),
            ),
            |(mut pages, mut pages_by_canonical), (path, page)| {
                pages.insert(
                    path.clone(),
                    CachedContent {
                        value: Value::from_serializable(&SerializingPage::new(page, library)),
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

        // Second pass: inject sibling Values from cache
        // Collect siblings first to avoid borrow issues
        let siblings: Vec<_> = library
            .pages
            .iter()
            .filter_map(|(path, page)| {
                let lower = page.lower.as_ref().and_then(|p| pages.get(p)).map(|c| c.value.clone());
                let higher =
                    page.higher.as_ref().and_then(|p| pages.get(p)).map(|c| c.value.clone());
                if lower.is_some() || higher.is_some() {
                    Some((path.clone(), lower, higher))
                } else {
                    None
                }
            })
            .collect();

        for (path, lower, higher) in siblings {
            if let Some(mut cached) = pages.remove(&path) {
                let new_value = match cached.value.into_map() {
                    Some(mut map) => {
                        if let Some(lower_val) = lower {
                            map.insert("lower".into(), lower_val);
                        }
                        if let Some(higher_val) = higher {
                            map.insert("higher".into(), higher_val);
                        }
                        Value::from(map)
                    }
                    None => unreachable!("serialized page should always be a map"),
                };
                cached.value = new_value;
                pages.insert(path, cached);
            }
        }

        let (sections, sections_by_canonical) = library.sections.iter().fold(
            (
                AHashMap::with_capacity(library.sections.len()),
                AHashMap::<PathBuf, AHashMap<String, PathBuf>>::new(),
            ),
            |(mut sections, mut sections_by_canonical), (path, section)| {
                // Look up cached page values
                let section_pages: Vec<Value> = section
                    .pages
                    .iter()
                    .filter_map(|p| pages.get(p).map(|c| c.value.clone()))
                    .collect();

                sections.insert(
                    path.clone(),
                    CachedContent {
                        value: Value::from_serializable(&SerializingSection::new(
                            section,
                            library,
                            section_pages,
                        )),
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

        let cached_taxonomies: Vec<(_, _, _)> = taxonomies
            .iter()
            .map(|t| {
                // Check for custom templates (None = use default, avoids allocation)
                let single_tpl = format!("{}/single.html", t.kind.name);
                let list_tpl = format!("{}/list.html", t.kind.name);
                let single_template =
                    tera.get_template(&single_tpl).is_some().then_some(single_tpl);
                let list_template = tera.get_template(&list_tpl).is_some().then_some(list_tpl);

                // Serialize all terms using cached page values
                let mut terms = AHashMap::with_capacity(t.items.len());
                let mut serialized_terms = Vec::with_capacity(t.items.len());
                for term in &t.items {
                    // Look up pre-cached page values
                    let term_pages: Vec<Value> = term
                        .pages
                        .iter()
                        .filter_map(|p| pages.get(p).map(|c| c.value.clone()))
                        .collect();

                    let serialized_term =
                        SerializedTaxonomyTerm::from_item_with_pages(term, term_pages);
                    terms.insert(term.slug.clone(), Value::from_serializable(&serialized_term));
                    serialized_terms.push(serialized_term);
                }

                let cached = CachedTaxonomy {
                    value: Value::from_serializable(&SerializedTaxonomy::from_taxonomy_with_terms(
                        t,
                        serialized_terms,
                    )),
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

        self.pages = pages;
        self.sections = sections;
        self.pages_by_canonical = pages_by_canonical;
        self.sections_by_canonical = sections_by_canonical;
        self.taxonomies = taxonomies;
    }

    pub fn get_taxonomy(&self, lang: &str, slug: &str) -> Option<&CachedTaxonomy> {
        self.taxonomies.get(lang)?.get(slug)
    }
}
