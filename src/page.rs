/// A page, can be a blog post or a basic page
use std::collections::HashMap;
use std::fs::{read_dir};
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;


use tera::{Tera, Context};
use serde::ser::{SerializeStruct, self};
use slug::slugify;

use errors::{Result, ResultExt};
use config::Config;
use front_matter::{PageFrontMatter, SortBy, split_page_content};
use markdown::markdown_to_html;
use utils::{read_file, find_content_components};



/// Looks into the current folder for the path and see if there's anything that is not a .md
/// file. Those will be copied next to the rendered .html file
fn find_related_assets(path: &Path) -> Vec<PathBuf> {
    let mut assets = vec![];

    for entry in read_dir(path).unwrap().filter_map(|e| e.ok()) {
        let entry_path = entry.path();
        if entry_path.is_file() {
            match entry_path.extension() {
                Some(e) => match e.to_str() {
                    Some("md") => continue,
                    _ => assets.push(entry_path.to_path_buf()),
                },
                None => continue,
            }
        }
    }

    assets
}


#[derive(Clone, Debug, PartialEq)]
pub struct Page {
    /// The front matter meta-data
    pub meta: PageFrontMatter,
    /// The .md path
    pub file_path: PathBuf,
    /// The .md path, starting from the content directory, with / slashes
    pub relative_path: String,
    /// The parent directory of the file. Is actually the grand parent directory
    /// if it's an asset folder
    pub parent_path: PathBuf,
    /// The name of the .md file
    pub file_name: String,
    /// The directories above our .md file
    /// for example a file at content/kb/solutions/blabla.md will have 2 components:
    /// `kb` and `solutions`
    pub components: Vec<String>,
    /// The actual content of the page, in markdown
    pub raw_content: String,
    /// All the non-md files we found next to the .md file
    pub assets: Vec<PathBuf>,
    /// The HTML rendered of the page
    pub content: String,

    /// The slug of that page.
    /// First tries to find the slug in the meta and defaults to filename otherwise
    pub slug: String,
    /// The URL path of the page
    pub path: String,
    /// The full URL for that page
    pub permalink: String,
    /// The summary for the article, defaults to None
    /// When <!-- more --> is found in the text, will take the content up to that part
    /// as summary
    pub summary: Option<String>,

    /// The previous page, by whatever sorting is used for the index/section
    pub previous: Option<Box<Page>>,
    /// The next page, by whatever sorting is used for the index/section
    pub next: Option<Box<Page>>,
}


impl Page {
    pub fn new(meta: PageFrontMatter) -> Page {
        Page {
            meta: meta,
            file_path: PathBuf::new(),
            relative_path: String::new(),
            parent_path: PathBuf::new(),
            file_name: "".to_string(),
            components: vec![],
            raw_content: "".to_string(),
            assets: vec![],
            content: "".to_string(),
            slug: "".to_string(),
            path: "".to_string(),
            permalink: "".to_string(),
            summary: None,
            previous: None,
            next: None,
        }
    }

    pub fn has_date(&self) -> bool {
        self.meta.date.is_some()
    }

    /// Get word count and estimated reading time
    pub fn get_reading_analytics(&self) -> (usize, usize) {
        // Only works for latin language but good enough for a start
        let word_count: usize = self.raw_content.split_whitespace().count();

        // https://help.medium.com/hc/en-us/articles/214991667-Read-time
        // 275 seems a bit too high though
        (word_count, (word_count / 200))
    }

    /// Parse a page given the content of the .md file
    /// Files without front matter or with invalid front matter are considered
    /// erroneous
    pub fn parse(file_path: &Path, content: &str, config: &Config) -> Result<Page> {
        // 1. separate front matter from content
        let (meta, content) = split_page_content(file_path, content)?;
        let mut page = Page::new(meta);
        page.file_path = file_path.to_path_buf();
        page.parent_path = page.file_path.parent().unwrap().to_path_buf();
        page.raw_content = content;

        let path = Path::new(file_path);
        page.file_name = path.file_stem().unwrap().to_string_lossy().to_string();

        page.slug = {
            if let Some(ref slug) = page.meta.slug {
                slug.trim().to_string()
            } else {
                slugify(page.file_name.clone())
            }
        };
        page.components = find_content_components(&page.file_path);
        page.relative_path = format!("{}/{}.md", page.components.join("/"), page.file_name);

        // 4. Find sections
        // Pages with custom urls exists outside of sections
        let mut path_set = false;
        if let Some(ref u) = page.meta.url {
            page.path = u.trim().to_string();
            path_set = true;
        }

        if !page.components.is_empty() {
            // If we have a folder with an asset, don't consider it as a component
            if page.file_name == "index" {
                page.components.pop();
                // also set parent_path to grandparent instead
                page.parent_path = page.parent_path.parent().unwrap().to_path_buf();
            }
            if !path_set {
                // Don't add a trailing slash to sections
                page.path = format!("{}/{}", page.components.join("/"), page.slug);
            }
        } else if !path_set {
            page.path = page.slug.clone();
        }

        page.permalink = config.make_permalink(&page.path);

        Ok(page)
    }

    /// Read and parse a .md file into a Page struct
    pub fn from_file<P: AsRef<Path>>(path: P, config: &Config) -> Result<Page> {
        let path = path.as_ref();
        let content = read_file(path)?;
        let mut page = Page::parse(path, &content, config)?;
        page.assets = find_related_assets(path.parent().unwrap());

        if !page.assets.is_empty() && page.file_name != "index" {
            bail!("Page `{}` has assets ({:?}) but is not named index.md", path.display(), page.assets);
        }

        Ok(page)

    }

    /// We need access to all pages url to render links relative to content
    /// so that can't happen at the same time as parsing
    pub fn render_markdown(&mut self, permalinks: &HashMap<String, String>, tera: &Tera, config: &Config) -> Result<()> {
        self.content = markdown_to_html(&self.raw_content, permalinks, tera, config)?;

        if self.raw_content.contains("<!-- more -->") {
            self.summary = Some({
                let summary = self.raw_content.splitn(2, "<!-- more -->").collect::<Vec<&str>>()[0];
                markdown_to_html(summary, permalinks, tera, config)?
            })
        }

        Ok(())
    }

    /// Renders the page using the default layout, unless specified in front-matter
    pub fn render_html(&self, tera: &Tera, config: &Config) -> Result<String> {
        let tpl_name = match self.meta.template {
            Some(ref l) => l.to_string(),
            None => "page.html".to_string()
        };

        let mut context = Context::new();
        context.add("config", config);
        context.add("page", self);
        context.add("current_url", &self.permalink);
        context.add("current_path", &self.path);

        tera.render(&tpl_name, &context)
            .chain_err(|| format!("Failed to render page '{}'", self.file_path.display()))
    }
}

impl Default for Page {
    fn default() -> Page {
        Page {
            meta: PageFrontMatter::default(),
            file_path: PathBuf::new(),
            relative_path: String::new(),
            parent_path: PathBuf::new(),
            file_name: "".to_string(),
            components: vec![],
            raw_content: "".to_string(),
            assets: vec![],
            content: "".to_string(),
            slug: "".to_string(),
            path: "".to_string(),
            permalink: "".to_string(),
            summary: None,
            previous: None,
            next: None,
        }
    }
}

impl ser::Serialize for Page {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error> where S: ser::Serializer {
        let mut state = serializer.serialize_struct("page", 16)?;
        state.serialize_field("content", &self.content)?;
        state.serialize_field("title", &self.meta.title)?;
        state.serialize_field("description", &self.meta.description)?;
        state.serialize_field("date", &self.meta.date)?;
        state.serialize_field("slug", &self.slug)?;
        state.serialize_field("path", &format!("/{}", self.path))?;
        state.serialize_field("permalink", &self.permalink)?;
        state.serialize_field("summary", &self.summary)?;
        state.serialize_field("tags", &self.meta.tags)?;
        state.serialize_field("draft", &self.meta.draft)?;
        state.serialize_field("category", &self.meta.category)?;
        state.serialize_field("extra", &self.meta.extra)?;
        let (word_count, reading_time) = self.get_reading_analytics();
        state.serialize_field("word_count", &word_count)?;
        state.serialize_field("reading_time", &reading_time)?;
        state.serialize_field("previous", &self.previous)?;
        state.serialize_field("next", &self.next)?;
        state.end()
    }
}

/// Sort pages using the method for the given section
///
/// Any pages that doesn't have a date when the sorting method is date or order
/// when the sorting method is order will be ignored.
pub fn sort_pages(pages: Vec<Page>, sort_by: SortBy) -> (Vec<Page>, Vec<Page>) {
    match sort_by {
        SortBy::Date => {
            let mut can_be_sorted = vec![];
            let mut cannot_be_sorted = vec![];
            for page in pages {
                if page.meta.date.is_some() {
                    can_be_sorted.push(page);
                } else {
                    cannot_be_sorted.push(page);
                }
            }
            can_be_sorted.sort_by(|a, b| b.meta.date().unwrap().cmp(&a.meta.date().unwrap()));

            (can_be_sorted, cannot_be_sorted)
        },
        SortBy::Order => {
            let mut can_be_sorted = vec![];
            let mut cannot_be_sorted = vec![];
            for page in pages {
                if page.meta.order.is_some() {
                    can_be_sorted.push(page);
                } else {
                    cannot_be_sorted.push(page);
                }
            }
            can_be_sorted.sort_by(|a, b| b.meta.order().cmp(&a.meta.order()));

            (can_be_sorted, cannot_be_sorted)
        },
        SortBy::None => (pages,  vec![])
    }
}

/// Horribly inefficient way to set previous and next on each pages
/// So many clones
pub fn populate_previous_and_next_pages(input: &[Page]) -> Vec<Page> {
    let pages = input.to_vec();
    let mut res = Vec::new();

    // the input is already sorted
    // We might put prev/next randomly if a page is missing date/order, probably fine
    for (i, page) in input.iter().enumerate() {
        let mut new_page = page.clone();

        if i > 0 {
            let next = &pages[i - 1];
            new_page.next = Some(Box::new(next.clone()));
        }

        if i < input.len() - 1 {
            let previous = &pages[i + 1];
            new_page.previous = Some(Box::new(previous.clone()));
        }
        res.push(new_page);
    }

    res
}

#[cfg(test)]
mod tests {
    use tempdir::TempDir;

    use std::fs::File;

    use front_matter::{PageFrontMatter, SortBy};
    use super::{Page, find_related_assets, sort_pages, populate_previous_and_next_pages};

    fn create_page_with_date(date: &str) -> Page {
        let mut front_matter = PageFrontMatter::default();
        front_matter.date = Some(date.to_string());
        Page::new(front_matter)
    }

    fn create_page_with_order(order: usize) -> Page {
        let mut front_matter = PageFrontMatter::default();
        front_matter.order = Some(order);
        Page::new(front_matter)
    }

    #[test]
    fn test_find_related_assets() {
        let tmp_dir = TempDir::new("example").expect("create temp dir");
        File::create(tmp_dir.path().join("index.md")).unwrap();
        File::create(tmp_dir.path().join("example.js")).unwrap();
        File::create(tmp_dir.path().join("graph.jpg")).unwrap();
        File::create(tmp_dir.path().join("fail.png")).unwrap();

        let assets = find_related_assets(tmp_dir.path());
        assert_eq!(assets.len(), 3);
        assert_eq!(assets.iter().filter(|p| p.extension().unwrap() != "md").count(), 3);
        assert_eq!(assets.iter().filter(|p| p.file_name().unwrap() == "example.js").count(), 1);
        assert_eq!(assets.iter().filter(|p| p.file_name().unwrap() == "graph.jpg").count(), 1);
        assert_eq!(assets.iter().filter(|p| p.file_name().unwrap() == "fail.png").count(), 1);
    }

    #[test]
    fn test_can_sort_dates() {
        let input = vec![
            create_page_with_date("2018-01-01"),
            create_page_with_date("2017-01-01"),
            create_page_with_date("2019-01-01"),
        ];
        let (pages, _) = sort_pages(input, SortBy::Date);
        // Should be sorted by date
        assert_eq!(pages[0].clone().meta.date.unwrap(), "2019-01-01");
        assert_eq!(pages[1].clone().meta.date.unwrap(), "2018-01-01");
        assert_eq!(pages[2].clone().meta.date.unwrap(), "2017-01-01");
    }

    #[test]
    fn test_can_sort_order() {
        let input = vec![
            create_page_with_order(2),
            create_page_with_order(3),
            create_page_with_order(1),
        ];
        let (pages, _) = sort_pages(input, SortBy::Order);
        // Should be sorted by date
        assert_eq!(pages[0].clone().meta.order.unwrap(), 3);
        assert_eq!(pages[1].clone().meta.order.unwrap(), 2);
        assert_eq!(pages[2].clone().meta.order.unwrap(), 1);
    }

    #[test]
    fn test_can_sort_none() {
        let input = vec![
            create_page_with_order(2),
            create_page_with_order(3),
            create_page_with_order(1),
        ];
        let (pages, _) = sort_pages(input, SortBy::None);
        // Should be sorted by date
        assert_eq!(pages[0].clone().meta.order.unwrap(), 2);
        assert_eq!(pages[1].clone().meta.order.unwrap(), 3);
        assert_eq!(pages[2].clone().meta.order.unwrap(), 1);
    }

    #[test]
    fn test_ignore_page_with_missing_field() {
        let input = vec![
            create_page_with_order(2),
            create_page_with_order(3),
            create_page_with_date("2019-01-01"),
        ];
        let (pages, unsorted) = sort_pages(input, SortBy::Order);
        assert_eq!(pages.len(), 2);
        assert_eq!(unsorted.len(), 1);
    }

    #[test]
    fn test_populate_previous_and_next_pages() {
        let input = vec![
            create_page_with_order(3),
            create_page_with_order(2),
            create_page_with_order(1),
        ];
        let pages = populate_previous_and_next_pages(input.as_slice());

        assert!(pages[0].clone().next.is_none());
        assert!(pages[0].clone().previous.is_some());
        assert_eq!(pages[0].clone().previous.unwrap().meta.order.unwrap(), 2);

        assert!(pages[1].clone().next.is_some());
        assert!(pages[1].clone().previous.is_some());
        assert_eq!(pages[1].clone().next.unwrap().meta.order.unwrap(), 3);
        assert_eq!(pages[1].clone().previous.unwrap().meta.order.unwrap(), 1);

        assert!(pages[2].clone().next.is_some());
        assert!(pages[2].clone().previous.is_none());
        assert_eq!(pages[2].clone().next.unwrap().meta.order.unwrap(), 2);
    }
}
