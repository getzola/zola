use std::path::PathBuf;

use ahash::AHashMap;
use serde::Serialize;
use tera::Value;

use config::{Config, TaxonomyConfig};
use errors::{Result, bail};
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
    ) -> Result<Self> {
        let slug = slugify_paths(name, config.slugify.taxonomies);
        if slug.is_empty() {
            bail!(
                "The term `{name}` in the taxonomy `{}` slugifies to an empty string. You need to rename the term.",
                taxo.name
            )
        }
        let path = config.get_taxonomy_term_path(lang, taxo, &slug);
        let permalink = config.make_permalink(&path);

        // Taxonomy are almost always used for blogs so we filter by dates
        // and it's not like we can sort things across sections by anything other
        // than dates
        let (mut pages, mut ignored_pages) = sort_pages(taxo_pages, SortBy::Date);
        // Sorting for determinism
        ignored_pages.sort();
        pages.extend(ignored_pages);
        Ok(TaxonomyTerm { name: name.to_string(), permalink, path, slug, pages })
    }
}

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
    pub(crate) fn new(tax_found: TaxonomyFound, config: &Config) -> Result<Self> {
        let slug = tax_found.slug;
        let mut by_slug: AHashMap<String, (&str, Vec<&Page>)> = AHashMap::new();
        for (name, pages) in tax_found.terms {
            let slug = slugify_paths(name, config.slugify.taxonomies);
            let (canonical_name, all_pages) = by_slug.entry(slug).or_insert((name, Vec::new()));
            // We just need to pick a deterministic choice, so we pick the lesser one
            if name < *canonical_name {
                *canonical_name = name;
            }
            all_pages.extend(pages);
        }

        let mut sorted_items = by_slug
            .into_values()
            .map(|(name, pages)| {
                TaxonomyTerm::new(name, tax_found.lang, tax_found.config, &pages, config)
            })
            .collect::<Result<Vec<_>>>()?;

        sorted_items.sort_by(|a, b| a.slug.cmp(&b.slug));

        let path = config.get_taxonomy_path(tax_found.lang, tax_found.config);
        let permalink = config.make_permalink(&path);

        Ok(Taxonomy {
            slug,
            lang: tax_found.lang.to_owned(),
            kind: tax_found.config.clone(),
            path,
            permalink,
            items: sorted_items,
        })
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
    use std::path::PathBuf;

    use config::{Config, TaxonomyConfig};

    use crate::{Page, PageFrontMatter, Taxonomy, TaxonomyTerm};

    use super::TaxonomyFound;

    #[test]
    fn taxonomy_path_with_taxonomy_root() {
        let mut conf = Config::default_for_test();
        conf.taxonomy_root = Some("blog".to_string());
        let mut tax_conf = TaxonomyConfig::default();
        tax_conf.slug = "tags".to_string();
        let tax_found = TaxonomyFound::new("tags".into(), &conf.default_language, &tax_conf);
        let tax = Taxonomy::new(tax_found, &conf).unwrap();
        let pages = &[];
        let term =
            TaxonomyTerm::new("rust", &conf.default_language, &tax_conf, pages, &conf).unwrap();

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
        let tax = Taxonomy::new(tax_found, &conf).unwrap();
        let pages = &[];
        let term =
            TaxonomyTerm::new("rust", &conf.default_language, &tax_conf, pages, &conf).unwrap();

        // Verify taxonomy list path
        assert_eq!(tax.path, "/tags/");
        assert_eq!(tax.permalink, format!("{}/tags/", conf.base_url));

        // Verify taxonomy term path
        assert_eq!(term.path, "/tags/rust/");
        assert_eq!(term.permalink, format!("{}/tags/rust/", conf.base_url));
    }

    // https://github.com/getzola/zola/issues/2494
    #[test]
    fn merges_terms_with_different_case() {
        let conf = Config::default_for_test();
        let mut tax_conf = TaxonomyConfig::default();
        tax_conf.slug = "games".to_string();

        fn create_page(name: &str, date: &str) -> Page {
            let mut front_matter =
                PageFrontMatter { date: Some(date.to_string()), ..Default::default() };
            front_matter.date_to_datetime();
            Page::new(format!("content/{name}.md"), front_matter, &PathBuf::new())
        }
        let page1 = create_page("a", "2026-08-01");
        let page2 = create_page("b", "2026-08-03");
        let page3 = create_page("c", "2026-08-02");

        let mut tax_found = TaxonomyFound::new("games".into(), &conf.default_language, &tax_conf);
        // different capitalization of legends
        tax_found.terms.insert("League of legends", vec![&page1, &page2]);
        tax_found.terms.insert("League of Legends", vec![&page3]);

        let tax = Taxonomy::new(tax_found, &conf).unwrap();

        // Only one item in the end, with the 3 pages
        assert_eq!(tax.items.len(), 1);
        assert_eq!(tax.items[0].slug, "league-of-legends");
        assert_eq!(tax.items[0].pages.len(), 3);
        assert_eq!(
            tax.items[0].pages,
            vec![page2.file.path.clone(), page3.file.path.clone(), page1.file.path.clone()]
        );
    }

    // https://github.com/getzola/zola/issues/2338
    #[test]
    fn taxonomy_slug_is_empty_errors() {
        let conf = Config::default_for_test();
        let mut tax_conf = TaxonomyConfig::default();
        tax_conf.name = "tags".to_string();
        let pages = &[];
        let res = TaxonomyTerm::new(";", &conf.default_language, &tax_conf, pages, &conf);
        println!("{res:?}");
        assert!(res.is_err());
    }
}
