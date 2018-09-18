#[macro_use]
extern crate serde_derive;
extern crate tera;

extern crate errors;
extern crate config;
extern crate content;
extern crate utils;
extern crate taxonomies;

#[cfg(test)]
extern crate front_matter;

use std::collections::HashMap;

use tera::{Tera, Context, to_value, Value};

use errors::{Result, ResultExt};
use config::Config;
use content::{Page, Section};
use utils::templates::render_template;
use taxonomies::{Taxonomy, TaxonomyItem};


#[derive(Clone, Debug, PartialEq)]
enum PaginationRoot<'a> {
    Section(&'a Section),
    Taxonomy(&'a Taxonomy),
}


/// A list of all the pages in the paginator with their index and links
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Pager<'a> {
    /// The page number in the paginator (1-indexed)
    pub index: usize,
    /// Permalink to that page
    permalink: String,
    /// Path to that page
    path: String,
    /// All pages for the pager
    pages: Vec<&'a Page>,
}

impl<'a> Pager<'a> {
    fn new(index: usize, pages: Vec<&'a Page>, permalink: String, path: String) -> Pager<'a> {
        Pager {
            index,
            permalink,
            path,
            pages,
        }
    }

    /// Returns a manually cloned Pager with the pages removed
    /// for use as template context
    fn clone_without_pages(&self) -> Pager<'a> {
        Pager {
            index: self.index,
            permalink: self.permalink.clone(),
            path: self.path.clone(),
            pages: vec![],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Paginator<'a> {
    /// All pages in the section
    all_pages: &'a [Page],
    /// Pages split in chunks of `paginate_by`
    pub pagers: Vec<Pager<'a>>,
    /// How many content pages on a paginated page at max
    paginate_by: usize,
    /// The thing we are creating the paginator for: section or taxonomy
    root: PaginationRoot<'a>,
    // Those below can be obtained from the root but it would make the code more complex than needed
    pub permalink: String,
    path: String,
    pub paginate_path: String,
    is_index: bool,
}

impl<'a> Paginator<'a> {
    /// Create a new paginator from a section
    /// It will always at least create one pager (the first) even if there are no pages to paginate
    pub fn from_section(all_pages: &'a [Page], section: &'a Section) -> Paginator<'a> {
        let paginate_by = section.meta.paginate_by.unwrap();
        let mut paginator = Paginator {
            all_pages,
            pagers: vec![],
            paginate_by,
            root: PaginationRoot::Section(section),
            permalink: section.permalink.clone(),
            path: section.path.clone(),
            paginate_path: section.meta.paginate_path.clone(),
            is_index: section.is_index(),
        };

        paginator.fill_pagers();
        paginator
    }

    /// Create a new paginator from a taxonomy
    /// It will always at least create one pager (the first) even if there are no pages to paginate
    pub fn from_taxonomy(taxonomy: &'a Taxonomy, item: &'a TaxonomyItem) -> Paginator<'a> {
        let paginate_by = taxonomy.kind.paginate_by.unwrap();
        let mut paginator = Paginator {
            all_pages: &item.pages,
            pagers: vec![],
            paginate_by,
            root: PaginationRoot::Taxonomy(taxonomy),
            permalink: item.permalink.clone(),
            path: format!("{}/{}", taxonomy.kind.name, item.slug),
            paginate_path: taxonomy.kind.paginate_path.clone().unwrap_or_else(|| "pages".to_string()),
            is_index: false,
        };

        paginator.fill_pagers();
        paginator
    }

    fn fill_pagers(&mut self) {
        let mut pages = vec![];
        let mut current_page = vec![];

        for page in self.all_pages {
            current_page.push(page);

            if current_page.len() == self.paginate_by {
                pages.push(current_page);
                current_page = vec![];
            }
        }
        if !current_page.is_empty() {
            pages.push(current_page);
        }

        let mut pagers = vec![];
        for (index, page) in pages.iter().enumerate() {
            // First page has no pagination path
            if index == 0 {
                pagers.push(Pager::new(1, page.clone(), self.permalink.clone(), self.path.clone()));
                continue;
            }

            let page_path = format!("{}/{}/", self.paginate_path, index + 1);
            let permalink = format!("{}{}", self.permalink, page_path);

            let pager_path = if self.is_index {
                page_path
            } else {
                if self.path.ends_with("/") {
                    format!("{}{}", self.path, page_path)
                } else {
                    format!("{}/{}", self.path, page_path)
                }
            };

            pagers.push(Pager::new(
                index + 1,
                page.clone(),
                permalink,
                pager_path,
            ));
        }

        // We always have the index one at least
        if pagers.is_empty() {
            pagers.push(Pager::new(1, vec![], self.permalink.clone(), self.path.clone()));
        }

        self.pagers = pagers;
    }

    pub fn build_paginator_context(&self, current_pager: &Pager) -> HashMap<&str, Value> {
        let mut paginator = HashMap::new();
        // the pager index is 1-indexed so we want a 0-indexed one for indexing there
        let pager_index = current_pager.index - 1;

        // Global variables
        paginator.insert("paginate_by", to_value(self.paginate_by).unwrap());
        paginator.insert("first", to_value(&self.permalink).unwrap());
        let last_pager = &self.pagers[self.pagers.len() - 1];
        paginator.insert("last", to_value(&last_pager.permalink).unwrap());
        paginator.insert(
            "pagers",
            to_value(
                &self.pagers.iter().map(|p| p.clone_without_pages()).collect::<Vec<_>>()
            ).unwrap(),
        );

        // Variables for this specific page
        if pager_index > 0 {
            let prev_pager = &self.pagers[pager_index - 1];
            paginator.insert("previous", to_value(&prev_pager.permalink).unwrap());
        } else {
            paginator.insert("previous", to_value::<Option<()>>(None).unwrap());
        }

        if pager_index < self.pagers.len() - 1 {
            let next_pager = &self.pagers[pager_index + 1];
            paginator.insert("next", to_value(&next_pager.permalink).unwrap());
        } else {
            paginator.insert("next", to_value::<Option<()>>(None).unwrap());
        }
        paginator.insert("pages", to_value(&current_pager.pages).unwrap());
        paginator.insert("current_index", to_value(current_pager.index).unwrap());

        paginator
    }

    pub fn render_pager(&self, pager: &Pager, config: &Config, tera: &Tera) -> Result<String> {
        let mut context = Context::new();
        context.insert("config", &config);
        let template_name = match self.root {
            PaginationRoot::Section(s) => {
                context.insert("section", &s);
                s.get_template_name()
            }
            PaginationRoot::Taxonomy(t) => {
                context.insert("taxonomy", &t.kind);
                format!("{}/single.html", t.kind.name)
            }
        };
        context.insert("current_url", &pager.permalink);
        context.insert("current_path", &pager.path);
        context.insert("paginator", &self.build_paginator_context(pager));

        render_template(&template_name, tera, &context, &config.theme)
            .chain_err(|| format!("Failed to render pager {}", pager.index))
    }
}

#[cfg(test)]
mod tests {
    use tera::to_value;

    use front_matter::SectionFrontMatter;
    use content::{Page, Section};
    use config::Taxonomy as TaxonomyConfig;
    use taxonomies::{Taxonomy, TaxonomyItem};

    use super::Paginator;

    fn create_section(is_index: bool) -> Section {
        let mut f = SectionFrontMatter::default();
        f.paginate_by = Some(2);
        f.paginate_path = "page".to_string();
        let mut s = Section::new("content/_index.md", f);
        if !is_index {
            s.path = "posts/".to_string();
            s.permalink = "https://vincent.is/posts/".to_string();
            s.file.components = vec!["posts".to_string()];
        } else {
            s.permalink = "https://vincent.is/".to_string();
        }
        s
    }

    #[test]
    fn test_can_create_paginator() {
        let pages = vec![
            Page::default(),
            Page::default(),
            Page::default(),
        ];
        let section = create_section(false);
        let paginator = Paginator::from_section(pages.as_slice(), &section);
        assert_eq!(paginator.pagers.len(), 2);

        assert_eq!(paginator.pagers[0].index, 1);
        assert_eq!(paginator.pagers[0].pages.len(), 2);
        assert_eq!(paginator.pagers[0].permalink, "https://vincent.is/posts/");
        assert_eq!(paginator.pagers[0].path, "posts/");

        assert_eq!(paginator.pagers[1].index, 2);
        assert_eq!(paginator.pagers[1].pages.len(), 1);
        assert_eq!(paginator.pagers[1].permalink, "https://vincent.is/posts/page/2/");
        assert_eq!(paginator.pagers[1].path, "posts/page/2/");
    }

    #[test]
    fn test_can_create_paginator_for_index() {
        let pages = vec![
            Page::default(),
            Page::default(),
            Page::default(),
        ];
        let section = create_section(true);
        let paginator = Paginator::from_section(pages.as_slice(), &section);
        assert_eq!(paginator.pagers.len(), 2);

        assert_eq!(paginator.pagers[0].index, 1);
        assert_eq!(paginator.pagers[0].pages.len(), 2);
        assert_eq!(paginator.pagers[0].permalink, "https://vincent.is/");
        assert_eq!(paginator.pagers[0].path, "");

        assert_eq!(paginator.pagers[1].index, 2);
        assert_eq!(paginator.pagers[1].pages.len(), 1);
        assert_eq!(paginator.pagers[1].permalink, "https://vincent.is/page/2/");
        assert_eq!(paginator.pagers[1].path, "page/2/");
    }

    #[test]
    fn test_can_build_paginator_context() {
        let pages = vec![
            Page::default(),
            Page::default(),
            Page::default(),
        ];
        let section = create_section(false);
        let paginator = Paginator::from_section(pages.as_slice(), &section);
        assert_eq!(paginator.pagers.len(), 2);

        let context = paginator.build_paginator_context(&paginator.pagers[0]);
        assert_eq!(context["paginate_by"], to_value(2).unwrap());
        assert_eq!(context["first"], to_value("https://vincent.is/posts/").unwrap());
        assert_eq!(context["last"], to_value("https://vincent.is/posts/page/2/").unwrap());
        assert_eq!(context["previous"], to_value::<Option<()>>(None).unwrap());
        assert_eq!(context["next"], to_value("https://vincent.is/posts/page/2/").unwrap());
        assert_eq!(context["current_index"], to_value(1).unwrap());

        let context = paginator.build_paginator_context(&paginator.pagers[1]);
        assert_eq!(context["paginate_by"], to_value(2).unwrap());
        assert_eq!(context["first"], to_value("https://vincent.is/posts/").unwrap());
        assert_eq!(context["last"], to_value("https://vincent.is/posts/page/2/").unwrap());
        assert_eq!(context["next"], to_value::<Option<()>>(None).unwrap());
        assert_eq!(context["previous"], to_value("https://vincent.is/posts/").unwrap());
        assert_eq!(context["current_index"], to_value(2).unwrap());
    }

    #[test]
    fn test_can_create_paginator_for_taxonomy() {
        let pages = vec![
            Page::default(),
            Page::default(),
            Page::default(),
        ];
        let taxonomy_def = TaxonomyConfig {
            name: "tags".to_string(),
            paginate_by: Some(2),
            ..TaxonomyConfig::default()
        };
        let taxonomy_item = TaxonomyItem {
            name: "Something".to_string(),
            slug: "something".to_string(),
            permalink: "https://vincent.is/tags/something/".to_string(),
            pages,
        };
        let taxonomy = Taxonomy { kind: taxonomy_def, items: vec![taxonomy_item.clone()] };
        let paginator = Paginator::from_taxonomy(&taxonomy, &taxonomy_item);
        assert_eq!(paginator.pagers.len(), 2);

        assert_eq!(paginator.pagers[0].index, 1);
        assert_eq!(paginator.pagers[0].pages.len(), 2);
        assert_eq!(paginator.pagers[0].permalink, "https://vincent.is/tags/something/");
        assert_eq!(paginator.pagers[0].path, "tags/something");

        assert_eq!(paginator.pagers[1].index, 2);
        assert_eq!(paginator.pagers[1].pages.len(), 1);
        assert_eq!(paginator.pagers[1].permalink, "https://vincent.is/tags/something/pages/2/");
        assert_eq!(paginator.pagers[1].path, "tags/something/pages/2/");
    }
}
