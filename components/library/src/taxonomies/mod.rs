use std::cmp::Ordering;
use std::collections::HashMap;

use serde_derive::Serialize;
use slotmap::DefaultKey;
use tera::{Context, Tera};

use config::{Config, Taxonomy as TaxonomyConfig};
use errors::{bail, Error, Result};
use utils::templates::render_template;

use crate::content::SerializingPage;
use crate::library::Library;
use crate::sorting::sort_pages_by_date;
use utils::slugs::slugify_paths;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SerializedTaxonomyItem<'a> {
    name: &'a str,
    slug: &'a str,
    permalink: &'a str,
    pages: Vec<SerializingPage<'a>>,
}

impl<'a> SerializedTaxonomyItem<'a> {
    pub fn from_item(item: &'a TaxonomyItem, library: &'a Library) -> Self {
        let mut pages = vec![];

        for key in &item.pages {
            let page = library.get_page_by_key(*key);
            pages.push(page.to_serialized_basic(library));
        }

        SerializedTaxonomyItem {
            name: &item.name,
            slug: &item.slug,
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
    pub permalink: String,
    pub pages: Vec<DefaultKey>,
}

impl TaxonomyItem {
    pub fn new(
        name: &str,
        taxonomy: &TaxonomyConfig,
        taxo_slug: &str,
        config: &Config,
        keys: Vec<DefaultKey>,
        library: &Library,
    ) -> Self {
        // Taxonomy are almost always used for blogs so we filter by dates
        // and it's not like we can sort things across sections by anything other
        // than dates
        let data = keys
            .iter()
            .map(|k| {
                if let Some(page) = library.pages().get(*k) {
                    (k, page.meta.datetime, page.permalink.as_ref())
                } else {
                    unreachable!("Sorting got an unknown page")
                }
            })
            .collect();
        let (mut pages, ignored_pages) = sort_pages_by_date(data);
        let item_slug = slugify_paths(name, config.slugify.taxonomies);
        let permalink = if taxonomy.lang != config.default_language {
            config.make_permalink(&format!("/{}/{}/{}", taxonomy.lang, taxo_slug, item_slug))
        } else {
            config.make_permalink(&format!("/{}/{}", taxo_slug, item_slug))
        };

        // We still append pages without dates at the end
        pages.extend(ignored_pages);

        TaxonomyItem { name: name.to_string(), permalink, slug: item_slug, pages }
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
    items: Vec<SerializedTaxonomyItem<'a>>,
}

impl<'a> SerializedTaxonomy<'a> {
    pub fn from_taxonomy(taxonomy: &'a Taxonomy, library: &'a Library) -> Self {
        let items: Vec<SerializedTaxonomyItem> =
            taxonomy.items.iter().map(|i| SerializedTaxonomyItem::from_item(i, library)).collect();
        SerializedTaxonomy { kind: &taxonomy.kind, items }
    }
}

/// All different taxonomies we have and their content
#[derive(Debug, Clone, PartialEq)]
pub struct Taxonomy {
    pub kind: TaxonomyConfig,
    pub slug: String,
    // this vec is sorted by the count of item
    pub items: Vec<TaxonomyItem>,
}

impl Taxonomy {
    fn new(
        kind: TaxonomyConfig,
        config: &Config,
        items: HashMap<String, Vec<DefaultKey>>,
        library: &Library,
    ) -> Taxonomy {
        let mut sorted_items = vec![];
        let slug = slugify_paths(&kind.name, config.slugify.taxonomies);
        for (name, pages) in items {
            sorted_items.push(TaxonomyItem::new(&name, &kind, &slug, config, pages, library));
        }
        //sorted_items.sort_by(|a, b| a.name.cmp(&b.name));
        sorted_items.sort_by(|a, b| match a.slug.cmp(&b.slug) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => a.name.cmp(&b.name),
        });
        sorted_items.dedup_by(|a, b| {
            // custom Eq impl checks for equal permalinks
            // here we make sure all pages from a get coppied to b
            // before dedup gets rid of it
            if a == b {
                b.merge(a.to_owned());
                true
            } else {
                false
            }
        });
        Taxonomy { kind, slug, items: sorted_items }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn render_term(
        &self,
        item: &TaxonomyItem,
        tera: &Tera,
        config: &Config,
        library: &Library,
    ) -> Result<String> {
        let mut context = Context::new();
        context.insert("config", config);
        context.insert("lang", &self.kind.lang);
        context.insert("term", &SerializedTaxonomyItem::from_item(item, library));
        context.insert("taxonomy", &self.kind);
        context.insert(
            "current_url",
            &config.make_permalink(&format!("{}/{}", self.kind.name, item.slug)),
        );
        context.insert("current_path", &format!("/{}/{}/", self.kind.name, item.slug));

        render_template(&format!("{}/single.html", self.kind.name), tera, context, &config.theme)
            .map_err(|e| {
                Error::chain(format!("Failed to render single term {} page.", self.kind.name), e)
            })
    }

    pub fn render_all_terms(
        &self,
        tera: &Tera,
        config: &Config,
        library: &Library,
    ) -> Result<String> {
        let mut context = Context::new();
        context.insert("config", config);
        let terms: Vec<SerializedTaxonomyItem> =
            self.items.iter().map(|i| SerializedTaxonomyItem::from_item(i, library)).collect();
        context.insert("terms", &terms);
        context.insert("lang", &self.kind.lang);
        context.insert("taxonomy", &self.kind);
        context.insert("current_url", &config.make_permalink(&self.kind.name));
        context.insert("current_path", &format!("/{}/", self.kind.name));

        render_template(&format!("{}/list.html", self.kind.name), tera, context, &config.theme)
            .map_err(|e| {
                Error::chain(format!("Failed to render a list of {} page.", self.kind.name), e)
            })
    }

    pub fn to_serialized<'a>(&'a self, library: &'a Library) -> SerializedTaxonomy<'a> {
        SerializedTaxonomy::from_taxonomy(self, library)
    }
}

pub fn find_taxonomies(config: &Config, library: &Library) -> Result<Vec<Taxonomy>> {
    let taxonomies_def = {
        let mut m = HashMap::new();
        for t in &config.taxonomies {
            let slug = slugify_paths(&t.name, config.slugify.taxonomies);
            m.insert(format!("{}-{}", slug, t.lang), t);
        }
        m
    };

    let mut all_taxonomies = HashMap::new();
    for (key, page) in library.pages() {
        for (name, taxo_term) in &page.meta.taxonomies {
            let taxo_slug = slugify_paths(&name, config.slugify.taxonomies);
            let taxo_key = format!("{}-{}", &taxo_slug, page.lang);
            if taxonomies_def.contains_key(&taxo_key) {
                all_taxonomies.entry(taxo_key.clone()).or_insert_with(HashMap::new);

                for term in taxo_term {
                    all_taxonomies
                        .get_mut(&taxo_key)
                        .unwrap()
                        .entry(term.to_string())
                        .or_insert_with(Vec::new)
                        .push(key);
                }
            } else {
                bail!(
                    "Page `{}` has taxonomy `{}` which is not defined in config.toml",
                    page.file.path.display(),
                    name
                );
            }
        }
    }

    let mut taxonomies = vec![];

    for (name, taxo) in all_taxonomies {
        taxonomies.push(Taxonomy::new(taxonomies_def[&name].clone(), config, taxo, library));
    }

    Ok(taxonomies)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::content::Page;
    use crate::library::Library;
    use config::{Config, Language, Slugify, Taxonomy as TaxonomyConfig};
    use utils::slugs::SlugifyStrategy;

    #[test]
    fn can_make_taxonomies() {
        let mut config = Config::default();
        let mut library = Library::new(2, 0, false);

        config.taxonomies = vec![
            TaxonomyConfig {
                name: "categories".to_string(),
                lang: config.default_language.clone(),
                ..TaxonomyConfig::default()
            },
            TaxonomyConfig {
                name: "tags".to_string(),
                lang: config.default_language.clone(),
                ..TaxonomyConfig::default()
            },
            TaxonomyConfig {
                name: "authors".to_string(),
                lang: config.default_language.clone(),
                ..TaxonomyConfig::default()
            },
        ];

        let mut page1 = Page::default();
        let mut taxo_page1 = HashMap::new();
        taxo_page1.insert("tags".to_string(), vec!["rust".to_string(), "db".to_string()]);
        taxo_page1.insert("categories".to_string(), vec!["Programming tutorials".to_string()]);
        page1.meta.taxonomies = taxo_page1;
        page1.lang = config.default_language.clone();
        library.insert_page(page1);

        let mut page2 = Page::default();
        let mut taxo_page2 = HashMap::new();
        taxo_page2.insert("tags".to_string(), vec!["rust".to_string(), "js".to_string()]);
        taxo_page2.insert("categories".to_string(), vec!["Other".to_string()]);
        page2.meta.taxonomies = taxo_page2;
        page2.lang = config.default_language.clone();
        library.insert_page(page2);

        let mut page3 = Page::default();
        let mut taxo_page3 = HashMap::new();
        taxo_page3.insert("tags".to_string(), vec!["js".to_string()]);
        taxo_page3.insert("authors".to_string(), vec!["Vincent Prouillet".to_string()]);
        page3.meta.taxonomies = taxo_page3;
        page3.lang = config.default_language.clone();
        library.insert_page(page3);

        let taxonomies = find_taxonomies(&config, &library).unwrap();
        let (tags, categories, authors) = {
            let mut t = None;
            let mut c = None;
            let mut a = None;
            for x in taxonomies {
                match x.kind.name.as_ref() {
                    "tags" => t = Some(x),
                    "categories" => c = Some(x),
                    "authors" => a = Some(x),
                    _ => unreachable!(),
                }
            }
            (t.unwrap(), c.unwrap(), a.unwrap())
        };
        assert_eq!(tags.items.len(), 3);
        assert_eq!(categories.items.len(), 2);
        assert_eq!(authors.items.len(), 1);

        assert_eq!(tags.items[0].name, "db");
        assert_eq!(tags.items[0].slug, "db");
        assert_eq!(tags.items[0].permalink, "http://a-website.com/tags/db/");
        assert_eq!(tags.items[0].pages.len(), 1);

        assert_eq!(tags.items[1].name, "js");
        assert_eq!(tags.items[1].slug, "js");
        assert_eq!(tags.items[1].permalink, "http://a-website.com/tags/js/");
        assert_eq!(tags.items[1].pages.len(), 2);

        assert_eq!(tags.items[2].name, "rust");
        assert_eq!(tags.items[2].slug, "rust");
        assert_eq!(tags.items[2].permalink, "http://a-website.com/tags/rust/");
        assert_eq!(tags.items[2].pages.len(), 2);

        assert_eq!(categories.items[0].name, "Other");
        assert_eq!(categories.items[0].slug, "other");
        assert_eq!(categories.items[0].permalink, "http://a-website.com/categories/other/");
        assert_eq!(categories.items[0].pages.len(), 1);

        assert_eq!(categories.items[1].name, "Programming tutorials");
        assert_eq!(categories.items[1].slug, "programming-tutorials");
        assert_eq!(
            categories.items[1].permalink,
            "http://a-website.com/categories/programming-tutorials/"
        );
        assert_eq!(categories.items[1].pages.len(), 1);
    }

    #[test]
    fn can_make_slugified_taxonomies() {
        let mut config = Config::default();
        let mut library = Library::new(2, 0, false);

        config.taxonomies = vec![
            TaxonomyConfig {
                name: "categories".to_string(),
                lang: config.default_language.clone(),
                ..TaxonomyConfig::default()
            },
            TaxonomyConfig {
                name: "tags".to_string(),
                lang: config.default_language.clone(),
                ..TaxonomyConfig::default()
            },
            TaxonomyConfig {
                name: "authors".to_string(),
                lang: config.default_language.clone(),
                ..TaxonomyConfig::default()
            },
        ];

        let mut page1 = Page::default();
        let mut taxo_page1 = HashMap::new();
        taxo_page1.insert("tags".to_string(), vec!["rust".to_string(), "db".to_string()]);
        taxo_page1.insert("categories".to_string(), vec!["Programming tutorials".to_string()]);
        page1.meta.taxonomies = taxo_page1;
        page1.lang = config.default_language.clone();
        library.insert_page(page1);

        let mut page2 = Page::default();
        let mut taxo_page2 = HashMap::new();
        taxo_page2.insert("tags".to_string(), vec!["rust".to_string(), "js".to_string()]);
        taxo_page2.insert("categories".to_string(), vec!["Other".to_string()]);
        page2.meta.taxonomies = taxo_page2;
        page2.lang = config.default_language.clone();
        library.insert_page(page2);

        let mut page3 = Page::default();
        let mut taxo_page3 = HashMap::new();
        taxo_page3.insert("tags".to_string(), vec!["js".to_string()]);
        taxo_page3.insert("authors".to_string(), vec!["Vincent Prouillet".to_string()]);
        page3.meta.taxonomies = taxo_page3;
        page3.lang = config.default_language.clone();
        library.insert_page(page3);

        let taxonomies = find_taxonomies(&config, &library).unwrap();
        let (tags, categories, authors) = {
            let mut t = None;
            let mut c = None;
            let mut a = None;
            for x in taxonomies {
                match x.kind.name.as_ref() {
                    "tags" => t = Some(x),
                    "categories" => c = Some(x),
                    "authors" => a = Some(x),
                    _ => unreachable!(),
                }
            }
            (t.unwrap(), c.unwrap(), a.unwrap())
        };
        assert_eq!(tags.items.len(), 3);
        assert_eq!(categories.items.len(), 2);
        assert_eq!(authors.items.len(), 1);

        assert_eq!(tags.items[0].name, "db");
        assert_eq!(tags.items[0].slug, "db");
        assert_eq!(tags.items[0].permalink, "http://a-website.com/tags/db/");
        assert_eq!(tags.items[0].pages.len(), 1);

        assert_eq!(tags.items[1].name, "js");
        assert_eq!(tags.items[1].slug, "js");
        assert_eq!(tags.items[1].permalink, "http://a-website.com/tags/js/");
        assert_eq!(tags.items[1].pages.len(), 2);

        assert_eq!(tags.items[2].name, "rust");
        assert_eq!(tags.items[2].slug, "rust");
        assert_eq!(tags.items[2].permalink, "http://a-website.com/tags/rust/");
        assert_eq!(tags.items[2].pages.len(), 2);

        assert_eq!(categories.items[0].name, "Other");
        assert_eq!(categories.items[0].slug, "other");
        assert_eq!(categories.items[0].permalink, "http://a-website.com/categories/other/");
        assert_eq!(categories.items[0].pages.len(), 1);

        assert_eq!(categories.items[1].name, "Programming tutorials");
        assert_eq!(categories.items[1].slug, "programming-tutorials");
        assert_eq!(
            categories.items[1].permalink,
            "http://a-website.com/categories/programming-tutorials/"
        );
        assert_eq!(categories.items[1].pages.len(), 1);
    }

    #[test]
    fn errors_on_unknown_taxonomy() {
        let mut config = Config::default();
        let mut library = Library::new(2, 0, false);

        config.taxonomies = vec![TaxonomyConfig {
            name: "authors".to_string(),
            lang: config.default_language.clone(),
            ..TaxonomyConfig::default()
        }];
        let mut page1 = Page::default();
        let mut taxo_page1 = HashMap::new();
        taxo_page1.insert("tags".to_string(), vec!["rust".to_string(), "db".to_string()]);
        page1.meta.taxonomies = taxo_page1;
        page1.lang = config.default_language.clone();
        library.insert_page(page1);

        let taxonomies = find_taxonomies(&config, &library);
        assert!(taxonomies.is_err());
        let err = taxonomies.unwrap_err();
        // no path as this is created by Default
        assert_eq!(
            format!("{}", err),
            "Page `` has taxonomy `tags` which is not defined in config.toml"
        );
    }

    #[test]
    fn can_make_taxonomies_in_multiple_languages() {
        let mut config = Config::default();
        config.languages.push(Language { feed: false, code: "fr".to_string(), search: false });
        let mut library = Library::new(2, 0, true);

        config.taxonomies = vec![
            TaxonomyConfig {
                name: "categories".to_string(),
                lang: config.default_language.clone(),
                ..TaxonomyConfig::default()
            },
            TaxonomyConfig {
                name: "tags".to_string(),
                lang: config.default_language.clone(),
                ..TaxonomyConfig::default()
            },
            TaxonomyConfig {
                name: "auteurs".to_string(),
                lang: "fr".to_string(),
                ..TaxonomyConfig::default()
            },
            TaxonomyConfig {
                name: "tags".to_string(),
                lang: "fr".to_string(),
                ..TaxonomyConfig::default()
            },
        ];

        let mut page1 = Page::default();
        let mut taxo_page1 = HashMap::new();
        taxo_page1.insert("tags".to_string(), vec!["rust".to_string(), "db".to_string()]);
        taxo_page1.insert("categories".to_string(), vec!["Programming tutorials".to_string()]);
        page1.meta.taxonomies = taxo_page1;
        page1.lang = config.default_language.clone();
        library.insert_page(page1);

        let mut page2 = Page::default();
        let mut taxo_page2 = HashMap::new();
        taxo_page2.insert("tags".to_string(), vec!["rust".to_string()]);
        taxo_page2.insert("categories".to_string(), vec!["Other".to_string()]);
        page2.meta.taxonomies = taxo_page2;
        page2.lang = config.default_language.clone();
        library.insert_page(page2);

        let mut page3 = Page::default();
        page3.lang = "fr".to_string();
        let mut taxo_page3 = HashMap::new();
        taxo_page3.insert("tags".to_string(), vec!["rust".to_string()]);
        taxo_page3.insert("auteurs".to_string(), vec!["Vincent Prouillet".to_string()]);
        page3.meta.taxonomies = taxo_page3;
        library.insert_page(page3);

        let taxonomies = find_taxonomies(&config, &library).unwrap();
        let (tags, categories, authors) = {
            let mut t = None;
            let mut c = None;
            let mut a = None;
            for x in taxonomies {
                match x.kind.name.as_ref() {
                    "tags" => {
                        if x.kind.lang == "en" {
                            t = Some(x)
                        }
                    }
                    "categories" => c = Some(x),
                    "auteurs" => a = Some(x),
                    _ => unreachable!(),
                }
            }
            (t.unwrap(), c.unwrap(), a.unwrap())
        };

        assert_eq!(tags.items.len(), 2);
        assert_eq!(categories.items.len(), 2);
        assert_eq!(authors.items.len(), 1);

        assert_eq!(tags.items[0].name, "db");
        assert_eq!(tags.items[0].slug, "db");
        assert_eq!(tags.items[0].permalink, "http://a-website.com/tags/db/");
        assert_eq!(tags.items[0].pages.len(), 1);

        assert_eq!(tags.items[1].name, "rust");
        assert_eq!(tags.items[1].slug, "rust");
        assert_eq!(tags.items[1].permalink, "http://a-website.com/tags/rust/");
        assert_eq!(tags.items[1].pages.len(), 2);

        assert_eq!(authors.items[0].name, "Vincent Prouillet");
        assert_eq!(authors.items[0].slug, "vincent-prouillet");
        assert_eq!(
            authors.items[0].permalink,
            "http://a-website.com/fr/auteurs/vincent-prouillet/"
        );
        assert_eq!(authors.items[0].pages.len(), 1);

        assert_eq!(categories.items[0].name, "Other");
        assert_eq!(categories.items[0].slug, "other");
        assert_eq!(categories.items[0].permalink, "http://a-website.com/categories/other/");
        assert_eq!(categories.items[0].pages.len(), 1);

        assert_eq!(categories.items[1].name, "Programming tutorials");
        assert_eq!(categories.items[1].slug, "programming-tutorials");
        assert_eq!(
            categories.items[1].permalink,
            "http://a-website.com/categories/programming-tutorials/"
        );
        assert_eq!(categories.items[1].pages.len(), 1);
    }

    #[test]
    fn can_make_utf8_taxonomies() {
        let mut config = Config::default();
        config.slugify.taxonomies = SlugifyStrategy::Safe;
        config.languages.push(Language {
            feed: false,
            code: "fr".to_string(),
            ..Language::default()
        });
        let mut library = Library::new(2, 0, true);

        config.taxonomies = vec![TaxonomyConfig {
            name: "catégories".to_string(),
            lang: "fr".to_string(),
            ..TaxonomyConfig::default()
        }];

        let mut page = Page::default();
        page.lang = "fr".to_string();
        let mut taxo_page = HashMap::new();
        taxo_page.insert("catégories".to_string(), vec!["Écologie".to_string()]);
        page.meta.taxonomies = taxo_page;
        library.insert_page(page);

        let taxonomies = find_taxonomies(&config, &library).unwrap();
        let categories = &taxonomies[0];

        assert_eq!(categories.items.len(), 1);
        assert_eq!(categories.items[0].name, "Écologie");
        assert_eq!(categories.items[0].permalink, "http://a-website.com/fr/catégories/Écologie/");
        assert_eq!(categories.items[0].pages.len(), 1);
    }

    #[test]
    fn can_make_slugified_taxonomies_in_multiple_languages() {
        let mut config = Config::default();
        config.slugify.taxonomies = SlugifyStrategy::On;
        config.languages.push(Language {
            feed: false,
            code: "fr".to_string(),
            ..Language::default()
        });
        let mut library = Library::new(2, 0, true);

        config.taxonomies = vec![
            TaxonomyConfig {
                name: "categories".to_string(),
                lang: config.default_language.clone(),
                ..TaxonomyConfig::default()
            },
            TaxonomyConfig {
                name: "tags".to_string(),
                lang: config.default_language.clone(),
                ..TaxonomyConfig::default()
            },
            TaxonomyConfig {
                name: "auteurs".to_string(),
                lang: "fr".to_string(),
                ..TaxonomyConfig::default()
            },
            TaxonomyConfig {
                name: "tags".to_string(),
                lang: "fr".to_string(),
                ..TaxonomyConfig::default()
            },
        ];

        let mut page1 = Page::default();
        let mut taxo_page1 = HashMap::new();
        taxo_page1.insert("tags".to_string(), vec!["rust".to_string(), "db".to_string()]);
        taxo_page1.insert("categories".to_string(), vec!["Programming tutorials".to_string()]);
        page1.meta.taxonomies = taxo_page1;
        page1.lang = config.default_language.clone();
        library.insert_page(page1);

        let mut page2 = Page::default();
        let mut taxo_page2 = HashMap::new();
        taxo_page2.insert("tags".to_string(), vec!["rust".to_string()]);
        taxo_page2.insert("categories".to_string(), vec!["Other".to_string()]);
        page2.meta.taxonomies = taxo_page2;
        page2.lang = config.default_language.clone();
        library.insert_page(page2);

        let mut page3 = Page::default();
        page3.lang = "fr".to_string();
        let mut taxo_page3 = HashMap::new();
        taxo_page3.insert("tags".to_string(), vec!["rust".to_string()]);
        taxo_page3.insert("auteurs".to_string(), vec!["Vincent Prouillet".to_string()]);
        page3.meta.taxonomies = taxo_page3;
        library.insert_page(page3);

        let taxonomies = find_taxonomies(&config, &library).unwrap();
        let (tags, categories, authors) = {
            let mut t = None;
            let mut c = None;
            let mut a = None;
            for x in taxonomies {
                match x.kind.name.as_ref() {
                    "tags" => {
                        if x.kind.lang == "en" {
                            t = Some(x)
                        }
                    }
                    "categories" => c = Some(x),
                    "auteurs" => a = Some(x),
                    _ => unreachable!(),
                }
            }
            (t.unwrap(), c.unwrap(), a.unwrap())
        };

        assert_eq!(tags.items.len(), 2);
        assert_eq!(categories.items.len(), 2);
        assert_eq!(authors.items.len(), 1);

        assert_eq!(tags.items[0].name, "db");
        assert_eq!(tags.items[0].slug, "db");
        assert_eq!(tags.items[0].permalink, "http://a-website.com/tags/db/");
        assert_eq!(tags.items[0].pages.len(), 1);

        assert_eq!(tags.items[1].name, "rust");
        assert_eq!(tags.items[1].slug, "rust");
        assert_eq!(tags.items[1].permalink, "http://a-website.com/tags/rust/");
        assert_eq!(tags.items[1].pages.len(), 2);

        assert_eq!(authors.items[0].name, "Vincent Prouillet");
        assert_eq!(authors.items[0].slug, "vincent-prouillet");
        assert_eq!(
            authors.items[0].permalink,
            "http://a-website.com/fr/auteurs/vincent-prouillet/"
        );
        assert_eq!(authors.items[0].pages.len(), 1);

        assert_eq!(categories.items[0].name, "Other");
        assert_eq!(categories.items[0].slug, "other");
        assert_eq!(categories.items[0].permalink, "http://a-website.com/categories/other/");
        assert_eq!(categories.items[0].pages.len(), 1);

        assert_eq!(categories.items[1].name, "Programming tutorials");
        assert_eq!(categories.items[1].slug, "programming-tutorials");
        assert_eq!(
            categories.items[1].permalink,
            "http://a-website.com/categories/programming-tutorials/"
        );
        assert_eq!(categories.items[1].pages.len(), 1);
    }

    #[test]
    fn taxonomies_are_groupted_by_permalink() {
        let mut config = Config::default();
        let mut library = Library::new(2, 0, false);

        config.taxonomies = vec![
            TaxonomyConfig {
                name: "test-taxonomy".to_string(),
                lang: config.default_language.clone(),
                ..TaxonomyConfig::default()
            },
            TaxonomyConfig {
                name: "test taxonomy".to_string(),
                lang: config.default_language.clone(),
                ..TaxonomyConfig::default()
            },
            TaxonomyConfig {
                name: "test-taxonomy ".to_string(),
                lang: config.default_language.clone(),
                ..TaxonomyConfig::default()
            },
            TaxonomyConfig {
                name: "Test-Taxonomy ".to_string(),
                lang: config.default_language.clone(),
                ..TaxonomyConfig::default()
            },
        ];

        let mut page1 = Page::default();
        let mut taxo_page1 = HashMap::new();
        taxo_page1.insert(
            "test-taxonomy".to_string(),
            vec!["term one".to_string(), "term two".to_string()],
        );
        page1.meta.taxonomies = taxo_page1;
        page1.lang = config.default_language.clone();
        library.insert_page(page1);

        let mut page2 = Page::default();
        let mut taxo_page2 = HashMap::new();
        taxo_page2.insert(
            "test taxonomy".to_string(),
            vec!["Term Two".to_string(), "term-one".to_string()],
        );
        page2.meta.taxonomies = taxo_page2;
        page2.lang = config.default_language.clone();
        library.insert_page(page2);

        let mut page3 = Page::default();
        let mut taxo_page3 = HashMap::new();
        taxo_page3.insert("test-taxonomy ".to_string(), vec!["term one ".to_string()]);
        page3.meta.taxonomies = taxo_page3;
        page3.lang = config.default_language.clone();
        library.insert_page(page3);

        let mut page4 = Page::default();
        let mut taxo_page4 = HashMap::new();
        taxo_page4.insert("Test-Taxonomy ".to_string(), vec!["Term-Two ".to_string()]);
        page4.meta.taxonomies = taxo_page4;
        page4.lang = config.default_language.clone();
        library.insert_page(page4);

        // taxonomies should all be the same
        let taxonomies = find_taxonomies(&config, &library).unwrap();
        assert_eq!(taxonomies.len(), 1);

        let tax = &taxonomies[0];

        // terms should be "term one", "term two"
        assert_eq!(tax.items.len(), 2);

        let term1 = &tax.items[0];
        let term2 = &tax.items[1];

        assert_eq!(term1.name, "term one");
        assert_eq!(term1.slug, "term-one");
        assert_eq!(term1.permalink, "http://a-website.com/test-taxonomy/term-one/");
        assert_eq!(term1.pages.len(), 3);

        assert_eq!(term2.name, "Term Two");
        assert_eq!(term2.slug, "term-two");
        assert_eq!(term2.permalink, "http://a-website.com/test-taxonomy/term-two/");
        assert_eq!(term2.pages.len(), 3);
    }

    #[test]
    fn taxonomies_with_unic_are_grouped_with_default_slugify_strategy() {
        let mut config = Config::default();
        let mut library = Library::new(2, 0, false);

        config.taxonomies = vec![
            TaxonomyConfig {
                name: "test-taxonomy".to_string(),
                lang: config.default_language.clone(),
                ..TaxonomyConfig::default()
            },
            TaxonomyConfig {
                name: "test taxonomy".to_string(),
                lang: config.default_language.clone(),
                ..TaxonomyConfig::default()
            },
            TaxonomyConfig {
                name: "test-taxonomy ".to_string(),
                lang: config.default_language.clone(),
                ..TaxonomyConfig::default()
            },
            TaxonomyConfig {
                name: "Test-Taxonomy ".to_string(),
                lang: config.default_language.clone(),
                ..TaxonomyConfig::default()
            },
        ];

        let mut page1 = Page::default();
        let mut taxo_page1 = HashMap::new();
        taxo_page1.insert("test-taxonomy".to_string(), vec!["Ecole".to_string()]);
        page1.meta.taxonomies = taxo_page1;
        page1.lang = config.default_language.clone();
        library.insert_page(page1);

        let mut page2 = Page::default();
        let mut taxo_page2 = HashMap::new();
        taxo_page2.insert("test taxonomy".to_string(), vec!["École".to_string()]);
        page2.meta.taxonomies = taxo_page2;
        page2.lang = config.default_language.clone();
        library.insert_page(page2);

        let mut page3 = Page::default();
        let mut taxo_page3 = HashMap::new();
        taxo_page3.insert("test-taxonomy ".to_string(), vec!["ecole".to_string()]);
        page3.meta.taxonomies = taxo_page3;
        page3.lang = config.default_language.clone();
        library.insert_page(page3);

        let mut page4 = Page::default();
        let mut taxo_page4 = HashMap::new();
        taxo_page4.insert("Test-Taxonomy ".to_string(), vec!["école".to_string()]);
        page4.meta.taxonomies = taxo_page4;
        page4.lang = config.default_language.clone();
        library.insert_page(page4);

        // taxonomies should all be the same
        let taxonomies = find_taxonomies(&config, &library).unwrap();
        assert_eq!(taxonomies.len(), 1);

        let tax = &taxonomies[0];

        // under the default slugify stratagy all of the provided terms should be the same
        assert_eq!(tax.items.len(), 1);

        let term1 = &tax.items[0];

        assert_eq!(term1.name, "Ecole");
        assert_eq!(term1.slug, "ecole");
        assert_eq!(term1.permalink, "http://a-website.com/test-taxonomy/ecole/");
        assert_eq!(term1.pages.len(), 4);
    }

    #[test]
    fn taxonomies_with_unic_are_not_grouped_with_safe_slugify_strategy() {
        let mut config = Config::default();
        config.slugify = Slugify {
            paths: SlugifyStrategy::Safe,
            taxonomies: SlugifyStrategy::Safe,
            anchors: SlugifyStrategy::Safe,
        };
        let mut library = Library::new(2, 0, false);

        config.taxonomies = vec![
            TaxonomyConfig {
                name: "test-taxonomy".to_string(),
                lang: config.default_language.clone(),
                ..TaxonomyConfig::default()
            },
            TaxonomyConfig {
                name: "test taxonomy".to_string(),
                lang: config.default_language.clone(),
                ..TaxonomyConfig::default()
            },
            TaxonomyConfig {
                name: "test-taxonomy ".to_string(),
                lang: config.default_language.clone(),
                ..TaxonomyConfig::default()
            },
            TaxonomyConfig {
                name: "Test-Taxonomy ".to_string(),
                lang: config.default_language.clone(),
                ..TaxonomyConfig::default()
            },
        ];

        let mut page1 = Page::default();
        let mut taxo_page1 = HashMap::new();
        taxo_page1.insert("test-taxonomy".to_string(), vec!["Ecole".to_string()]);
        page1.meta.taxonomies = taxo_page1;
        page1.lang = config.default_language.clone();
        library.insert_page(page1);

        let mut page2 = Page::default();
        let mut taxo_page2 = HashMap::new();
        taxo_page2.insert("test-taxonomy".to_string(), vec!["École".to_string()]);
        page2.meta.taxonomies = taxo_page2;
        page2.lang = config.default_language.clone();
        library.insert_page(page2);

        let mut page3 = Page::default();
        let mut taxo_page3 = HashMap::new();
        taxo_page3.insert("test-taxonomy".to_string(), vec!["ecole".to_string()]);
        page3.meta.taxonomies = taxo_page3;
        page3.lang = config.default_language.clone();
        library.insert_page(page3);

        let mut page4 = Page::default();
        let mut taxo_page4 = HashMap::new();
        taxo_page4.insert("test-taxonomy".to_string(), vec!["école".to_string()]);
        page4.meta.taxonomies = taxo_page4;
        page4.lang = config.default_language.clone();
        library.insert_page(page4);

        // taxonomies should all be the same
        let taxonomies = find_taxonomies(&config, &library).unwrap();
        let tax = &taxonomies[0];

        // if names are different permalinks should also be different so
        // the tems are still accessable
        for term1 in tax.items.iter() {
            for term2 in tax.items.iter() {
                assert!(term1.name == term2.name || term1.permalink != term2.permalink);
            }
        }

        // under the safe slugify strategy all terms should be distinct
        assert_eq!(tax.items.len(), 4);
    }
}
