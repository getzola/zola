use std::collections::HashMap;
use tera::{Context, to_value, Value};

use errors::{Result, ResultExt};
use page::Page;
use section::Section;
use site::Site;


/// A list of all the pages in the paginator with their index and links
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Pager<'a> {
    /// The page number in the paginator (1-indexed)
    index: usize,
    /// Permalink to that page
    permalink: String,
    /// Path to that page
    path: String,
    /// All pages for the pager
    pages: Vec<&'a Page>
}

impl<'a> Pager<'a> {
    fn new(index: usize, pages: Vec<&'a Page>, permalink: String, path: String) -> Pager<'a> {
        Pager {
            index: index,
            permalink: permalink,
            path: path,
            pages: pages,
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
    /// The section struct we're building the paginator for
    section: &'a Section,
}

impl<'a> Paginator<'a> {
    pub fn new(all_pages: &'a [Page], section: &'a Section) -> Paginator<'a> {
        let paginate_by = section.meta.paginate_by.unwrap();
        let paginate_path = match section.meta.paginate_path {
            Some(ref p) => p,
            None => unreachable!(),
        };

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

        let mut pagers = vec![];
        for index in 0..pages.len() {
            // First page has no pagination path
            if index == 0 {
                pagers.push(Pager::new(1, pages[index].clone(), section.permalink.clone(), section.path.clone()));
                continue;
            }

            let page_path = format!("{}/{}", paginate_path, index + 1);
            let permalink = if section.permalink.ends_with('/') {
                format!("{}{}", section.permalink, page_path)
            } else {
                format!("{}/{}", section.permalink, page_path)
            };
            pagers.push(Pager::new(
                index + 1,
                pages[index].clone(),
                permalink,
                if section.is_index() { format!("{}", page_path) } else { format!("{}/{}", section.path, page_path) }
            ));
        }

        //println!("{:?}", pagers);

        Paginator {
            all_pages: all_pages,
            pagers: pagers,
            paginate_by: paginate_by,
            section: section,
        }
    }

    pub fn build_paginator_context(&self, current_pager: &Pager) -> HashMap<&str, Value> {
        let mut paginator = HashMap::new();
        // the pager index is 1-indexed so we want a 0-indexed one for indexing there
        let pager_index = current_pager.index - 1;

        // Global variables
        paginator.insert("paginate_by", to_value(self.paginate_by).unwrap());
        paginator.insert("first", to_value(&self.section.permalink).unwrap());
        let last_pager = &self.pagers[self.pagers.len() - 1];
        paginator.insert("last", to_value(&last_pager.permalink).unwrap());
        paginator.insert("pagers", to_value(&self.pagers).unwrap());

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

    pub fn render_pager(&self, pager: &Pager, site: &Site) -> Result<String> {
        let mut context = Context::new();
        context.add("config", &site.config);
        context.add("section", self.section);
        context.add("current_url", &pager.permalink);
        context.add("current_path", &pager.path);
        context.add("paginator", &self.build_paginator_context(pager));
        if self.section.is_index() {
            context.add("section", &site.sections);
        }

        site.tera.render(&self.section.get_template_name(), &context)
            .chain_err(|| format!("Failed to render pager {} of section '{}'", pager.index, self.section.file_path.display()))
    }
}

#[cfg(test)]
mod tests {
    use tera::{to_value};

    use front_matter::FrontMatter;
    use page::Page;
    use section::Section;

    use super::{Paginator};

    fn create_section(is_index: bool) -> Section {
        let mut f = FrontMatter::default();
        f.paginate_by = Some(2);
        f.paginate_path = Some("page".to_string());
        let mut s = Section::new("content/_index.md", f);
        if !is_index {
            s.path = "posts".to_string();
            s.permalink = "https://vincent.is/posts".to_string();
            s.components = vec!["posts".to_string()];
        } else {
            s.permalink = "https://vincent.is".to_string();
        }
        s
    }

    #[test]
    fn test_can_create_paginator() {
        let pages = vec![
            Page::new(FrontMatter::default()),
            Page::new(FrontMatter::default()),
            Page::new(FrontMatter::default()),
        ];
        let section = create_section(false);
        let paginator = Paginator::new(pages.as_slice(), &section);
        assert_eq!(paginator.pagers.len(), 2);

        assert_eq!(paginator.pagers[0].index, 1);
        assert_eq!(paginator.pagers[0].pages.len(), 2);
        assert_eq!(paginator.pagers[0].permalink, "https://vincent.is/posts");
        assert_eq!(paginator.pagers[0].path, "posts");

        assert_eq!(paginator.pagers[1].index, 2);
        assert_eq!(paginator.pagers[1].pages.len(), 1);
        assert_eq!(paginator.pagers[1].permalink, "https://vincent.is/posts/page/2");
        assert_eq!(paginator.pagers[1].path, "posts/page/2");
    }

    #[test]
    fn test_can_create_paginator_for_index() {
        let pages = vec![
            Page::new(FrontMatter::default()),
            Page::new(FrontMatter::default()),
            Page::new(FrontMatter::default()),
        ];
        let section = create_section(true);
        let paginator = Paginator::new(pages.as_slice(), &section);
        assert_eq!(paginator.pagers.len(), 2);

        assert_eq!(paginator.pagers[0].index, 1);
        assert_eq!(paginator.pagers[0].pages.len(), 2);
        assert_eq!(paginator.pagers[0].permalink, "https://vincent.is");
        assert_eq!(paginator.pagers[0].path, "");

        assert_eq!(paginator.pagers[1].index, 2);
        assert_eq!(paginator.pagers[1].pages.len(), 1);
        assert_eq!(paginator.pagers[1].permalink, "https://vincent.is/page/2");
        assert_eq!(paginator.pagers[1].path, "page/2");
    }

    #[test]
    fn test_can_build_paginator_context() {
        let pages = vec![
            Page::new(FrontMatter::default()),
            Page::new(FrontMatter::default()),
            Page::new(FrontMatter::default()),
        ];
        let section = create_section(false);
        let paginator = Paginator::new(pages.as_slice(), &section);
        assert_eq!(paginator.pagers.len(), 2);

        let context = paginator.build_paginator_context(&paginator.pagers[0]);
        assert_eq!(context["paginate_by"], to_value(2).unwrap());
        assert_eq!(context["first"], to_value("https://vincent.is/posts").unwrap());
        assert_eq!(context["last"], to_value("https://vincent.is/posts/page/2").unwrap());
        assert_eq!(context["previous"], to_value::<Option<()>>(None).unwrap());
        assert_eq!(context["next"], to_value("https://vincent.is/posts/page/2").unwrap());
        assert_eq!(context["current_index"], to_value(1).unwrap());

        let context = paginator.build_paginator_context(&paginator.pagers[1]);
        assert_eq!(context["paginate_by"], to_value(2).unwrap());
        assert_eq!(context["first"], to_value("https://vincent.is/posts").unwrap());
        assert_eq!(context["last"], to_value("https://vincent.is/posts/page/2").unwrap());
        assert_eq!(context["next"], to_value::<Option<()>>(None).unwrap());
        assert_eq!(context["previous"], to_value("https://vincent.is/posts").unwrap());
        assert_eq!(context["current_index"], to_value(2).unwrap());
    }
}
