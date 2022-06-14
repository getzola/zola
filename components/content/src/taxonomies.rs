use std::cmp::Ordering;
use std::path::PathBuf;

use serde::Serialize;

use config::{Config, TaxonomyConfig};
use errors::{Context as ErrorContext, Result};
use libs::ahash::AHashMap;
use libs::tera::{Context, Tera};
use utils::slugs::slugify_paths;
use utils::templates::{check_template_fallbacks, render_template};

use crate::library::Library;
use crate::ser::SerializingPage;
use crate::{Page, SortBy};

use crate::sorting::sort_pages;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SerializedTaxonomyTerm<'a> {
    name: &'a str,
    slug: &'a str,
    path: &'a str,
    permalink: &'a str,
    pages: Vec<SerializingPage<'a>>,
}

impl<'a> SerializedTaxonomyTerm<'a> {
    pub fn from_item(item: &'a TaxonomyTerm, library: &'a Library) -> Self {
        let mut pages = vec![];

        for p in &item.pages {
            pages.push(SerializingPage::new(&library.pages[p], Some(library), false));
        }

        SerializedTaxonomyTerm {
            name: &item.name,
            slug: &item.slug,
            path: &item.path,
            permalink: &item.permalink,
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
        taxo_slug: &str,
        taxo_pages: &[&Page],
        config: &Config,
    ) -> Self {
        let item_slug = slugify_paths(name, config.slugify.taxonomies);
        let path = if lang != config.default_language {
            format!("/{}/{}/{}/", lang, taxo_slug, item_slug)
        } else {
            format!("/{}/{}/", taxo_slug, item_slug)
        };
        let permalink = config.make_permalink(&path);

        // Taxonomy are almost always used for blogs so we filter by dates
        // and it's not like we can sort things across sections by anything other
        // than dates
        let (mut pages, ignored_pages) = sort_pages(taxo_pages, SortBy::Date);
        // We still append pages without dates at the end
        pages.extend(ignored_pages);
        TaxonomyTerm { name: name.to_string(), permalink, path, slug: item_slug, pages }
    }

    pub fn serialize<'a>(&'a self, library: &'a Library) -> SerializedTaxonomyTerm<'a> {
        SerializedTaxonomyTerm::from_item(self, library)
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SerializedTaxonomy<'a> {
    kind: &'a TaxonomyConfig,
    lang: &'a str,
    permalink: &'a str,
    items: Vec<SerializedTaxonomyTerm<'a>>,
}

impl<'a> SerializedTaxonomy<'a> {
    pub fn from_taxonomy(taxonomy: &'a Taxonomy, library: &'a Library) -> Self {
        let items: Vec<SerializedTaxonomyTerm> =
            taxonomy.items.iter().map(|i| SerializedTaxonomyTerm::from_item(i, library)).collect();
        SerializedTaxonomy {
            kind: &taxonomy.kind,
            lang: &taxonomy.lang,
            permalink: &taxonomy.permalink,
            items,
        }
    }
}
/// All different taxonomies we have and their content
#[derive(Debug, Clone, PartialEq)]
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
            sorted_items.push(TaxonomyTerm::new(name, tax_found.lang, &slug, &pages, config));
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
        let path = if tax_found.lang != config.default_language {
            format!("/{}/{}/", tax_found.lang, slug)
        } else {
            format!("/{}/", slug)
        };
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

    pub fn render_term(
        &self,
        item: &TaxonomyTerm,
        tera: &Tera,
        config: &Config,
        library: &Library,
    ) -> Result<String> {
        let mut context = Context::new();
        context.insert("config", &config.serialize(&self.lang));
        context.insert("lang", &self.lang);
        context.insert("term", &SerializedTaxonomyTerm::from_item(item, library));
        context.insert("taxonomy", &self.kind);
        context.insert("current_url", &self.permalink);
        context.insert("current_path", &self.path);

        // Check for taxon-specific template, or use generic as fallback.
        let specific_template = format!("{}/single.html", self.kind.name);
        let template = check_template_fallbacks(&specific_template, tera, &config.theme)
            .unwrap_or("taxonomy_single.html");

        render_template(template, tera, context, &config.theme)
            .with_context(|| format!("Failed to render single term {} page.", self.kind.name))
    }

    pub fn render_all_terms(
        &self,
        tera: &Tera,
        config: &Config,
        library: &Library,
    ) -> Result<String> {
        let mut context = Context::new();
        context.insert("config", &config.serialize(&self.lang));
        let terms: Vec<SerializedTaxonomyTerm> =
            self.items.iter().map(|i| SerializedTaxonomyTerm::from_item(i, library)).collect();
        context.insert("terms", &terms);
        context.insert("lang", &self.lang);
        context.insert("taxonomy", &self.kind);
        context.insert("current_url", &self.permalink);
        context.insert("current_path", &self.path);

        // Check for taxon-specific template, or use generic as fallback.
        let specific_template = format!("{}/list.html", self.kind.name);
        let template = check_template_fallbacks(&specific_template, tera, &config.theme)
            .unwrap_or("taxonomy_list.html");

        render_template(template, tera, context, &config.theme)
            .with_context(|| format!("Failed to render a list of {} page.", self.kind.name))
    }

    pub fn to_serialized<'a>(&'a self, library: &'a Library) -> SerializedTaxonomy<'a> {
        SerializedTaxonomy::from_taxonomy(self, library)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Only used while building the taxonomies
#[derive(Debug, PartialEq)]
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
