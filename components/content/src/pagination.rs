use config::Config;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;

use errors::{Context as ErrorContext, Result};
use libs::tera::{to_value, Context, Tera, Value};
use utils::templates::{check_template_fallbacks, render_template};

use crate::library::Library;
use crate::ser::{SectionSerMode, SerializingPage, SerializingSection};
use crate::taxonomies::{Taxonomy, TaxonomyTerm};
use crate::Section;

#[derive(Clone, Debug, PartialEq, Eq)]
enum PaginationRoot<'a> {
    Section(&'a Section),
    Taxonomy(&'a Taxonomy, &'a TaxonomyTerm),
}

/// A list of all the pages in the paginator with their index and links
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Pager<'a> {
    /// The page number in the paginator (1-indexed)
    pub index: usize,
    /// Permalink to that page
    pub permalink: String,
    /// Path to that page
    pub path: String,
    /// All pages for the pager
    pub pages: Vec<SerializingPage<'a>>,
}

impl<'a> Pager<'a> {
    fn new(
        index: usize,
        pages: Vec<SerializingPage<'a>>,
        permalink: String,
        path: String,
    ) -> Pager<'a> {
        Pager { index, permalink, path, pages }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paginator<'a> {
    /// All pages in the section/taxonomy
    all_pages: Cow<'a, [PathBuf]>,
    /// Pages split in chunks of `paginate_by`
    pub pagers: Vec<Pager<'a>>,
    /// How many content pages on a paginated page at max
    paginate_by: usize,
    /// whether to reverse before grouping
    paginate_reversed: bool,
    /// The thing we are creating the paginator for: section or taxonomy
    root: PaginationRoot<'a>,
    // Those below can be obtained from the root but it would make the code more complex than needed
    pub permalink: String,
    path: String,
    pub paginate_path: String,
    template: String,
    /// Whether this is the index section, we need it for the template name
    is_index: bool,
}

impl<'a> Paginator<'a> {
    /// Create a new paginator from a section
    /// It will always at least create one pager (the first) even if there are not enough pages to paginate
    pub fn from_section(section: &'a Section, library: &'a Library) -> Paginator<'a> {
        let paginate_by = section.meta.paginate_by.unwrap();
        let mut paginator = Paginator {
            all_pages: Cow::from(&section.pages[..]),
            pagers: Vec::with_capacity(section.pages.len() / paginate_by),
            paginate_by,
            paginate_reversed: section.meta.paginate_reversed,
            root: PaginationRoot::Section(section),
            permalink: section.permalink.clone(),
            path: section.path.clone(),
            paginate_path: section.meta.paginate_path.clone(),
            is_index: section.is_index(),
            template: section.get_template_name().to_string(),
        };

        paginator.fill_pagers(library);
        paginator
    }

    /// Create a new paginator from a taxonomy
    /// It will always at least create one pager (the first) even if there are not enough pages to paginate
    pub fn from_taxonomy(
        taxonomy: &'a Taxonomy,
        item: &'a TaxonomyTerm,
        library: &'a Library,
        tera: &Tera,
        theme: &Option<String>,
    ) -> Paginator<'a> {
        let paginate_by = taxonomy.kind.paginate_by.unwrap();
        // Check for taxon-specific template, or use generic as fallback.
        let specific_template = format!("{}/single.html", taxonomy.kind.name);
        let template = check_template_fallbacks(&specific_template, tera, theme)
            .unwrap_or("taxonomy_single.html");
        let mut paginator = Paginator {
            all_pages: Cow::Borrowed(&item.pages),
            pagers: Vec::with_capacity(item.pages.len() / paginate_by),
            paginate_by,
            paginate_reversed: false,
            root: PaginationRoot::Taxonomy(taxonomy, item),
            permalink: item.permalink.clone(),
            path: item.path.clone(),
            paginate_path: taxonomy.kind.paginate_path().to_owned(),
            is_index: false,
            template: template.to_string(),
        };

        // taxonomy paginators have no sorting so we won't have to reverse
        paginator.fill_pagers(library);
        paginator
    }

    fn fill_pagers(&mut self, library: &'a Library) {
        // the list of pagers
        let mut pages = vec![];
        // the pages in the current pagers
        let mut current_page = vec![];

        if self.paginate_reversed {
            self.all_pages.to_mut().reverse();
        }

        for p in &*self.all_pages {
            let page = &library.pages[p];
            current_page.push(SerializingPage::new(page, Some(library), false));

            if current_page.len() == self.paginate_by {
                pages.push(current_page);
                current_page = vec![];
            }
        }

        if !current_page.is_empty() {
            pages.push(current_page);
        }

        let mut pagers = vec![];
        for (index, page) in pages.into_iter().enumerate() {
            // First page has no pagination path
            if index == 0 {
                pagers.push(Pager::new(1, page, self.permalink.clone(), self.path.clone()));
                continue;
            }

            let page_path = if self.paginate_path.is_empty() {
                format!("{}/", index + 1)
            } else {
                format!("{}/{}/", self.paginate_path, index + 1)
            };
            let permalink = format!("{}{}", self.permalink, page_path);

            let pager_path = if self.is_index {
                format!("/{}", page_path)
            } else if self.path.ends_with('/') {
                format!("{}{}", self.path, page_path)
            } else {
                format!("{}/{}", self.path, page_path)
            };

            pagers.push(Pager::new(index + 1, page, permalink, pager_path));
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

        // Variables for this specific page
        if pager_index > 0 {
            let prev_pager = &self.pagers[pager_index - 1];
            paginator.insert("previous", to_value(&prev_pager.permalink).unwrap());
        } else {
            paginator.insert("previous", Value::Null);
        }

        if pager_index < self.pagers.len() - 1 {
            let next_pager = &self.pagers[pager_index + 1];
            paginator.insert("next", to_value(&next_pager.permalink).unwrap());
        } else {
            paginator.insert("next", Value::Null);
        }
        paginator.insert("number_pagers", to_value(self.pagers.len()).unwrap());
        let base_url = if self.paginate_path.is_empty() {
            self.permalink.to_string()
        } else {
            format!("{}{}/", self.permalink, self.paginate_path)
        };
        paginator.insert("base_url", to_value(base_url).unwrap());
        paginator.insert("pages", to_value(&current_pager.pages).unwrap());
        paginator.insert("current_index", to_value(current_pager.index).unwrap());
        paginator.insert("total_pages", to_value(self.all_pages.len()).unwrap());

        paginator
    }

    pub fn render_pager(
        &self,
        pager: &Pager,
        config: &Config,
        tera: &Tera,
        library: &Library,
    ) -> Result<String> {
        let mut context = Context::new();
        match self.root {
            PaginationRoot::Section(s) => {
                context.insert(
                    "section",
                    &SerializingSection::new(s, SectionSerMode::MetadataOnly(library)),
                );
                context.insert("lang", &s.lang);
                context.insert("config", &config.serialize(&s.lang));
            }
            PaginationRoot::Taxonomy(t, item) => {
                context.insert("taxonomy", &t.kind);
                context.insert("term", &item.serialize(library));
                context.insert("lang", &t.lang);
                context.insert("config", &config.serialize(&t.lang));
            }
        };
        context.insert("current_url", &pager.permalink);
        context.insert("current_path", &pager.path);
        context.insert("paginator", &self.build_paginator_context(pager));

        render_template(&self.template, tera, context, &config.theme)
            .with_context(|| format!("Failed to render pager {}", pager.index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Page, SectionFrontMatter};
    use config::TaxonomyConfig;

    fn create_section(is_index: bool, paginate_reversed: bool) -> Section {
        let f = SectionFrontMatter {
            paginate_by: Some(2),
            paginate_path: "page".to_string(),
            paginate_reversed,
            ..Default::default()
        };

        let mut s = Section::new("content/_index.md", f, &PathBuf::new());
        if !is_index {
            s.path = "/posts/".to_string();
            s.permalink = "https://vincent.is/posts/".to_string();
            s.file.path = PathBuf::from("posts/_index.md");
            s.file.components = vec!["posts".to_string()];
        } else {
            s.path = "/".into();
            s.file.path = PathBuf::from("_index.md");
            s.permalink = "https://vincent.is/".to_string();
        }
        s
    }

    fn create_library(
        is_index: bool,
        num_pages: usize,
        paginate_reversed: bool,
    ) -> (Section, Library) {
        let mut library = Library::default();
        for i in 1..=num_pages {
            let mut page = Page::default();
            page.meta.title = Some(i.to_string());
            page.file.path = PathBuf::from(&format!("{}.md", i));
            library.insert_page(page);
        }

        let mut section = create_section(is_index, paginate_reversed);
        section.pages = library.pages.keys().cloned().collect();
        section.pages.sort();
        library.insert_section(section.clone());

        (section, library)
    }

    #[test]
    fn test_can_create_section_paginator() {
        let (section, library) = create_library(false, 3, false);
        let paginator = Paginator::from_section(&section, &library);
        assert_eq!(paginator.pagers.len(), 2);

        assert_eq!(paginator.pagers[0].index, 1);
        assert_eq!(paginator.pagers[0].pages.len(), 2);
        assert_eq!(paginator.pagers[0].pages[0].title.clone().unwrap(), "1");
        assert_eq!(paginator.pagers[0].pages[1].title.clone().unwrap(), "2");
        assert_eq!(paginator.pagers[0].permalink, "https://vincent.is/posts/");
        assert_eq!(paginator.pagers[0].path, "/posts/");

        assert_eq!(paginator.pagers[1].index, 2);
        assert_eq!(paginator.pagers[1].pages.len(), 1);
        assert_eq!(paginator.pagers[1].pages[0].title.clone().unwrap(), "3");
        assert_eq!(paginator.pagers[1].permalink, "https://vincent.is/posts/page/2/");
        assert_eq!(paginator.pagers[1].path, "/posts/page/2/");
    }

    #[test]
    fn test_can_create_reversed_section_paginator() {
        let (section, library) = create_library(false, 3, true);
        let paginator = Paginator::from_section(&section, &library);
        assert_eq!(paginator.pagers.len(), 2);

        assert_eq!(paginator.pagers[0].index, 1);
        assert_eq!(paginator.pagers[0].pages.len(), 2);
        assert_eq!(paginator.pagers[0].pages[0].title.clone().unwrap(), "3");
        assert_eq!(paginator.pagers[0].pages[1].title.clone().unwrap(), "2");
        assert_eq!(paginator.pagers[0].permalink, "https://vincent.is/posts/");
        assert_eq!(paginator.pagers[0].path, "/posts/");

        assert_eq!(paginator.pagers[1].index, 2);
        assert_eq!(paginator.pagers[1].pages.len(), 1);
        assert_eq!(paginator.pagers[1].pages[0].title.clone().unwrap(), "1");
        assert_eq!(paginator.pagers[1].permalink, "https://vincent.is/posts/page/2/");
        assert_eq!(paginator.pagers[1].path, "/posts/page/2/");
    }

    #[test]
    fn can_create_paginator_for_index() {
        let (section, library) = create_library(true, 3, false);
        let paginator = Paginator::from_section(&section, &library);
        assert_eq!(paginator.pagers.len(), 2);

        assert_eq!(paginator.pagers[0].index, 1);
        assert_eq!(paginator.pagers[0].pages.len(), 2);
        assert_eq!(paginator.pagers[0].permalink, "https://vincent.is/");
        assert_eq!(paginator.pagers[0].path, "/");

        assert_eq!(paginator.pagers[1].index, 2);
        assert_eq!(paginator.pagers[1].pages.len(), 1);
        assert_eq!(paginator.pagers[1].permalink, "https://vincent.is/page/2/");
        assert_eq!(paginator.pagers[1].path, "/page/2/");
    }

    #[test]
    fn test_can_build_paginator_context() {
        let (section, library) = create_library(false, 3, false);
        let paginator = Paginator::from_section(&section, &library);
        assert_eq!(paginator.pagers.len(), 2);

        let context = paginator.build_paginator_context(&paginator.pagers[0]);
        assert_eq!(context["paginate_by"], to_value(2).unwrap());
        assert_eq!(context["first"], to_value("https://vincent.is/posts/").unwrap());
        assert_eq!(context["last"], to_value("https://vincent.is/posts/page/2/").unwrap());
        assert_eq!(context["previous"], to_value::<Option<()>>(None).unwrap());
        assert_eq!(context["next"], to_value("https://vincent.is/posts/page/2/").unwrap());
        assert_eq!(context["current_index"], to_value(1).unwrap());
        assert_eq!(context["pages"].as_array().unwrap().len(), 2);

        let context = paginator.build_paginator_context(&paginator.pagers[1]);
        assert_eq!(context["paginate_by"], to_value(2).unwrap());
        assert_eq!(context["first"], to_value("https://vincent.is/posts/").unwrap());
        assert_eq!(context["last"], to_value("https://vincent.is/posts/page/2/").unwrap());
        assert_eq!(context["next"], to_value::<Option<()>>(None).unwrap());
        assert_eq!(context["previous"], to_value("https://vincent.is/posts/").unwrap());
        assert_eq!(context["current_index"], to_value(2).unwrap());
        assert_eq!(context["total_pages"], to_value(3).unwrap());
        assert_eq!(context["pages"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_can_create_paginator_for_taxonomy() {
        let (_, library) = create_library(false, 3, false);
        let tera = Tera::default();
        let taxonomy_def = TaxonomyConfig {
            name: "some tags".to_string(),
            paginate_by: Some(2),
            ..TaxonomyConfig::default()
        };
        let taxonomy_item = TaxonomyTerm {
            name: "Something".to_string(),
            slug: "something".to_string(),
            path: "/some-tags/something/".to_string(),
            permalink: "https://vincent.is/some-tags/something/".to_string(),
            pages: library.pages.keys().cloned().collect(),
        };
        let taxonomy = Taxonomy {
            kind: taxonomy_def,
            lang: "en".to_owned(),
            slug: "some-tags".to_string(),
            path: "/some-tags/".to_string(),
            permalink: "https://vincent.is/some-tags/".to_string(),
            items: vec![taxonomy_item.clone()],
        };
        let paginator = Paginator::from_taxonomy(&taxonomy, &taxonomy_item, &library, &tera, &None);
        assert_eq!(paginator.pagers.len(), 2);

        assert_eq!(paginator.pagers[0].index, 1);
        assert_eq!(paginator.pagers[0].pages.len(), 2);
        assert_eq!(paginator.pagers[0].permalink, "https://vincent.is/some-tags/something/");
        assert_eq!(paginator.pagers[0].path, "/some-tags/something/");

        assert_eq!(paginator.pagers[1].index, 2);
        assert_eq!(paginator.pagers[1].pages.len(), 1);
        assert_eq!(paginator.pagers[1].permalink, "https://vincent.is/some-tags/something/page/2/");
        assert_eq!(paginator.pagers[1].path, "/some-tags/something/page/2/");
    }

    // https://github.com/getzola/zola/issues/866
    #[test]
    fn works_with_empty_paginate_path() {
        let (mut section, library) = create_library(false, 3, false);
        section.meta.paginate_path = String::new();
        let paginator = Paginator::from_section(&section, &library);
        assert_eq!(paginator.pagers.len(), 2);

        assert_eq!(paginator.pagers[0].index, 1);
        assert_eq!(paginator.pagers[0].pages.len(), 2);
        assert_eq!(paginator.pagers[0].permalink, "https://vincent.is/posts/");
        assert_eq!(paginator.pagers[0].path, "/posts/");

        assert_eq!(paginator.pagers[1].index, 2);
        assert_eq!(paginator.pagers[1].pages.len(), 1);
        assert_eq!(paginator.pagers[1].permalink, "https://vincent.is/posts/2/");
        assert_eq!(paginator.pagers[1].path, "/posts/2/");

        let context = paginator.build_paginator_context(&paginator.pagers[0]);
        assert_eq!(context["base_url"], to_value("https://vincent.is/posts/").unwrap());
    }
}
