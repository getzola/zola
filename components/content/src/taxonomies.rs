use std::cmp::Ordering;
use std::path::PathBuf;

use serde::Serialize;

use config::{Config, TaxonomyConfig};
use errors::{bail, Context as ErrorContext, Result};
use libs::ahash::AHashMap;
use libs::tera::{Context, Tera};
use utils::slugs::slugify_paths;
use utils::templates::{check_template_fallbacks, render_template};

use crate::library::Library;
use crate::ser::SerializingPage;
use crate::{Page, SortBy};

use crate::sorting::sort_pages;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SerializedTaxonomyItem<'a> {
    name: &'a str,
    slug: &'a str,
    path: &'a str,
    permalink: &'a str,
    pages: Vec<SerializingPage<'a>>,
}

impl<'a> SerializedTaxonomyItem<'a> {
    pub fn from_item(item: &'a TaxonomyItem, library: &'a Library) -> Self {
        let mut pages = vec![];

        for p in &item.pages {
            pages.push(SerializingPage::new(&library.pages[p], Some(library), false));
        }

        SerializedTaxonomyItem {
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
pub struct TaxonomyItem {
    pub name: String,
    pub slug: String,
    pub path: String,
    pub permalink: String,
    pub pages: Vec<PathBuf>,
}

impl TaxonomyItem {
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
        TaxonomyItem { name: name.to_string(), permalink, path, slug: item_slug, pages }
    }

    pub fn serialize<'a>(&'a self, library: &'a Library) -> SerializedTaxonomyItem<'a> {
        SerializedTaxonomyItem::from_item(self, library)
    }

    pub fn merge(&mut self, other: Self) {
        self.pages.extend(other.pages);
    }
}

impl PartialEq for TaxonomyItem {
    fn eq(&self, other: &Self) -> bool {
        self.permalink == other.permalink
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SerializedTaxonomy<'a> {
    kind: &'a TaxonomyConfig,
    lang: &'a str,
    permalink: &'a str,
    items: Vec<SerializedTaxonomyItem<'a>>,
}

impl<'a> SerializedTaxonomy<'a> {
    pub fn from_taxonomy(taxonomy: &'a Taxonomy, library: &'a Library) -> Self {
        let items: Vec<SerializedTaxonomyItem> =
            taxonomy.items.iter().map(|i| SerializedTaxonomyItem::from_item(i, library)).collect();
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
    pub permalink: String,
    // this vec is sorted by the count of item
    pub items: Vec<TaxonomyItem>,
}

impl Taxonomy {
    fn new(tax_found: TaxonomyFound, config: &Config) -> Self {
        let mut sorted_items = vec![];
        let slug = tax_found.slug;
        for (name, pages) in tax_found.terms {
            sorted_items.push(TaxonomyItem::new(name, tax_found.lang, &slug, &pages, config));
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
            permalink,
            items: sorted_items,
        }
    }

    pub fn render_term(
        &self,
        item: &TaxonomyItem,
        tera: &Tera,
        config: &Config,
        library: &Library,
    ) -> Result<String> {
        let mut context = Context::new();
        context.insert("config", &config.serialize(&self.lang));
        context.insert("lang", &self.lang);
        context.insert("term", &SerializedTaxonomyItem::from_item(item, library));
        context.insert("taxonomy", &self.kind);
        context.insert(
            "current_url",
            &config.make_permalink(&format!("{}/{}", self.kind.name, item.slug)),
        );
        context.insert("current_path", &format!("/{}/{}/", self.kind.name, item.slug));

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
        let terms: Vec<SerializedTaxonomyItem> =
            self.items.iter().map(|i| SerializedTaxonomyItem::from_item(i, library)).collect();
        context.insert("terms", &terms);
        context.insert("lang", &self.lang);
        context.insert("taxonomy", &self.kind);
        context.insert("current_url", &config.make_permalink(&self.kind.name));
        context.insert("current_path", &format!("/{}/", self.kind.name));

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
struct TaxonomyFound<'a> {
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

pub fn find_taxonomies(config: &Config, pages: &AHashMap<PathBuf, Page>) -> Result<Vec<Taxonomy>> {
    // lang -> tax names -> def
    let mut taxonomies_def = AHashMap::new();
    let mut taxonomies_slug = AHashMap::new();

    for (code, options) in &config.languages {
        let mut taxo_lang_def = AHashMap::new();
        for t in &options.taxonomies {
            let slug = slugify_paths(&t.name, config.slugify.taxonomies);
            taxonomies_slug.insert(&t.name, slug.clone());
            taxo_lang_def.insert(slug.clone(), TaxonomyFound::new(slug, code, t));
        }
        taxonomies_def.insert(code, taxo_lang_def);
    }

    for (_, page) in pages {
        for (name, terms) in &page.meta.taxonomies {
            let slug = taxonomies_slug.get(name);
            let mut exists = slug.is_some();
            if let Some(s) = slug {
                if !taxonomies_def[&page.lang].contains_key(s) {
                    exists = false;
                }
            }
            if !exists {
                bail!(
                    "Page `{}` has taxonomy `{}` which is not defined in config.toml",
                    page.file.path.display(),
                    name
                );
            }
            let slug = slug.unwrap();

            let taxonomy_found = taxonomies_def.get_mut(&page.lang).unwrap().get_mut(slug).unwrap();
            for term in terms {
                taxonomy_found.terms.entry(term).or_insert_with(Vec::new).push(page);
            }
        }
    }

    // And now generates the actual taxonomies
    let mut taxonomies = vec![];
    for (_, vals) in taxonomies_def {
        for (_, tax_found) in vals {
            taxonomies.push(Taxonomy::new(tax_found, config));
        }
    }

    Ok(taxonomies)
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::LanguageOptions;
    use std::collections::HashMap;
    use utils::slugs::SlugifyStrategy;

    macro_rules! taxonomies {
        ($config:expr, [$($page:expr),+]) => {{
            let mut pages = AHashMap::new();
            $(
                pages.insert($page.file.path.clone(), $page.clone());
            )+
            find_taxonomies(&$config, &pages).unwrap()
        }};
    }

    fn create_page(path: &str, lang: &str, taxo: Vec<(&str, Vec<&str>)>) -> Page {
        let mut page = Page::default();
        page.file.path = PathBuf::from(path);
        page.lang = lang.to_owned();
        let mut taxonomies = HashMap::new();
        for (name, terms) in taxo {
            taxonomies.insert(name.to_owned(), terms.iter().map(|t| t.to_string()).collect());
        }
        page.meta.taxonomies = taxonomies;
        page
    }

    #[test]
    fn errors_on_unknown_taxonomy() {
        let config = Config::default_for_test();
        let page1 = create_page("unknown/taxo.md", "en", vec![("tags", vec!["rust", "db"])]);
        let mut pages = AHashMap::new();
        pages.insert(page1.file.path.clone(), page1);
        let taxonomies = find_taxonomies(&config, &pages);
        assert!(taxonomies.is_err());
        let err = taxonomies.unwrap_err();
        assert_eq!(
            err.to_string(),
            "Page `unknown/taxo.md` has taxonomy `tags` which is not defined in config.toml"
        );
    }

    #[test]
    fn can_make_taxonomies() {
        let mut config = Config::default_for_test();
        config.languages.get_mut("en").unwrap().taxonomies = vec![
            TaxonomyConfig { name: "categories".to_string(), ..TaxonomyConfig::default() },
            TaxonomyConfig { name: "tags".to_string(), ..TaxonomyConfig::default() },
            TaxonomyConfig { name: "authors".to_string(), ..TaxonomyConfig::default() },
        ];

        let page1 = create_page(
            "a.md",
            "en",
            vec![("tags", vec!["rust", "db"]), ("categories", vec!["tutorials"])],
        );
        let page2 = create_page(
            "b.md",
            "en",
            vec![("tags", vec!["rust", "js"]), ("categories", vec!["others"])],
        );
        let page3 = create_page(
            "c.md",
            "en",
            vec![("tags", vec!["js"]), ("authors", vec!["Vincent Prouillet"])],
        );
        let taxonomies = taxonomies!(config, [page1, page2, page3]);

        let tags = taxonomies.iter().find(|t| t.kind.name == "tags").unwrap();
        assert_eq!(tags.len(), 3);
        assert_eq!(tags.items[0].name, "db");
        assert_eq!(tags.items[0].permalink, "http://a-website.com/tags/db/");
        assert_eq!(tags.items[0].pages.len(), 1);
        assert_eq!(tags.items[1].name, "js");
        assert_eq!(tags.items[1].permalink, "http://a-website.com/tags/js/");
        assert_eq!(tags.items[1].pages.len(), 2);
        assert_eq!(tags.items[2].name, "rust");
        assert_eq!(tags.items[2].permalink, "http://a-website.com/tags/rust/");
        assert_eq!(tags.items[2].pages.len(), 2);

        let categories = taxonomies.iter().find(|t| t.kind.name == "categories").unwrap();
        assert_eq!(categories.items.len(), 2);
        assert_eq!(categories.items[0].name, "others");
        assert_eq!(categories.items[0].permalink, "http://a-website.com/categories/others/");
        assert_eq!(categories.items[0].pages.len(), 1);

        let authors = taxonomies.iter().find(|t| t.kind.name == "authors").unwrap();
        assert_eq!(authors.items.len(), 1);
        assert_eq!(authors.items[0].permalink, "http://a-website.com/authors/vincent-prouillet/");
    }

    #[test]
    fn can_make_multiple_language_taxonomies() {
        let mut config = Config::default_for_test();
        config.slugify.taxonomies = SlugifyStrategy::Safe;
        config.languages.insert("fr".to_owned(), LanguageOptions::default());
        config.languages.get_mut("en").unwrap().taxonomies = vec![
            TaxonomyConfig { name: "categories".to_string(), ..TaxonomyConfig::default() },
            TaxonomyConfig { name: "tags".to_string(), ..TaxonomyConfig::default() },
        ];
        config.languages.get_mut("fr").unwrap().taxonomies = vec![
            TaxonomyConfig { name: "catégories".to_string(), ..TaxonomyConfig::default() },
            TaxonomyConfig { name: "tags".to_string(), ..TaxonomyConfig::default() },
        ];

        let page1 = create_page("a.md", "en", vec![("categories", vec!["rust"])]);
        let page2 = create_page("b.md", "en", vec![("tags", vec!["rust"])]);
        let page3 = create_page("c.md", "fr", vec![("catégories", vec!["rust"])]);
        let taxonomies = taxonomies!(config, [page1, page2, page3]);

        let categories = taxonomies.iter().find(|t| t.kind.name == "categories").unwrap();
        assert_eq!(categories.len(), 1);
        assert_eq!(categories.items[0].permalink, "http://a-website.com/categories/rust/");
        let tags = taxonomies.iter().find(|t| t.kind.name == "tags" && t.lang == "en").unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags.items[0].permalink, "http://a-website.com/tags/rust/");
        let fr_categories = taxonomies.iter().find(|t| t.kind.name == "catégories").unwrap();
        assert_eq!(fr_categories.len(), 1);
        assert_eq!(fr_categories.items[0].permalink, "http://a-website.com/fr/catégories/rust/");
    }

    #[test]
    fn taxonomies_with_unic_are_grouped_with_default_slugify_strategy() {
        let mut config = Config::default_for_test();
        config.languages.get_mut("en").unwrap().taxonomies = vec![
            TaxonomyConfig { name: "test-taxonomy".to_string(), ..TaxonomyConfig::default() },
            TaxonomyConfig { name: "test taxonomy".to_string(), ..TaxonomyConfig::default() },
            TaxonomyConfig { name: "test-taxonomy ".to_string(), ..TaxonomyConfig::default() },
            TaxonomyConfig { name: "Test-Taxonomy ".to_string(), ..TaxonomyConfig::default() },
        ];
        let page1 = create_page("a.md", "en", vec![("test-taxonomy", vec!["Ecole"])]);
        let page2 = create_page("b.md", "en", vec![("test taxonomy", vec!["École"])]);
        let page3 = create_page("c.md", "en", vec![("test-taxonomy ", vec!["ecole"])]);
        let page4 = create_page("d.md", "en", vec![("Test-Taxonomy ", vec!["école"])]);
        let taxonomies = taxonomies!(config, [page1, page2, page3, page4]);
        assert_eq!(taxonomies.len(), 1);

        let tax = &taxonomies[0];
        // under the default slugify strategy all of the provided terms should be the same
        assert_eq!(tax.items.len(), 1);
        let term1 = &tax.items[0];
        assert_eq!(term1.name, "Ecole");
        assert_eq!(term1.slug, "ecole");
        assert_eq!(term1.permalink, "http://a-website.com/test-taxonomy/ecole/");
        assert_eq!(term1.pages.len(), 4);
    }

    #[test]
    fn taxonomies_with_unic_are_not_grouped_with_safe_slugify_strategy() {
        let mut config = Config::default_for_test();
        config.slugify.taxonomies = SlugifyStrategy::Safe;
        config.languages.get_mut("en").unwrap().taxonomies =
            vec![TaxonomyConfig { name: "test".to_string(), ..TaxonomyConfig::default() }];
        let page1 = create_page("a.md", "en", vec![("test", vec!["Ecole"])]);
        let page2 = create_page("b.md", "en", vec![("test", vec!["École"])]);
        let page3 = create_page("c.md", "en", vec![("test", vec!["ecole"])]);
        let page4 = create_page("d.md", "en", vec![("test", vec!["école"])]);
        let taxonomies = taxonomies!(config, [page1, page2, page3, page4]);
        assert_eq!(taxonomies.len(), 1);
        let tax = &taxonomies[0];
        // under the safe slugify strategy all terms should be distinct
        assert_eq!(tax.items.len(), 4);
    }
}
