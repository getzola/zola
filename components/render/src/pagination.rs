use crate::RenderCache;
use content::{Library, Section, Taxonomy, TaxonomyTerm};
use serde::Serialize;
use std::path::PathBuf;
use tera::{Map, Tera, Value};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PaginationRoot<'a> {
    Section(&'a Section),
    Taxonomy(&'a Taxonomy, &'a TaxonomyTerm),
}

/// A list of all the pages in the paginator with their index and links
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Pager {
    /// The page number in the paginator (1-indexed)
    pub index: usize,
    /// Permalink to that page
    pub permalink: String,
    /// Path to that page
    pub path: String,
    /// Indices into all_pages for this pager
    pub page_indices: Vec<usize>,
}

impl Pager {
    fn new(index: usize, page_indices: Vec<usize>, permalink: String, path: String) -> Pager {
        Pager { index, permalink, path, page_indices }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paginator<'a> {
    /// All pages in the section/taxonomy
    all_pages: &'a [PathBuf],
    /// Pages split in chunks of `paginate_by`
    pub pagers: Vec<Pager>,
    /// How many content pages on a paginated page at max
    paginate_by: usize,
    /// whether to reverse before grouping
    paginate_reversed: bool,
    /// The thing we are creating the paginator for: section or taxonomy
    pub(crate) root: PaginationRoot<'a>,
    // Those below can be obtained from the root but it would make the code more complex than needed
    pub permalink: String,
    path: String,
    pub paginate_path: String,
    pub template: String,
    /// Whether this is the index section, we need it for the template name
    is_index: bool,
    /// Total number of rendered pages
    total_pages: usize,
}

impl<'a> Paginator<'a> {
    /// Create a new paginator from a section
    /// It will always at least create one pager (the first) even if there are not enough pages to paginate
    pub fn from_section(section: &'a Section, library: &'a Library) -> Paginator<'a> {
        let paginate_by = section.meta.paginate_by.unwrap();
        let mut paginator = Paginator {
            all_pages: &section.pages,
            pagers: Vec::with_capacity((section.pages.len() / paginate_by) + 1),
            paginate_by,
            paginate_reversed: section.meta.paginate_reversed,
            root: PaginationRoot::Section(section),
            permalink: section.permalink.clone(),
            path: section.path.clone(),
            paginate_path: section.meta.paginate_path.clone(),
            is_index: section.is_index(),
            template: section.get_template_name().to_string(),
            total_pages: 0, // filled by fill_pagers()
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
    ) -> Paginator<'a> {
        let paginate_by = taxonomy.kind.paginate_by.unwrap();
        // Check for taxon-specific template, or use generic as fallback.
        let specific_template = format!("{}/single.html", taxonomy.kind.name);
        let template = if tera.get_template(&specific_template).is_some() {
            specific_template
        } else {
            "taxonomy_single.html".to_string()
        };
        let mut paginator = Paginator {
            all_pages: &item.pages,
            pagers: Vec::with_capacity((item.pages.len() / paginate_by) + 1),
            paginate_by,
            paginate_reversed: false,
            root: PaginationRoot::Taxonomy(taxonomy, item),
            permalink: item.permalink.clone(),
            path: item.path.clone(),
            paginate_path: taxonomy.kind.paginate_path().to_owned(),
            is_index: false,
            template,
            total_pages: 0, // filled by fill_pagers()
        };

        // taxonomy paginators have no sorting so we won't have to reverse
        paginator.fill_pagers(library);
        paginator
    }

    fn fill_pagers(&mut self, library: &'a Library) {
        // Collect indices of rendered pages
        let mut rendered_indices: Vec<usize> = self
            .all_pages
            .iter()
            .enumerate()
            .filter(|(_, p)| library.pages[*p].meta.render)
            .map(|(i, _)| i)
            .collect();

        // Reverse if needed
        if self.paginate_reversed {
            rendered_indices.reverse();
        }

        // Build pagers - each pager gets a slice of indices
        let path_base = if self.is_index {
            "/".to_string()
        } else if self.path.ends_with('/') {
            self.path.clone()
        } else {
            format!("{}/", self.path)
        };

        let mut pagers = vec![];
        for (pager_index, chunk) in rendered_indices.chunks(self.paginate_by).enumerate() {
            let page_indices = chunk.to_vec();

            if pager_index == 0 {
                pagers.push(Pager::new(1, page_indices, self.permalink.clone(), self.path.clone()));
            } else {
                let permalink = self.build_paginate_url(&self.permalink, Some(pager_index + 1));
                let pager_path = self.build_paginate_url(&path_base, Some(pager_index + 1));

                pagers.push(Pager::new(pager_index + 1, page_indices, permalink, pager_path));
            }
        }

        // Handle empty case
        if pagers.is_empty() {
            pagers.push(Pager::new(1, vec![], self.permalink.clone(), self.path.clone()));
        }

        self.total_pages = rendered_indices.len();
        self.pagers = pagers;
    }

    fn build_paginate_url(&self, base: &str, page_number: Option<usize>) -> String {
        match (self.paginate_path.is_empty(), page_number) {
            (true, None) => base.to_string(),
            (true, Some(n)) => format!("{}{}/", base, n),
            (false, None) => format!("{}{}/", base, self.paginate_path),
            (false, Some(n)) => format!("{}{}/{}/", base, self.paginate_path, n),
        }
    }

    pub fn build_context(&self, current_pager: &Pager, cache: &RenderCache) -> Value {
        let mut map = Map::new();
        // the pager index is 1-indexed so we want a 0-indexed one for indexing there
        let pager_index = current_pager.index - 1;

        // Global variables
        map.insert("paginate_by".into(), Value::from(self.paginate_by));
        map.insert("first".into(), Value::from(self.pagers[0].permalink.as_str()));
        map.insert("last".into(), Value::from(self.pagers.last().unwrap().permalink.as_str()));
        map.insert("total_pages".into(), Value::from(self.total_pages));
        map.insert("number_pagers".into(), Value::from(self.pagers.len()));

        // For this pager
        map.insert("current_index".into(), Value::from(current_pager.index));
        let pages: Vec<Value> = current_pager
            .page_indices
            .iter()
            .filter_map(|&idx| cache.pages.get(&self.all_pages[idx]).map(|c| c.value.clone()))
            .collect();
        map.insert("pages".into(), Value::from(pages));

        // Variables for this specific page
        if pager_index > 0 {
            let prev_pager = &self.pagers[pager_index - 1];
            map.insert("previous".into(), Value::from(prev_pager.permalink.as_str()));
        } else {
            map.insert("previous".into(), Value::null());
        }

        if pager_index < self.pagers.len() - 1 {
            let next_pager = &self.pagers[pager_index + 1];
            map.insert("next".into(), Value::from(next_pager.permalink.as_str()));
        } else {
            map.insert("next".into(), Value::null());
        }
        let base_url = self.build_paginate_url(&self.permalink, None);
        map.insert("base_url".into(), Value::from(base_url));

        Value::from(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::TaxonomyConfig;
    use content::{Page, SectionFrontMatter};

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
        assert_eq!(paginator.pagers[0].page_indices.len(), 2);
        let page_path = &paginator.all_pages[paginator.pagers[0].page_indices[0]];
        let page = &library.pages[page_path];
        assert_eq!(page.meta.title.clone().unwrap(), "1");
        let page_path = &paginator.all_pages[paginator.pagers[0].page_indices[1]];
        let page = &library.pages[page_path];
        assert_eq!(page.meta.title.clone().unwrap(), "2");
        assert_eq!(paginator.pagers[0].permalink, "https://vincent.is/posts/");
        assert_eq!(paginator.pagers[0].path, "/posts/");

        assert_eq!(paginator.pagers[1].index, 2);
        assert_eq!(paginator.pagers[1].page_indices.len(), 1);
        let page_path = &paginator.all_pages[paginator.pagers[1].page_indices[0]];
        let page = &library.pages[page_path];
        assert_eq!(page.meta.title.clone().unwrap(), "3");
        assert_eq!(paginator.pagers[1].permalink, "https://vincent.is/posts/page/2/");
        assert_eq!(paginator.pagers[1].path, "/posts/page/2/");
    }

    #[test]
    fn test_can_create_reversed_section_paginator() {
        let (section, library) = create_library(false, 3, true);
        let paginator = Paginator::from_section(&section, &library);
        assert_eq!(paginator.pagers.len(), 2);

        assert_eq!(paginator.pagers[0].index, 1);
        assert_eq!(paginator.pagers[0].page_indices.len(), 2);
        let page_path = &paginator.all_pages[paginator.pagers[0].page_indices[0]];
        let page = &library.pages[page_path];
        assert_eq!(page.meta.title.clone().unwrap(), "3");
        let page_path = &paginator.all_pages[paginator.pagers[0].page_indices[1]];
        let page = &library.pages[page_path];
        assert_eq!(page.meta.title.clone().unwrap(), "2");
        assert_eq!(paginator.pagers[0].permalink, "https://vincent.is/posts/");
        assert_eq!(paginator.pagers[0].path, "/posts/");

        assert_eq!(paginator.pagers[1].index, 2);
        assert_eq!(paginator.pagers[1].page_indices.len(), 1);
        let page_path = &paginator.all_pages[paginator.pagers[1].page_indices[0]];
        let page = &library.pages[page_path];
        assert_eq!(page.meta.title.clone().unwrap(), "1");
        assert_eq!(paginator.pagers[1].permalink, "https://vincent.is/posts/page/2/");
        assert_eq!(paginator.pagers[1].path, "/posts/page/2/");
    }

    #[test]
    fn can_create_paginator_for_index() {
        let (section, library) = create_library(true, 3, false);
        let paginator = Paginator::from_section(&section, &library);
        assert_eq!(paginator.pagers.len(), 2);

        assert_eq!(paginator.pagers[0].index, 1);
        assert_eq!(paginator.pagers[0].page_indices.len(), 2);
        assert_eq!(paginator.pagers[0].permalink, "https://vincent.is/");
        assert_eq!(paginator.pagers[0].path, "/");

        assert_eq!(paginator.pagers[1].index, 2);
        assert_eq!(paginator.pagers[1].page_indices.len(), 1);
        assert_eq!(paginator.pagers[1].permalink, "https://vincent.is/page/2/");
        assert_eq!(paginator.pagers[1].path, "/page/2/");
    }

    #[test]
    fn test_can_build_paginator_context() {
        let (section, library) = create_library(false, 3, false);
        let paginator = Paginator::from_section(&section, &library);
        assert_eq!(paginator.pagers.len(), 2);

        let config = config::Config::default();
        let tera = Tera::default();
        let cache = RenderCache::build(&config, &library, &[], &tera);

        let context = paginator.build_context(&paginator.pagers[0], &cache);
        let context = context.as_map().unwrap();
        assert_eq!(context.get(&"paginate_by".into()).unwrap(), &Value::from(2));
        assert_eq!(
            context.get(&"first".into()).unwrap(),
            &Value::from("https://vincent.is/posts/")
        );
        assert_eq!(
            context.get(&"last".into()).unwrap(),
            &Value::from("https://vincent.is/posts/page/2/")
        );
        assert_eq!(context.get(&"previous".into()).unwrap(), &Value::null());
        assert_eq!(
            context.get(&"next".into()).unwrap(),
            &Value::from("https://vincent.is/posts/page/2/")
        );
        assert_eq!(context.get(&"current_index".into()).unwrap(), &Value::from(1));
        assert_eq!(context.get(&"pages".into()).unwrap().as_vec().unwrap().len(), 2);

        let context = paginator.build_context(&paginator.pagers[1], &cache);
        let context = context.as_map().unwrap();
        assert_eq!(context.get(&"paginate_by".into()).unwrap(), &Value::from(2));
        assert_eq!(
            context.get(&"first".into()).unwrap(),
            &Value::from("https://vincent.is/posts/")
        );
        assert_eq!(
            context.get(&"last".into()).unwrap(),
            &Value::from("https://vincent.is/posts/page/2/")
        );
        assert_eq!(context.get(&"next".into()).unwrap(), &Value::null());
        assert_eq!(
            context.get(&"previous".into()).unwrap(),
            &Value::from("https://vincent.is/posts/")
        );
        assert_eq!(context.get(&"current_index".into()).unwrap(), &Value::from(2));
        assert_eq!(context.get(&"total_pages".into()).unwrap(), &Value::from(3));
        assert_eq!(context.get(&"pages".into()).unwrap().as_vec().unwrap().len(), 1);
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
        let paginator = Paginator::from_taxonomy(&taxonomy, &taxonomy_item, &library, &tera);
        assert_eq!(paginator.pagers.len(), 2);

        assert_eq!(paginator.pagers[0].index, 1);
        assert_eq!(paginator.pagers[0].page_indices.len(), 2);
        assert_eq!(paginator.pagers[0].permalink, "https://vincent.is/some-tags/something/");
        assert_eq!(paginator.pagers[0].path, "/some-tags/something/");

        assert_eq!(paginator.pagers[1].index, 2);
        assert_eq!(paginator.pagers[1].page_indices.len(), 1);
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
        assert_eq!(paginator.pagers[0].page_indices.len(), 2);
        assert_eq!(paginator.pagers[0].permalink, "https://vincent.is/posts/");
        assert_eq!(paginator.pagers[0].path, "/posts/");

        assert_eq!(paginator.pagers[1].index, 2);
        assert_eq!(paginator.pagers[1].page_indices.len(), 1);
        assert_eq!(paginator.pagers[1].permalink, "https://vincent.is/posts/2/");
        assert_eq!(paginator.pagers[1].path, "/posts/2/");

        let config = config::Config::default();
        let tera = Tera::default();
        let cache = RenderCache::build(&config, &library, &[], &tera);
        let context = paginator.build_context(&paginator.pagers[0], &cache);
        let context = context.as_map().unwrap();
        assert_eq!(
            context.get(&"base_url".into()).unwrap(),
            &Value::from("https://vincent.is/posts/")
        );
    }

    #[test]
    fn test_paginator_skips_non_rendered_pages() {
        let mut library = Library::default();
        for i in 1..=5 {
            let mut page = Page::default();
            page.meta.title = Some(i.to_string());
            page.file.path = PathBuf::from(&format!("{}.md", i));
            if i == 2 || i == 4 {
                page.meta.render = false;
            }
            library.insert_page(page);
        }

        let mut section = create_section(false, false);
        section.pages = library.pages.keys().cloned().collect();
        section.pages.sort();
        library.insert_section(section.clone());

        let paginator = Paginator::from_section(&section, &library);

        // Only 3 rendered pages with paginate_by=2 → 2 pagers
        assert_eq!(paginator.pagers.len(), 2);
        assert_eq!(paginator.pagers[0].page_indices.len(), 2);
        assert_eq!(paginator.pagers[1].page_indices.len(), 1);

        // Verify total_pages is correct (only rendered)
        let config = config::Config::default();
        let tera = Tera::default();
        let cache = RenderCache::build(&config, &library, &[], &tera);
        let context = paginator.build_context(&paginator.pagers[0], &cache);
        let context = context.as_map().unwrap();
        assert_eq!(context.get(&"total_pages".into()).unwrap(), &Value::from(3));
    }

    #[test]
    fn test_paginator_with_all_non_rendered_pages() {
        let mut library = Library::default();
        for i in 1..=3 {
            let mut page = Page::default();
            page.meta.title = Some(i.to_string());
            page.file.path = PathBuf::from(&format!("{}.md", i));
            page.meta.render = false;
            library.insert_page(page);
        }

        let mut section = create_section(false, false);
        section.pages = library.pages.keys().cloned().collect();
        library.insert_section(section.clone());

        let paginator = Paginator::from_section(&section, &library);

        assert_eq!(paginator.pagers.len(), 1);
        assert_eq!(paginator.pagers[0].page_indices.len(), 0);
    }
}
