/// A page, can be a blog post or a basic page
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tera::{Tera, Context as TeraContext, Value, Map};
use slug::slugify;
use slotmap::{Key};

use errors::{Result, ResultExt};
use config::Config;
use utils::fs::{read_file, find_related_assets};
use utils::site::get_reading_analytics;
use utils::templates::render_template;
use front_matter::{PageFrontMatter, InsertAnchor, split_page_content};
use rendering::{RenderContext, Header, render_content};
use library::Library;

use content::file_info::FileInfo;

/// What we are sending to the templates when rendering them
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SerializingPage<'a> {
    content: &'a str,
    permalink: &'a str,
    slug: &'a str,
    parent_section: Option<String>,
    title: &'a Option<String>,
    description: &'a Option<String>,
    date: &'a Option<String>,
    year: Option<i32>,
    month: Option<u32>,
    day: Option<u32>,
    taxonomies: &'a HashMap<String, Vec<String>>,
    extra: &'a Map<String, Value>,
    path: &'a str,
    components: &'a [String],
    summary: &'a Option<String>,
    word_count: Option<usize>,
    reading_time: Option<usize>,
    toc: &'a [Header],
    assets: Vec<String>,
    draft: bool,
    lighter: Option<Box<SerializingPage<'a>>>,
    heavier: Option<Box<SerializingPage<'a>>>,
    earlier: Option<Box<SerializingPage<'a>>>,
    later: Option<Box<SerializingPage<'a>>>,
}

impl<'a> SerializingPage<'a> {
    /// Grabs all the data from a page, including sibling pages
    pub fn from_page(page: &'a Page, library: &'a Library) -> Self {
        let mut year = None;
        let mut month = None;
        let mut day = None;
        if let Some(d) = page.meta.datetime_tuple {
            year = Some(d.0);
            month = Some(d.1);
            day = Some(d.2);
        }
        let pages = library.pages();
        let lighter = page.lighter.map(|k| Box::new(SerializingPage::from_page_basic(pages.get(k).unwrap())));
        let heavier = page.heavier.map(|k| Box::new(SerializingPage::from_page_basic(pages.get(k).unwrap())));
        let earlier = page.earlier.map(|k| Box::new(SerializingPage::from_page_basic(pages.get(k).unwrap())));
        let later = page.later.map(|k| Box::new(SerializingPage::from_page_basic(pages.get(k).unwrap())));
        let parent_section = page.parent_section.map(|k| library.get_section_by_key(k).file.relative.clone());

        SerializingPage {
            parent_section,
            content: &page.content,
            permalink: &page.permalink,
            slug: &page.slug,
            title: &page.meta.title,
            description: &page.meta.description,
            extra: &page.meta.extra,
            date: &page.meta.date,
            year,
            month,
            day,
            taxonomies: &page.meta.taxonomies,
            path: &page.path,
            components: &page.components,
            summary: &page.summary,
            word_count: page.word_count,
            reading_time: page.reading_time,
            toc: &page.toc,
            assets: page.serialize_assets(),
            draft: page.is_draft(),
            lighter,
            heavier,
            earlier,
            later,
        }
    }

    /// Same as from_page but does not fill sibling pages
    pub fn from_page_basic(page: &'a Page) -> Self {
        let mut year = None;
        let mut month = None;
        let mut day = None;
        if let Some(d) = page.meta.datetime_tuple {
            year = Some(d.0);
            month = Some(d.1);
            day = Some(d.2);
        }

        SerializingPage {
            parent_section: None,
            content: &page.content,
            permalink: &page.permalink,
            slug: &page.slug,
            title: &page.meta.title,
            description: &page.meta.description,
            extra: &page.meta.extra,
            date: &page.meta.date,
            year,
            month,
            day,
            taxonomies: &page.meta.taxonomies,
            path: &page.path,
            components: &page.components,
            summary: &page.summary,
            word_count: page.word_count,
            reading_time: page.reading_time,
            toc: &page.toc,
            assets: page.serialize_assets(),
            draft: page.is_draft(),
            lighter: None,
            heavier: None,
            earlier: None,
            later: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Page {
    /// All info about the actual file
    pub file: FileInfo,
    /// The front matter meta-data
    pub meta: PageFrontMatter,
    /// The parent section if there is one
    pub parent_section: Option<Key>,
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
    /// The components of the path of the page
    pub components: Vec<String>,
    /// The full URL for that page
    pub permalink: String,
    /// The summary for the article, defaults to None
    /// When <!-- more --> is found in the text, will take the content up to that part
    /// as summary
    pub summary: Option<String>,
    /// The earlier page, for pages sorted by date
    pub earlier: Option<Key>,
    /// The later page, for pages sorted by date
    pub later: Option<Key>,
    /// The lighter page, for pages sorted by weight
    pub lighter: Option<Key>,
    /// The heavier page, for pages sorted by weight
    pub heavier: Option<Key>,
    /// Toc made from the headers of the markdown file
    pub toc: Vec<Header>,
    /// How many words in the raw content
    pub word_count: Option<usize>,
    /// How long would it take to read the raw content.
    /// See `get_reading_analytics` on how it is calculated
    pub reading_time: Option<usize>,
}


impl Page {
    pub fn new<P: AsRef<Path>>(file_path: P, meta: PageFrontMatter) -> Page {
        let file_path = file_path.as_ref();

        Page {
            file: FileInfo::new_page(file_path),
            meta,
            parent_section: None,
            raw_content: "".to_string(),
            assets: vec![],
            content: "".to_string(),
            slug: "".to_string(),
            path: "".to_string(),
            components: vec![],
            permalink: "".to_string(),
            summary: None,
            earlier: None,
            later: None,
            lighter: None,
            heavier: None,
            toc: vec![],
            word_count: None,
            reading_time: None,
        }
    }

    pub fn is_draft(&self) -> bool {
        self.meta.draft
    }

    /// Parse a page given the content of the .md file
    /// Files without front matter or with invalid front matter are considered
    /// erroneous
    pub fn parse(file_path: &Path, content: &str, config: &Config) -> Result<Page> {
        let (meta, content) = split_page_content(file_path, content)?;
        let mut page = Page::new(file_path, meta);
        page.raw_content = content;
        let (word_count, reading_time) = get_reading_analytics(&page.raw_content);
        page.word_count = Some(word_count);
        page.reading_time = Some(reading_time);
        page.slug = {
            if let Some(ref slug) = page.meta.slug {
                slug.trim().to_string()
            } else if page.file.name == "index" {
                if let Some(parent) = page.file.path.parent() {
                    slugify(parent.file_name().unwrap().to_str().unwrap())
                } else {
                    slugify(page.file.name.clone())
                }
            } else {
                slugify(page.file.name.clone())
            }
        };

        if let Some(ref p) = page.meta.path {
            page.path = p.trim().trim_left_matches('/').to_string();
        } else {
            page.path = if page.file.components.is_empty() {
                page.slug.clone()
            } else {
                format!("{}/{}", page.file.components.join("/"), page.slug)
            };
        }
        if !page.path.ends_with('/') {
            page.path = format!("{}/", page.path);
        }

        page.components = page.path.split('/')
            .map(|p| p.to_string())
            .filter(|p| !p.is_empty())
            .collect::<Vec<_>>();
        page.permalink = config.make_permalink(&page.path);

        Ok(page)
    }

    /// Read and parse a .md file into a Page struct
    pub fn from_file<P: AsRef<Path>>(path: P, config: &Config) -> Result<Page> {
        let path = path.as_ref();
        let content = read_file(path)?;
        let mut page = Page::parse(path, &content, config)?;

        if page.file.name == "index" {
            let parent_dir = path.parent().unwrap();
            let assets = find_related_assets(parent_dir);

            if let Some(ref globset) = config.ignored_content_globset {
                // `find_related_assets` only scans the immediate directory (it is not recursive) so our
                // filtering only needs to work against the file_name component, not the full suffix. If
                // `find_related_assets` was changed to also return files in subdirectories, we could
                // use `PathBuf.strip_prefix` to remove the parent directory and then glob-filter
                // against the remaining path. Note that the current behaviour effectively means that
                // the `ignored_content` setting in the config file is limited to single-file glob
                // patterns (no "**" patterns).
                page.assets = assets.into_iter()
                    .filter(|path|
                        match path.file_name() {
                            None => true,
                            Some(file) => !globset.is_match(file)
                        }
                    ).collect();
            } else {
                page.assets = assets;
            }
        } else {
            page.assets = vec![];
        }

        Ok(page)
    }

    /// We need access to all pages url to render links relative to content
    /// so that can't happen at the same time as parsing
    pub fn render_markdown(
        &mut self,
        permalinks: &HashMap<String, String>,
        tera: &Tera,
        config: &Config,
        anchor_insert: InsertAnchor,
    ) -> Result<()> {
        let mut context = RenderContext::new(
            tera,
            config,
            &self.permalink,
            permalinks,
            anchor_insert,
        );

        context.tera_context.insert("page", &SerializingPage::from_page_basic(self));

        let res = render_content(&self.raw_content, &context)
            .chain_err(|| format!("Failed to render content of {}", self.file.path.display()))?;

        self.summary = res.summary_len.map(|l| res.body[0..l].to_owned());
        self.content = res.body;
        self.toc = res.toc;

        Ok(())
    }

    /// Renders the page using the default layout, unless specified in front-matter
    pub fn render_html(&self, tera: &Tera, config: &Config, library: &Library) -> Result<String> {
        let tpl_name = match self.meta.template {
            Some(ref l) => l.to_string(),
            None => "page.html".to_string()
        };

        let mut context = TeraContext::new();
        context.insert("config", config);
        context.insert("current_url", &self.permalink);
        context.insert("current_path", &self.path);
        context.insert("page", &self.to_serialized(library));

        render_template(&tpl_name, tera, &context, &config.theme)
            .chain_err(|| format!("Failed to render page '{}'", self.file.path.display()))
    }

    /// Creates a vectors of asset URLs.
    fn serialize_assets(&self) -> Vec<String> {
        self.assets.iter()
            .filter_map(|asset| asset.file_name())
            .filter_map(|filename| filename.to_str())
            .map(|filename| self.path.clone() + filename)
            .collect()
    }

    pub fn to_serialized<'a>(&'a self, library: &'a Library) -> SerializingPage<'a> {
        SerializingPage::from_page(self, library)
    }

    pub fn to_serialized_basic(&self) -> SerializingPage {
        SerializingPage::from_page_basic(self)
    }
}

impl Default for Page {
    fn default() -> Page {
        Page {
            file: FileInfo::default(),
            meta: PageFrontMatter::default(),
            parent_section: None,
            raw_content: "".to_string(),
            assets: vec![],
            content: "".to_string(),
            slug: "".to_string(),
            path: "".to_string(),
            components: vec![],
            permalink: "".to_string(),
            summary: None,
            earlier: None,
            later: None,
            lighter: None,
            heavier: None,
            toc: vec![],
            word_count: None,
            reading_time: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::io::Write;
    use std::fs::{File, create_dir};
    use std::path::Path;

    use tera::Tera;
    use tempfile::tempdir;
    use globset::{Glob, GlobSetBuilder};

    use config::Config;
    use super::Page;
    use front_matter::InsertAnchor;


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
        page.render_markdown(
            &HashMap::default(),
            &Tera::default(),
            &Config::default(),
            InsertAnchor::None,
        ).unwrap();

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
        let page = res.unwrap();
        assert_eq!(page.path, "posts/intro/hello-world/");
        assert_eq!(page.components, vec!["posts", "intro", "hello-world"]);
        assert_eq!(page.permalink, "http://hello.com/posts/intro/hello-world/");
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
        let page = res.unwrap();
        assert_eq!(page.path, "hello-world/");
        assert_eq!(page.components, vec!["hello-world"]);
        assert_eq!(page.permalink, config.make_permalink("hello-world"));
    }

    #[test]
    fn can_make_url_from_path() {
        let content = r#"
    +++
    path = "hello-world"
    +++
    Hello world"#;
        let config = Config::default();
        let res = Page::parse(Path::new("content/posts/intro/start.md"), content, &config);
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.path, "hello-world/");
        assert_eq!(page.components, vec!["hello-world"]);
        assert_eq!(page.permalink, config.make_permalink("hello-world"));
    }

    #[test]
    fn can_make_url_from_path_starting_slash() {
        let content = r#"
    +++
    path = "/hello-world"
    +++
    Hello world"#;
        let config = Config::default();
        let res = Page::parse(Path::new("content/posts/intro/start.md"), content, &config);
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.path, "hello-world/");
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
        let page = res.unwrap();
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
        page.render_markdown(
            &HashMap::default(),
            &Tera::default(),
            &config,
            InsertAnchor::None,
        ).unwrap();
        assert_eq!(page.summary, Some("<p>Hello world</p>\n".to_string()));
    }

    #[test]
    fn page_with_assets_gets_right_info() {
        let tmp_dir = tempdir().expect("create temp dir");
        let path = tmp_dir.path();
        create_dir(&path.join("content")).expect("create content temp dir");
        create_dir(&path.join("content").join("posts")).expect("create posts temp dir");
        let nested_path = path.join("content").join("posts").join("with-assets");
        create_dir(&nested_path).expect("create nested temp dir");
        let mut f = File::create(nested_path.join("index.md")).unwrap();
        f.write_all(b"+++\n+++\n").unwrap();
        File::create(nested_path.join("example.js")).unwrap();
        File::create(nested_path.join("graph.jpg")).unwrap();
        File::create(nested_path.join("fail.png")).unwrap();

        let res = Page::from_file(
            nested_path.join("index.md").as_path(),
            &Config::default(),
        );
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.file.parent, path.join("content").join("posts"));
        assert_eq!(page.slug, "with-assets");
        assert_eq!(page.assets.len(), 3);
        assert_eq!(page.permalink, "http://a-website.com/posts/with-assets/");
    }

    #[test]
    fn page_with_assets_and_slug_overrides_path() {
        let tmp_dir = tempdir().expect("create temp dir");
        let path = tmp_dir.path();
        create_dir(&path.join("content")).expect("create content temp dir");
        create_dir(&path.join("content").join("posts")).expect("create posts temp dir");
        let nested_path = path.join("content").join("posts").join("with-assets");
        create_dir(&nested_path).expect("create nested temp dir");
        let mut f = File::create(nested_path.join("index.md")).unwrap();
        f.write_all(b"+++\nslug=\"hey\"\n+++\n").unwrap();
        File::create(nested_path.join("example.js")).unwrap();
        File::create(nested_path.join("graph.jpg")).unwrap();
        File::create(nested_path.join("fail.png")).unwrap();

        let res = Page::from_file(
            nested_path.join("index.md").as_path(),
            &Config::default(),
        );
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.file.parent, path.join("content").join("posts"));
        assert_eq!(page.slug, "hey");
        assert_eq!(page.assets.len(), 3);
        assert_eq!(page.permalink, "http://a-website.com/posts/hey/");
    }

    #[test]
    fn page_with_ignored_assets_filters_out_correct_files() {
        let tmp_dir = tempdir().expect("create temp dir");
        let path = tmp_dir.path();
        create_dir(&path.join("content")).expect("create content temp dir");
        create_dir(&path.join("content").join("posts")).expect("create posts temp dir");
        let nested_path = path.join("content").join("posts").join("with-assets");
        create_dir(&nested_path).expect("create nested temp dir");
        let mut f = File::create(nested_path.join("index.md")).unwrap();
        f.write_all(b"+++\nslug=\"hey\"\n+++\n").unwrap();
        File::create(nested_path.join("example.js")).unwrap();
        File::create(nested_path.join("graph.jpg")).unwrap();
        File::create(nested_path.join("fail.png")).unwrap();

        let mut gsb = GlobSetBuilder::new();
        gsb.add(Glob::new("*.{js,png}").unwrap());
        let mut config = Config::default();
        config.ignored_content_globset = Some(gsb.build().unwrap());

        let res = Page::from_file(
            nested_path.join("index.md").as_path(),
            &config,
        );

        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.assets.len(), 1);
        assert_eq!(page.assets[0].file_name().unwrap().to_str(), Some("graph.jpg"));
    }
}
