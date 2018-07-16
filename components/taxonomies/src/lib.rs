#[macro_use]
extern crate serde_derive;
extern crate tera;
extern crate slug;

extern crate errors;
extern crate config;
extern crate content;
extern crate front_matter;
extern crate utils;

use std::collections::HashMap;

use slug::slugify;
use tera::{Context, Tera};

use config::{Config, Taxonomy as TaxonomyConfig};
use errors::{Result, ResultExt};
use content::{Page, sort_pages};
use front_matter::SortBy;
use utils::templates::render_template;


/// A tag or category
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TaxonomyItem {
    pub name: String,
    pub slug: String,
    pub permalink: String,
    pub pages: Vec<Page>,
}

impl TaxonomyItem {
    pub fn new(name: &str, path: &str, config: &Config, pages: Vec<Page>) -> TaxonomyItem {
        // Taxonomy are almost always used for blogs so we filter by dates
        // and it's not like we can sort things across sections by anything other
        // than dates
        let (mut pages, ignored_pages) = sort_pages(pages, SortBy::Date);
        let slug = slugify(name);
        let permalink = {
            config.make_permalink(&format!("/{}/{}", path, slug))
        };

        // We still append pages without dates at the end
        pages.extend(ignored_pages);

        TaxonomyItem {
            name: name.to_string(),
            permalink,
            slug,
            pages,
        }
    }
}

/// All the tags or categories
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Taxonomy {
    pub kind: TaxonomyConfig,
    // this vec is sorted by the count of item
    pub items: Vec<TaxonomyItem>,
}

impl Taxonomy {
    fn new(kind: TaxonomyConfig, config: &Config, items: HashMap<String, Vec<Page>>) -> Taxonomy {
        let mut sorted_items = vec![];
        for (name, pages) in items {
            sorted_items.push(
                TaxonomyItem::new(&name, &kind.name, config, pages)
            );
        }
        sorted_items.sort_by(|a, b| a.name.cmp(&b.name));

        Taxonomy {
            kind,
            items: sorted_items,
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn render_term(&self, item: &TaxonomyItem, tera: &Tera, config: &Config) -> Result<String> {
        let mut context = Context::new();
        context.add("config", config);
        context.add("term", item);
        context.add("taxonomy", &self.kind);
        context.add("current_url", &config.make_permalink(&format!("{}/{}", self.kind.name, item.slug)));
        context.add("current_path", &format!("/{}/{}", self.kind.name, item.slug));

        render_template(&format!("{}/single.html", self.kind.name), tera, &context, &config.theme)
            .chain_err(|| format!("Failed to render single term {} page.", self.kind.name))
    }

    pub fn render_all_terms(&self, tera: &Tera, config: &Config) -> Result<String> {
        let mut context = Context::new();
        context.add("config", config);
        context.add("terms", &self.items);
        context.add("taxonomy", &self.kind);
        context.add("current_url", &config.make_permalink(&self.kind.name));
        context.add("current_path", &self.kind.name);

        render_template(&format!("{}/list.html", self.kind.name), tera, &context, &config.theme)
            .chain_err(|| format!("Failed to render a list of {} page.", self.kind.name))
    }
}

pub fn find_taxonomies(config: &Config, all_pages: &[Page]) -> Vec<Taxonomy> {
    let taxonomies_def = {
        let mut m = HashMap::new();
        for t in &config.taxonomies {
            m.insert(t.name.clone(), t);
        }
        m
    };
    let mut all_taxonomies = HashMap::new();

    // Find all the taxonomies first
    for page in all_pages {
        for (name, val) in &page.meta.taxonomies {
            if taxonomies_def.contains_key(name) {
                all_taxonomies
                    .entry(name)
                    .or_insert_with(|| HashMap::new());

                for v in val {
                    all_taxonomies.get_mut(name)
                        .unwrap()
                        .entry(v.to_string())
                        .or_insert_with(|| vec![])
                        .push(page.clone());
                }
            } else {
                // TODO: bail with error
            }
        }
    }

    let mut taxonomies = vec![];

    for (name, taxo) in all_taxonomies {
        taxonomies.push(Taxonomy::new(taxonomies_def[name].clone(), config, taxo));
    }

    taxonomies
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use config::{Config, Taxonomy};
    use content::Page;

    #[test]
    fn can_make_taxonomies() {
        let mut config = Config::default();
        config.taxonomies = vec![
            Taxonomy {name: "categories".to_string(), ..Taxonomy::default()},
            Taxonomy {name: "tags".to_string(), ..Taxonomy::default()},
            Taxonomy {name: "authors".to_string(), ..Taxonomy::default()},
        ];
        let mut page1 = Page::default();
        let mut taxo_page1 = HashMap::new();
        taxo_page1.insert("tags".to_string(), vec!["rust".to_string(), "db".to_string()]);
        taxo_page1.insert("categories".to_string(), vec!["Programming tutorials".to_string()]);
        page1.meta.taxonomies = taxo_page1;
        let mut page2 = Page::default();
        let mut taxo_page2 = HashMap::new();
        taxo_page2.insert("tags".to_string(), vec!["rust".to_string(), "js".to_string()]);
        taxo_page2.insert("categories".to_string(), vec!["Other".to_string()]);
        page2.meta.taxonomies = taxo_page2;
        let mut page3 = Page::default();
        let mut taxo_page3 = HashMap::new();
        taxo_page3.insert("tags".to_string(), vec!["js".to_string()]);
        taxo_page3.insert("authors".to_string(), vec!["Vincent Prouillet".to_string()]);
        page3.meta.taxonomies = taxo_page3;
        let pages = vec![page1, page2, page3];

        let taxonomies = find_taxonomies(&config, &pages);
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
        assert_eq!(categories.items[1].permalink, "http://a-website.com/categories/programming-tutorials/");
        assert_eq!(categories.items[1].pages.len(), 1);
    }
}
