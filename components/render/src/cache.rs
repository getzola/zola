use std::path::PathBuf;

use ahash::AHashMap;
use tera::{Tera, Value};

use config::Config;
use content::{
    Library, Page, SerializedTaxonomy, SerializedTaxonomyTerm, SerializingPage, SerializingSection,
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
    /// canonical path -> (lang -> file path)
    pub pages_by_canonical: AHashMap<PathBuf, AHashMap<String, PathBuf>>,
    /// Serialized config per language
    pub configs: AHashMap<String, Value>,
    /// Cached taxonomies: lang -> (taxonomy_slug -> CachedTaxonomy)
    pub taxonomies: AHashMap<String, AHashMap<String, CachedTaxonomy>>,
    /// Aliases for taxos: lang -> (taxonomy name -> taxonomy slug)
    pub taxo_name_to_slug: AHashMap<String, AHashMap<String, String>>,
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
        let mut cached_pages = AHashMap::with_capacity(library.pages.len());
        let mut pages_by_canonical: AHashMap<PathBuf, AHashMap<String, PathBuf>> = AHashMap::new();

        // First pass: serialize all pages without siblings
        for (path, page) in library.pages.iter().filter(|(_, x)| x.is_page()) {
            cached_pages.insert(
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
        }
        inject_siblings(&mut cached_pages, library.pages.values().filter(|p| p.is_page()));

        for (path, page) in library.pages.iter().filter(|(_, x)| x.is_section()) {
            // Look up cached page values
            let pages: Vec<Value> = page
                .pages
                .iter()
                .filter_map(|p| cached_pages.get(p).map(|c| c.value.clone()))
                .collect();

            cached_pages.insert(
                path.clone(),
                CachedContent {
                    value: Value::from_serializable(&SerializingSection::new(page, library, pages)),
                    canonical: page.file.canonical.clone(),
                },
            );
            pages_by_canonical
                .entry(page.file.canonical.clone())
                .or_default()
                .insert(page.lang.clone(), path.clone());
        }
        inject_siblings(&mut cached_pages, library.pages.values().filter(|p| p.is_section()));

        let mut taxonomies_slug: AHashMap<_, AHashMap<_, _>> = AHashMap::new();
        let cached_taxonomies: Vec<(_, _, _, _)> = taxonomies
            .iter()
            .map(|t| {
                // Check for custom templates (None = use default, avoids allocation)
                let single_tpl = format!("{}/single.html", t.kind.name);
                let list_tpl = format!("{}/list.html", t.kind.name);
                let single_template = tera.contains_template(&single_tpl).then_some(single_tpl);
                let list_template = tera.contains_template(&list_tpl).then_some(list_tpl);

                // Serialize all terms using cached page values
                let mut terms = AHashMap::with_capacity(t.items.len());
                let mut serialized_terms = Vec::with_capacity(t.items.len());
                for term in &t.items {
                    // Look up pre-cached page values
                    let term_pages: Vec<Value> = term
                        .pages
                        .iter()
                        .filter_map(|p| cached_pages.get(p).map(|c| c.value.clone()))
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

                (t.lang.clone(), t.kind.name.clone(), t.slug.clone(), cached)
            })
            .collect();

        let taxonomies = cached_taxonomies.into_iter().fold(
            AHashMap::<String, AHashMap<String, CachedTaxonomy>>::new(),
            |mut acc, (lang, name, slug, cached)| {
                if slug != name {
                    taxonomies_slug
                        .entry(lang.clone())
                        .or_default()
                        .insert(name.clone(), slug.clone());
                }
                acc.entry(lang).or_default().insert(slug, cached);
                acc
            },
        );

        self.pages = cached_pages;
        self.pages_by_canonical = pages_by_canonical;
        self.taxonomies = taxonomies;
        self.taxo_name_to_slug = taxonomies_slug;
    }

    pub fn get_taxonomy(&self, lang: &str, kind: &str) -> Option<&CachedTaxonomy> {
        let taxonomies = self.taxonomies.get(lang)?;
        if let Some(terms) = taxonomies.get(kind) {
            return Some(terms);
        }
        taxonomies.get(self.taxo_name_to_slug.get(lang)?.get(kind)?)
    }
}

fn inject_siblings<'a>(
    cache: &mut AHashMap<PathBuf, CachedContent>,
    items: impl Iterator<Item = &'a Page>,
) {
    let siblings: Vec<_> = items
        .filter_map(|p| {
            let lower = p.lower.as_ref().and_then(|x| cache.get(x)).map(|c| c.value.clone());
            let higher = p.higher.as_ref().and_then(|x| cache.get(x)).map(|c| c.value.clone());
            if lower.is_some() || higher.is_some() {
                Some((p.file.path.clone(), lower, higher))
            } else {
                None
            }
        })
        .collect();

    for (path, lower, higher) in siblings {
        if let Some(mut cached) = cache.remove(&path) {
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
            cache.insert(path, cached);
        }
    }
}
