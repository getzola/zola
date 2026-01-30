use std::cmp::Ordering;
use std::path::PathBuf;

use serde::Serialize;
use tera::Value;

use ahash::AHashMap;
use config::{Config, TaxonomyConfig};
use utils::slugs::slugify_paths;

use crate::{Page, SortBy};

use crate::sorting::sort_pages;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SerializedTaxonomyTerm<'a> {
    name: &'a str,
    slug: &'a str,
    path: &'a str,
    permalink: &'a str,
    pages: Vec<Value>,
    page_count: usize,
}

impl<'a> SerializedTaxonomyTerm<'a> {
    /// Build from pre-cached page Values (used by RenderCache)
    pub fn from_item_with_pages(item: &'a TaxonomyTerm, pages: Vec<Value>) -> Self {
        SerializedTaxonomyTerm {
            name: &item.name,
            slug: &item.slug,
            path: &item.path,
            permalink: &item.permalink,
            page_count: item.pages.len(),
            pages,
        }
    }
}

/// A taxonomy with all its pages
#[derive(Debug, Clone)]
pub struct TaxonomyTerm {
    pub name: String,
    pub slug: String,
    pub path: String,
    pub permalink: String,
    pub pages: Vec<PathBuf>,
}

impl TaxonomyTerm {
    pub fn new(
        name: &str,
        lang: &str,
        taxo: &TaxonomyConfig,
        taxo_pages: &[&Page],
        config: &Config,
    ) -> Self {
        let slug = slugify_paths(name, config.slugify.taxonomies);
        let path = config.get_taxonomy_term_path(lang, &taxo, &slug);
        let permalink = config.make_permalink(&path);

        // Taxonomy are almost always used for blogs so we filter by dates
        // and it's not like we can sort things across sections by anything other
        // than dates
        let (mut pages, ignored_pages) = sort_pages(taxo_pages, SortBy::Date);
        // We still append pages without dates at the end
        pages.extend(ignored_pages);
        TaxonomyTerm { name: name.to_string(), permalink, path, slug, pages }
    }

    pub fn merge(&mut self, other: Self) {
        self.pages.extend(other.pages);
    }
}

impl PartialEq for TaxonomyTerm {
    fn eq(&self, other: &Self) -> bool {
        self.permalink == other.permalink
    }
}

impl Eq for TaxonomyTerm {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SerializedTaxonomy<'a> {
    kind: &'a TaxonomyConfig,
    lang: &'a str,
    permalink: &'a str,
    items: Vec<SerializedTaxonomyTerm<'a>>,
}

impl<'a> SerializedTaxonomy<'a> {
    /// Build from pre-built terms (used by RenderCache)
    pub fn from_taxonomy_with_terms(
        taxonomy: &'a Taxonomy,
        terms: Vec<SerializedTaxonomyTerm<'a>>,
    ) -> Self {
        SerializedTaxonomy {
            kind: &taxonomy.kind,
            lang: &taxonomy.lang,
            permalink: &taxonomy.permalink,
            items: terms,
        }
    }
}
/// All different taxonomies we have and their content
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Taxonomy {
    pub kind: TaxonomyConfig,
    pub lang: String,
    pub slug: String,
    pub path: String,
    pub permalink: String,
    // this vec is sorted by the count of item
    pub items: Vec<TaxonomyTerm>,
}

impl Taxonomy {
    pub(crate) fn new(tax_found: TaxonomyFound, config: &Config) -> Self {
        let mut sorted_items = vec![];
        let slug = tax_found.slug;
        for (name, pages) in tax_found.terms {
            sorted_items.push(TaxonomyTerm::new(
                name,
                tax_found.lang,
                &tax_found.config,
                &pages,
                config,
            ));
        }

        sorted_items.sort_by(|a, b| match a.slug.cmp(&b.slug) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => a.name.cmp(&b.name),
        });
        sorted_items.dedup_by(|a, b| {
            // custom Eq impl checks for equal permalinks
            // here we make sure all pages from a get copied to b
            // before dedup gets rid of it
            if a == b {
                b.merge(a.to_owned());
                true
            } else {
                false
            }
        });
        let path = config.get_taxonomy_path(tax_found.lang, tax_found.config);
        let permalink = config.make_permalink(&path);

        Taxonomy {
            slug,
            lang: tax_found.lang.to_owned(),
            kind: tax_found.config.clone(),
            path,
            permalink,
            items: sorted_items,
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Only used while building the taxonomies
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TaxonomyFound<'a> {
    pub lang: &'a str,
    pub slug: String,
    pub config: &'a TaxonomyConfig,
    pub terms: AHashMap<&'a str, Vec<&'a Page>>,
}

impl<'a> TaxonomyFound<'a> {
    pub fn new(slug: String, lang: &'a str, config: &'a TaxonomyConfig) -> Self {
        Self { slug, lang, config, terms: AHashMap::new() }
    }
}

#[cfg(test)]
mod tests {
    use config::{Config, TaxonomyConfig};

    use crate::{Taxonomy, TaxonomyTerm};

    use super::TaxonomyFound;

    #[test]
    fn taxonomy_path_with_taxonomy_root() {
        let mut conf = Config::default_for_test();
        conf.taxonomy_root = Some("blog".to_string());
        let mut tax_conf = TaxonomyConfig::default();
        tax_conf.slug = "tags".to_string();
        let tax_found = TaxonomyFound::new("tags".into(), &conf.default_language, &tax_conf);
        let tax = Taxonomy::new(tax_found, &conf);
        let pages = &[];
        let term = TaxonomyTerm::new("rust", &conf.default_language, &tax_conf, pages, &conf);

        // Verify taxonomy list path
        assert_eq!(tax.path, "/blog/tags/");
        assert_eq!(tax.permalink, format!("{}/blog/tags/", conf.base_url));

        // Verify taxonomy term path
        assert_eq!(term.path, "/blog/tags/rust/");
        assert_eq!(term.permalink, format!("{}/blog/tags/rust/", conf.base_url));
    }

    #[test]
    fn taxonomy_path_without_taxonomy_root() {
        let conf = Config::default_for_test();
        let mut tax_conf = TaxonomyConfig::default();
        tax_conf.slug = "tags".to_string();
        let tax_found = TaxonomyFound::new("tags".into(), &conf.default_language, &tax_conf);
        let tax = Taxonomy::new(tax_found, &conf);
        let pages = &[];
        let term = TaxonomyTerm::new("rust", &conf.default_language, &tax_conf, pages, &conf);

        // Verify taxonomy list path
        assert_eq!(tax.path, "/tags/");
        assert_eq!(tax.permalink, format!("{}/tags/", conf.base_url));

        // Verify taxonomy term path
        assert_eq!(term.path, "/tags/rust/");
        assert_eq!(term.permalink, format!("{}/tags/rust/", conf.base_url));
    }
}
