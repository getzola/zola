/// A page, can be a blog post or a basic page
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::{read_dir};
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;


use tera::{Tera, Context};
use serde::ser::{SerializeStruct, self};
use slug::slugify;

use errors::{Result, ResultExt};
use config::Config;
use front_matter::{FrontMatter, split_content};
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
    /// The front matter meta-data
    pub meta: FrontMatter,

    /// The slug of that page.
    /// First tries to find the slug in the meta and defaults to filename otherwise
    pub slug: String,
    /// The URL path of the page
    pub path: String,
    /// The full URL for that page
    pub permalink: String,
    /// The summary for the article, defaults to empty string
    /// When <!-- more --> is found in the text, will take the content up to that part
    /// as summary
    pub summary: String,

    /// The previous page, by date globally
    pub previous: Option<Box<Page>>,
    /// The previous page, by date only for the section the page is in
    pub previous_in_section: Option<Box<Page>>,
    /// The next page, by date
    pub next: Option<Box<Page>>,
    /// The next page, by date only for the section the page is in
    pub next_in_section: Option<Box<Page>>,
}


impl Page {
    pub fn new(meta: FrontMatter) -> Page {
        Page {
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
            summary: "".to_string(),
            meta: meta,
            previous: None,
            previous_in_section: None,
            next: None,
            next_in_section: None,
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
        let (meta, content) = split_content(file_path, content)?;
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
        if let Some(ref u) = page.meta.url {
            page.path = u.trim().to_string();
        } else {
            if !page.components.is_empty() {
                // If we have a folder with an asset, don't consider it as a component
                if page.file_name == "index" {
                    page.components.pop();
                    // also set parent_path to grandparent instead
                    page.parent_path = page.parent_path.parent().unwrap().to_path_buf();
                }

                // Don't add a trailing slash to sections
                page.path = format!("{}/{}", page.components.join("/"), page.slug);
            } else {
                page.path = page.slug.clone();
            }
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
            bail!("Page `{}` has assets but is not named index.md", path.display());
        }

        Ok(page)

    }

    /// We need access to all pages url to render links relative to content
    /// so that can't happen at the same time as parsing
    pub fn render_markdown(&mut self, permalinks: &HashMap<String, String>, tera: &Tera, config: &Config) -> Result<()> {
        self.content = markdown_to_html(&self.raw_content, permalinks, tera, config)?;

        if self.raw_content.contains("<!-- more -->") {
            self.summary = {
                let summary = self.raw_content.splitn(2, "<!-- more -->").collect::<Vec<&str>>()[0];
                markdown_to_html(summary, permalinks, tera, config)?
            }
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

impl ser::Serialize for Page {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error> where S: ser::Serializer {
        let mut state = serializer.serialize_struct("page", 13)?;
        state.serialize_field("content", &self.content)?;
        state.serialize_field("title", &self.meta.title)?;
        state.serialize_field("description", &self.meta.description)?;
        state.serialize_field("date", &self.meta.date)?;
        state.serialize_field("slug", &self.slug)?;
        state.serialize_field("path", &format!("/{}", self.path))?;
        state.serialize_field("permalink", &self.permalink)?;
        state.serialize_field("tags", &self.meta.tags)?;
        state.serialize_field("draft", &self.meta.draft)?;
        state.serialize_field("category", &self.meta.category)?;
        state.serialize_field("extra", &self.meta.extra)?;
        let (word_count, reading_time) = self.get_reading_analytics();
        state.serialize_field("word_count", &word_count)?;
        state.serialize_field("reading_time", &reading_time)?;
        state.end()
    }
}

impl PartialOrd for Page {
    fn partial_cmp(&self, other: &Page) -> Option<Ordering> {
        if self.meta.date.is_none() {
            return Some(Ordering::Less);
        }

        if other.meta.date.is_none() {
            return Some(Ordering::Greater);
        }

        let this_date = self.meta.parse_date().unwrap();
        let other_date = other.meta.parse_date().unwrap();

        if this_date > other_date {
            return Some(Ordering::Less);
        }
        if this_date < other_date {
            return Some(Ordering::Greater);
        }

        Some(Ordering::Equal)
    }
}


/// Horribly inefficient way to set previous and next on each pages
/// So many clones
pub fn populate_previous_and_next_pages(input: &[Page], in_section: bool) -> Vec<Page> {
    let pages = input.to_vec();
    let mut res = Vec::new();

    // the input is sorted from most recent to least recent already
    for (i, page) in input.iter().enumerate() {
        let mut new_page = page.clone();

        if new_page.has_date() {
            if i > 0 {
                let next = &pages[i - 1];
                if next.has_date() {
                    if in_section {
                        new_page.next_in_section = Some(Box::new(next.clone()));
                    } else {
                        new_page.next = Some(Box::new(next.clone()));
                    }
                }
            }

            if i < input.len() - 1 {
                let previous = &pages[i + 1];
                if previous.has_date() {
                    if in_section {
                        new_page.previous_in_section = Some(Box::new(previous.clone()));
                    } else {
                        new_page.previous = Some(Box::new(previous.clone()));
                    }
                }
            }
        }
        res.push(new_page);
    }

    res
}

#[cfg(test)]
mod tests {
    use tempdir::TempDir;

    use std::fs::File;

    use super::{find_related_assets};

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
}
