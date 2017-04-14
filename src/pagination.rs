use std::collections::HashMap;
use tera::{Tera, Context, to_value};

use config::Config;
use errors::{Result, ResultExt};
use page::Page;
use section::Section;


/// A list of all the pages in the paginator with their index and links
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Pager {
    /// The page number in the paginator (1-indexed)
    index: usize,
    /// Permalink to that page
    permalink: String,
    /// Path to that page
    path: String,
}

impl Pager {
    fn new(index: usize, permalink: String, path: String) -> Pager {
        Pager {
            index: index,
            permalink: permalink,
            path: path,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Paginator<'a> {
    /// All pages in the section
    all_pages: &'a [Page],
    /// Pages split in chunks of `paginate_by`
    pub pages: Vec<Vec<&'a Page>>,
    /// How many content pages on a paginated page at max
    paginate_by: usize,
}

impl<'a> Paginator<'a> {
    pub fn new(all_pages: &'a [Page], paginate_by: usize) -> Paginator<'a> {
        let mut pages = vec![];
        let mut current_page = vec![];

        for page in all_pages {
            current_page.push(page);

            if current_page.len() == paginate_by {
                pages.push(current_page);
                current_page = vec![];
            }
        }
        if !current_page.is_empty() {
            pages.push(current_page);
        }

        Paginator {
            all_pages: all_pages,
            pages: pages,
            paginate_by: paginate_by,
        }
    }

    /// Take a section and return (permalink, path) for the paginated pages in it
    pub fn get_paginated_list(&self, config: &Config, section: &Section) -> Vec<Pager> {
        let paginate_path = match config.paginate_path {
            Some(ref p) => p,
            None => unreachable!(),
        };

        let mut paginated = vec![];
        for index in 0..self.pages.len() {
            if index > 0 {
                let page_path = format!("{}/{}", paginate_path, index + 1);
                paginated.push(Pager::new(
                    index + 1,
                    format!("{}/{}", section.permalink, page_path),
                    format!("{}/{}", section.path, page_path)
                ));
            } else {
                // index = 0 -> first page, no paginate path used
                paginated.push(Pager::new(1, section.permalink.clone(), section.path.clone()));
            }
        }
        paginated
    }

    pub fn render_section_page(&self, index: usize, section: &Section, tera: &Tera, config: &Config) -> Result<String> {
        let tpl_name = section.get_template_name();
        let pages = &self.pages[index];
        let all_paginated_pages = self.get_paginated_list(config, section);

        let mut context = Context::new();
        context.add("config", config);
        context.add("section", section);
        let paginated = &all_paginated_pages[index];
        context.add("current_url", &paginated.permalink);
        context.add("current_path", &paginated.path);

        let mut paginator = HashMap::new();
        // Global variables
        paginator.insert("paginate_by", to_value(self.paginate_by).unwrap());
        paginator.insert("first", to_value(&section.permalink).unwrap());
        let last_paginated = &all_paginated_pages[self.pages.len() - 1];
        paginator.insert("last", to_value(&last_paginated.permalink).unwrap());
        paginator.insert("pagers", to_value(&all_paginated_pages).unwrap());

        // Variables for this specific page
        if index > 0 {
            let prev_paginated = &all_paginated_pages[index - 1];
            paginator.insert("prev", to_value(&prev_paginated.permalink).unwrap());
        } else {
            paginator.insert("prev", to_value::<Option<()>>(None).unwrap());
        }
        if index < self.pages.len() - 1 {
            let next_paginated = &all_paginated_pages[index + 1];
            paginator.insert("next", to_value(&next_paginated.permalink).unwrap());
        } else {
            paginator.insert("next", to_value::<Option<()>>(None).unwrap());
        }
        paginator.insert("pages", to_value(pages).unwrap());
        paginator.insert("current_index", to_value(index + 1).unwrap());
        context.add("paginator", &paginator);

        tera.render(&tpl_name, &context)
            .chain_err(|| format!("Failed to render page {} of section '{}'", index, section.file_path.display()))
    }
}
