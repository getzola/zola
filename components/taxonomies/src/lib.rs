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

use config::Config;
use errors::{Result, ResultExt};
use content::{Page, sort_pages};
use front_matter::SortBy;
use utils::templates::render_template;


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TaxonomyKind {
    Tags,
    Categories,
}

/// A tag or category
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TaxonomyItem {
    pub name: String,
    pub slug: String,
    pub permalink: String,
    pub pages: Vec<Page>,
}

impl TaxonomyItem {
    pub fn new(name: &str, kind: TaxonomyKind, config: &Config, pages: Vec<Page>) -> TaxonomyItem {
        // Taxonomy are almost always used for blogs so we filter by dates
        // and it's not like we can sort things across sections by anything other
        // than dates
        let (mut pages, ignored_pages) = sort_pages(pages, SortBy::Date);
        let slug = slugify(name);
        let permalink = {
            let kind_path = if kind == TaxonomyKind::Tags { "tags" } else { "categories" };
            config.make_permalink(&format!("/{}/{}", kind_path, slug))
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
#[derive(Debug, Clone, PartialEq)]
pub struct Taxonomy {
    pub kind: TaxonomyKind,
    // this vec is sorted by the count of item
    pub items: Vec<TaxonomyItem>,
}

impl Taxonomy {
    pub fn find_tags_and_categories(config: &Config, all_pages: &[Page]) -> (Taxonomy, Taxonomy) {
        let mut tags = HashMap::new();
        let mut categories = HashMap::new();

        // Find all the tags/categories first
        for page in all_pages {
            if let Some(ref category) = page.meta.category {
                categories
                    .entry(category.to_string())
                    .or_insert_with(|| vec![])
                    .push(page.clone());
            }

            if let Some(ref t) = page.meta.tags {
                for tag in t {
                    tags
                        .entry(tag.to_string())
                        .or_insert_with(|| vec![])
                        .push(page.clone());
                }
            }
        }

        // Then make TaxonomyItem out of them, after sorting it
        let tags_taxonomy = Taxonomy::new(TaxonomyKind::Tags, config, tags);
        let categories_taxonomy = Taxonomy::new(TaxonomyKind::Categories, config, categories);

        (tags_taxonomy, categories_taxonomy)
    }

    fn new(kind: TaxonomyKind, config: &Config, items: HashMap<String, Vec<Page>>) -> Taxonomy {
        let mut sorted_items = vec![];
        for (name, pages) in &items {
            sorted_items.push(
                TaxonomyItem::new(name, kind, config, pages.clone())
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

    pub fn get_single_item_name(&self) -> String {
        match self.kind {
            TaxonomyKind::Tags => "tag".to_string(),
            TaxonomyKind::Categories => "category".to_string(),
        }
    }

    pub fn get_list_name(&self) -> String {
        match self.kind {
            TaxonomyKind::Tags => "tags".to_string(),
            TaxonomyKind::Categories => "categories".to_string(),
        }
    }

    pub fn render_single_item(&self, item: &TaxonomyItem, tera: &Tera, config: &Config) -> Result<String> {
        let name = self.get_single_item_name();
        let mut context = Context::new();
        context.add("config", config);
        context.add(&name, item);
        context.add("current_url", &config.make_permalink(&format!("{}/{}", name, item.slug)));
        context.add("current_path", &format!("/{}/{}", name, item.slug));

        render_template(&format!("{}.html", name), tera, &context, config.theme.clone())
            .chain_err(|| format!("Failed to render {} page.", name))
    }

    pub fn render_list(&self, tera: &Tera, config: &Config) -> Result<String> {
        let name = self.get_list_name();
        let mut context = Context::new();
        context.add("config", config);
        context.add(&name, &self.items);
        context.add("current_url", &config.make_permalink(&name));
        context.add("current_path", &name);

        render_template(&format!("{}.html", name), tera, &context, config.theme.clone())
            .chain_err(|| format!("Failed to render {} page.", name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use config::Config;
    use content::Page;

    #[test]
    fn can_make_taxonomies() {
        let config = Config::default();
        let mut page1 = Page::default();
        page1.meta.tags = Some(vec!["rust".to_string(), "db".to_string()]);
        page1.meta.category = Some("Programming tutorials".to_string());
        let mut page2 = Page::default();
        page2.meta.tags = Some(vec!["rust".to_string(), "js".to_string()]);
        page2.meta.category = Some("Other".to_string());
        let mut page3 = Page::default();
        page3.meta.tags = Some(vec!["js".to_string()]);
        let pages = vec![page1, page2, page3];

        let (tags, categories) = Taxonomy::find_tags_and_categories(&config, &pages);

        assert_eq!(tags.items.len(), 3);
        assert_eq!(categories.items.len(), 2);

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
