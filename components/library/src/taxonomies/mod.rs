use std::collections::HashMap;

use slotmap::Key;
use slug::slugify;
use tera::{Context, Tera};

use config::{Config, Taxonomy as TaxonomyConfig};
use errors::{Error, Result};
use utils::templates::render_template;

use content::SerializingPage;
use library::Library;
use sorting::sort_pages_by_date;

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
#[derive(Debug, Clone, PartialEq)]
pub struct TaxonomyItem {
    pub name: String,
    pub slug: String,
    pub permalink: String,
    pub pages: Vec<Key>,
}

impl TaxonomyItem {
    pub fn new(
        name: &str,
        taxonomy: &TaxonomyConfig,
        config: &Config,
        keys: Vec<Key>,
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
        let slug = slugify(name);
        let permalink = if taxonomy.lang != config.default_language {
            config.make_permalink(&format!("/{}/{}/{}", taxonomy.lang, taxonomy.name, slug))
        } else {
            config.make_permalink(&format!("/{}/{}", taxonomy.name, slug))
        };

        // We still append pages without dates at the end
        pages.extend(ignored_pages);

        TaxonomyItem { name: name.to_string(), permalink, slug, pages }
    }

    pub fn serialize<'a>(&'a self, library: &'a Library) -> SerializedTaxonomyItem<'a> {
        SerializedTaxonomyItem::from_item(self, library)
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
    // this vec is sorted by the count of item
    pub items: Vec<TaxonomyItem>,
}

impl Taxonomy {
    fn new(
        kind: TaxonomyConfig,
        config: &Config,
        items: HashMap<String, Vec<Key>>,
        library: &Library,
    ) -> Taxonomy {
        let mut sorted_items = vec![];
        for (name, pages) in items {
            sorted_items.push(TaxonomyItem::new(&name, &kind, config, pages, library));
        }
        sorted_items.sort_by(|a, b| a.name.cmp(&b.name));

        Taxonomy { kind, items: sorted_items }
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
        context.insert("term", &SerializedTaxonomyItem::from_item(item, library));
        context.insert("taxonomy", &self.kind);
        context.insert(
            "current_url",
            &config.make_permalink(&format!("{}/{}", self.kind.name, item.slug)),
        );
        context.insert("current_path", &format!("/{}/{}", self.kind.name, item.slug));

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
        context.insert("taxonomy", &self.kind);
        context.insert("current_url", &config.make_permalink(&self.kind.name));
        context.insert("current_path", &self.kind.name);

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
            m.insert(t.name.clone(), t);
        }
        m
    };
    let mut all_taxonomies = HashMap::new();

    for (key, page) in library.pages() {
        // Draft are not part of taxonomies
        if page.is_draft() {
            continue;
        }

        for (name, val) in &page.meta.taxonomies {
            if taxonomies_def.contains_key(name) {
                if taxonomies_def[name].lang != page.lang {
                    bail!(
                        "Page `{}` has taxonomy `{}` which is not available in that language",
                        page.file.path.display(),
                        name
                    );
                }

                all_taxonomies.entry(name).or_insert_with(HashMap::new);

                for v in val {
                    all_taxonomies
                        .get_mut(name)
                        .unwrap()
                        .entry(v.to_string())
                        .or_insert_with(|| vec![])
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
        taxonomies.push(Taxonomy::new(taxonomies_def[name].clone(), config, taxo, library));
    }

    Ok(taxonomies)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use config::{Config, Language, Taxonomy as TaxonomyConfig};
    use content::Page;
    use library::Library;

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
        config.languages.push(Language { rss: false, code: "fr".to_string() });
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
                    "tags" => t = Some(x),
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
    fn errors_on_taxonomy_of_different_language() {
        let mut config = Config::default();
        config.languages.push(Language { rss: false, code: "fr".to_string() });
        let mut library = Library::new(2, 0, false);

        config.taxonomies =
            vec![TaxonomyConfig { name: "tags".to_string(), ..TaxonomyConfig::default() }];

        let mut page1 = Page::default();
        page1.lang = "fr".to_string();
        let mut taxo_page1 = HashMap::new();
        taxo_page1.insert("tags".to_string(), vec!["rust".to_string(), "db".to_string()]);
        page1.meta.taxonomies = taxo_page1;
        library.insert_page(page1);

        let taxonomies = find_taxonomies(&config, &library);
        assert!(taxonomies.is_err());
        let err = taxonomies.unwrap_err();
        // no path as this is created by Default
        assert_eq!(
            format!("{}", err),
            "Page `` has taxonomy `tags` which is not available in that language"
        );
    }
}
