/// A page, can be a blog post or a basic page
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;


use tera::{Tera, Context};
use serde::ser::{SerializeStruct, self};
use slug::slugify;

use errors::{Result, ResultExt};
use config::Config;
use front_matter::{PageFrontMatter, split_page_content};
use markdown::markdown_to_html;
use utils::{read_file};
use content::utils::{find_related_assets, find_content_components, get_reading_analytics};



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
        let (word_count, reading_time) = get_reading_analytics(&self.raw_content);
        state.serialize_field("word_count", &word_count)?;
        state.serialize_field("reading_time", &reading_time)?;
        state.serialize_field("previous", &self.previous)?;
        state.serialize_field("next", &self.next)?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs::{File, create_dir};
    use std::path::Path;

    use tera::Tera;
    use tempdir::TempDir;

    use config::Config;
    use super::Page;


    #[test]
    fn test_can_parse_a_valid_page() {
        let content = r#"
+++
title = "Hello"
description = "hey there"
slug = "hello-world"
+++
Hello world"#;
        let res = Page::parse(Path::new("post.md"), content, &Config::default());
        assert!(res.is_ok());
        let mut page = res.unwrap();
        page.render_markdown(&HashMap::default(), &Tera::default(), &Config::default()).unwrap();

        assert_eq!(page.meta.title.unwrap(), "Hello".to_string());
        assert_eq!(page.meta.slug.unwrap(), "hello-world".to_string());
        assert_eq!(page.raw_content, "Hello world".to_string());
        assert_eq!(page.content, "<p>Hello world</p>\n".to_string());
    }

    #[test]
    fn test_can_make_url_from_sections_and_slug() {
        let content = r#"
    +++
    slug = "hello-world"
    +++
    Hello world"#;
        let mut conf = Config::default();
        conf.base_url = "http://hello.com/".to_string();
        let res = Page::parse(Path::new("content/posts/intro/start.md"), content, &conf);
        assert!(res.is_ok());
        let mut page = res.unwrap();
        page.render_markdown(&HashMap::default(), &Tera::default(), &Config::default()).unwrap();
        assert_eq!(page.path, "posts/intro/hello-world");
        assert_eq!(page.permalink, "http://hello.com/posts/intro/hello-world");
    }

    #[test]
    fn can_make_url_from_slug_only() {
        let content = r#"
    +++
    slug = "hello-world"
    +++
    Hello world"#;
        let config = Config::default();
        let res = Page::parse(Path::new("start.md"), content, &config);
        assert!(res.is_ok());
        let mut page = res.unwrap();
        page.render_markdown(&HashMap::default(), &Tera::default(), &config).unwrap();
        assert_eq!(page.path, "hello-world");
        assert_eq!(page.permalink, config.make_permalink("hello-world"));
    }

    #[test]
    fn errors_on_invalid_front_matter_format() {
        // missing starting +++
        let content = r#"
    title = "Hello"
    description = "hey there"
    slug = "hello-world"
    +++
    Hello world"#;
        let res = Page::parse(Path::new("start.md"), content, &Config::default());
        assert!(res.is_err());
    }

    #[test]
    fn can_make_slug_from_non_slug_filename() {
        let config = Config::default();
        let res = Page::parse(Path::new(" file with space.md"), "+++\n+++", &config);
        assert!(res.is_ok());
        let mut page = res.unwrap();
        page.render_markdown(&HashMap::default(), &Tera::default(), &config).unwrap();
        assert_eq!(page.slug, "file-with-space");
        assert_eq!(page.permalink, config.make_permalink(&page.slug));
    }

    #[test]
    fn can_specify_summary() {
        let config = Config::default();
        let content = r#"
+++
+++
Hello world
<!-- more -->"#.to_string();
        let res = Page::parse(Path::new("hello.md"), &content, &config);
        assert!(res.is_ok());
        let mut page = res.unwrap();
        page.render_markdown(&HashMap::default(), &Tera::default(), &config).unwrap();
        assert_eq!(page.summary, Some("<p>Hello world</p>\n".to_string()));
    }

    #[test]
    fn page_with_assets_gets_right_parent_path() {
        let tmp_dir = TempDir::new("example").expect("create temp dir");
        let path = tmp_dir.path();
        create_dir(&path.join("content")).expect("create content temp dir");
        create_dir(&path.join("content").join("posts")).expect("create posts temp dir");
        let nested_path = path.join("content").join("posts").join("assets");
        create_dir(&nested_path).expect("create nested temp dir");
        File::create(nested_path.join("index.md")).unwrap();
        File::create(nested_path.join("example.js")).unwrap();
        File::create(nested_path.join("graph.jpg")).unwrap();
        File::create(nested_path.join("fail.png")).unwrap();

        let res = Page::parse(
            nested_path.join("index.md").as_path(),
            "+++\nurl=\"hey\"+++\n",
            &Config::default()
        );
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.parent_path, path.join("content").join("posts"));
    }

    #[test]
    fn errors_file_not_named_index_with_assets() {
        let tmp_dir = TempDir::new("example").expect("create temp dir");
        File::create(tmp_dir.path().join("something.md")).unwrap();
        File::create(tmp_dir.path().join("example.js")).unwrap();
        File::create(tmp_dir.path().join("graph.jpg")).unwrap();
        File::create(tmp_dir.path().join("fail.png")).unwrap();

        let page = Page::from_file(tmp_dir.path().join("something.md"), &Config::default());
        assert!(page.is_err());
    }
}
