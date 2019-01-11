/// A page, can be a blog post or a basic page
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use regex::Regex;
use slotmap::Key;
use slug::slugify;
use tera::{Context as TeraContext, Tera};

use config::Config;
use errors::{Result, Error};
use front_matter::{split_page_content, InsertAnchor, PageFrontMatter};
use library::Library;
use rendering::{render_content, Header, RenderContext};
use utils::fs::{find_related_assets, read_file};
use utils::site::get_reading_analytics;
use utils::templates::render_template;

use content::file_info::FileInfo;
use content::ser::SerializingPage;

lazy_static! {
    // Based on https://regex101.com/r/H2n38Z/1/tests
    // A regex parsing RFC3339 date followed by {_,-}, some characters and ended by .md
    static ref RFC3339_DATE: Regex = Regex::new(
        r"^(?P<datetime>(\d{4})-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])(T([01][0-9]|2[0-3]):([0-5][0-9]):([0-5][0-9]|60)(\.[0-9]+)?(Z|(\+|-)([01][0-9]|2[0-3]):([0-5][0-9])))?)(_|-)(?P<slug>.+$)"
    ).unwrap();
}

#[derive(Clone, Debug, PartialEq)]
pub struct Page {
    /// All info about the actual file
    pub file: FileInfo,
    /// The front matter meta-data
    pub meta: PageFrontMatter,
    /// The list of parent sections
    pub ancestors: Vec<Key>,
    /// The actual content of the page, in markdown
    pub raw_content: String,
    /// All the non-md files we found next to the .md file
    pub assets: Vec<PathBuf>,
    /// All the non-md files we found next to the .md file as string for use in templates
    pub serialized_assets: Vec<String>,
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
    /// The language of that page. `None` if the user doesn't setup `languages` in config.
    /// Corresponds to the lang in the {slug}.{lang}.md file scheme
    pub lang: Option<String>,
    /// Contains all the translated version of that page
    pub translations: Vec<Key>,
}

impl Page {
    pub fn new<P: AsRef<Path>>(file_path: P, meta: PageFrontMatter) -> Page {
        let file_path = file_path.as_ref();

        Page {
            file: FileInfo::new_page(file_path),
            meta,
            ancestors: vec![],
            raw_content: "".to_string(),
            assets: vec![],
            serialized_assets: vec![],
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
            lang: None,
            translations: Vec::new(),
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

        page.lang = page.file.find_language(config)?;

        page.raw_content = content;
        let (word_count, reading_time) = get_reading_analytics(&page.raw_content);
        page.word_count = Some(word_count);
        page.reading_time = Some(reading_time);

        let mut slug_from_dated_filename = None;
        if let Some(ref caps) = RFC3339_DATE.captures(&page.file.name.replace(".md", "")) {
            slug_from_dated_filename = Some(caps.name("slug").unwrap().as_str().to_string());
            if page.meta.date.is_none() {
                page.meta.date = Some(caps.name("datetime").unwrap().as_str().to_string());
                page.meta.date_to_datetime();
            }
        }

        page.slug = {
            if let Some(ref slug) = page.meta.slug {
                slug.trim().to_string()
            } else if page.file.name == "index" {
                if let Some(parent) = page.file.path.parent() {
                    slugify(parent.file_name().unwrap().to_str().unwrap())
                } else {
                    slugify(&page.file.name)
                }
            } else {
                if let Some(slug) = slug_from_dated_filename {
                    slugify(&slug)
                } else {
                    slugify(&page.file.name)
                }
            }
        };

        if let Some(ref p) = page.meta.path {
            page.path = p.trim().trim_left_matches('/').to_string();
        } else {
            let mut path = if page.file.components.is_empty() {
                page.slug.clone()
            } else {
                format!("{}/{}", page.file.components.join("/"), page.slug)
            };

            if let Some(ref lang) = page.lang {
                path = format!("{}/{}", lang, path);
            }

            page.path = path;
        }
        if !page.path.ends_with('/') {
            page.path = format!("{}/", page.path);
        }

        page.components = page
            .path
            .split('/')
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
                page.assets = assets
                    .into_iter()
                    .filter(|path| match path.file_name() {
                        None => true,
                        Some(file) => !globset.is_match(file),
                    })
                    .collect();
            } else {
                page.assets = assets;
            }

            page.serialized_assets = page.serialize_assets();
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
        let mut context =
            RenderContext::new(tera, config, &self.permalink, permalinks, anchor_insert);

        context.tera_context.insert("page", &SerializingPage::from_page_basic(self, None));

        let res = render_content(&self.raw_content, &context)
            .map_err(|e| Error::chain(format!("Failed to render content of {}", self.file.path.display()), e))?;

        self.summary = res.summary_len.map(|l| res.body[0..l].to_owned());
        self.content = res.body;
        self.toc = res.toc;

        Ok(())
    }

    /// Renders the page using the default layout, unless specified in front-matter
    pub fn render_html(&self, tera: &Tera, config: &Config, library: &Library) -> Result<String> {
        let tpl_name = match self.meta.template {
            Some(ref l) => l,
            None => "page.html",
        };

        let mut context = TeraContext::new();
        context.insert("config", config);
        context.insert("current_url", &self.permalink);
        context.insert("current_path", &self.path);
        context.insert("page", &self.to_serialized(library));
        context.insert("lang", &self.lang);

        render_template(&tpl_name, tera, &context, &config.theme)
            .map_err(|e| Error::chain(format!("Failed to render page '{}'", self.file.path.display()), e))
    }

    /// Creates a vectors of asset URLs.
    fn serialize_assets(&self) -> Vec<String> {
        self.assets
            .iter()
            .filter_map(|asset| asset.file_name())
            .filter_map(|filename| filename.to_str())
            .map(|filename| self.path.clone() + filename)
            .collect()
    }

    pub fn to_serialized<'a>(&'a self, library: &'a Library) -> SerializingPage<'a> {
        SerializingPage::from_page(self, library)
    }

    pub fn to_serialized_basic<'a>(&'a self, library: &'a Library) -> SerializingPage<'a> {
        SerializingPage::from_page_basic(self, Some(library))
    }
}

impl Default for Page {
    fn default() -> Page {
        Page {
            file: FileInfo::default(),
            meta: PageFrontMatter::default(),
            ancestors: vec![],
            raw_content: "".to_string(),
            assets: vec![],
            serialized_assets: vec![],
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
            lang: None,
            translations: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs::{create_dir, File};
    use std::io::Write;
    use std::path::Path;

    use globset::{Glob, GlobSetBuilder};
    use tempfile::tempdir;
    use tera::Tera;

    use super::Page;
    use config::{Config, Language};
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
        )
        .unwrap();

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
<!-- more -->"#
            .to_string();
        let res = Page::parse(Path::new("hello.md"), &content, &config);
        assert!(res.is_ok());
        let mut page = res.unwrap();
        page.render_markdown(&HashMap::default(), &Tera::default(), &config, InsertAnchor::None)
            .unwrap();
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

        let res = Page::from_file(nested_path.join("index.md").as_path(), &Config::default());
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

        let res = Page::from_file(nested_path.join("index.md").as_path(), &Config::default());
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

        let res = Page::from_file(nested_path.join("index.md").as_path(), &config);

        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.assets.len(), 1);
        assert_eq!(page.assets[0].file_name().unwrap().to_str(), Some("graph.jpg"));
    }

    #[test]
    fn can_get_date_from_short_date_in_filename() {
        let config = Config::default();
        let content = r#"
+++
+++
Hello world
<!-- more -->"#
            .to_string();
        let res = Page::parse(Path::new("2018-10-08_hello.md"), &content, &config);
        assert!(res.is_ok());
        let page = res.unwrap();

        assert_eq!(page.meta.date, Some("2018-10-08".to_string()));
        assert_eq!(page.slug, "hello");
    }

    #[test]
    fn can_get_date_from_full_rfc3339_date_in_filename() {
        let config = Config::default();
        let content = r#"
+++
+++
Hello world
<!-- more -->"#
            .to_string();
        let res = Page::parse(Path::new("2018-10-02T15:00:00Z-hello.md"), &content, &config);
        assert!(res.is_ok());
        let page = res.unwrap();

        assert_eq!(page.meta.date, Some("2018-10-02T15:00:00Z".to_string()));
        assert_eq!(page.slug, "hello");
    }

    #[test]
    fn frontmatter_date_override_filename_date() {
        let config = Config::default();
        let content = r#"
+++
date = 2018-09-09
+++
Hello world
<!-- more -->"#
            .to_string();
        let res = Page::parse(Path::new("2018-10-08_hello.md"), &content, &config);
        assert!(res.is_ok());
        let page = res.unwrap();

        assert_eq!(page.meta.date, Some("2018-09-09".to_string()));
        assert_eq!(page.slug, "hello");
    }

    #[test]
    fn can_specify_language_in_filename() {
        let mut config = Config::default();
        config.languages.push(Language { code: String::from("fr"), rss: false });
        let content = r#"
+++
+++
Bonjour le monde"#
            .to_string();
        let res = Page::parse(Path::new("hello.fr.md"), &content, &config);
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.lang, Some("fr".to_string()));
        assert_eq!(page.slug, "hello");
        assert_eq!(page.permalink, "http://a-website.com/fr/hello/");
    }

    #[test]
    fn can_specify_language_in_filename_with_date() {
        let mut config = Config::default();
        config.languages.push(Language { code: String::from("fr"), rss: false });
        let content = r#"
+++
+++
Bonjour le monde"#
            .to_string();
        let res = Page::parse(Path::new("2018-10-08_hello.fr.md"), &content, &config);
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.meta.date, Some("2018-10-08".to_string()));
        assert_eq!(page.lang, Some("fr".to_string()));
        assert_eq!(page.slug, "hello");
        assert_eq!(page.permalink, "http://a-website.com/fr/hello/");
    }

    #[test]
    fn i18n_frontmatter_path_overrides_default_permalink() {
        let mut config = Config::default();
        config.languages.push(Language { code: String::from("fr"), rss: false });
        let content = r#"
+++
path = "bonjour"
+++
Bonjour le monde"#
            .to_string();
        let res = Page::parse(Path::new("hello.fr.md"), &content, &config);
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.lang, Some("fr".to_string()));
        assert_eq!(page.slug, "hello");
        assert_eq!(page.permalink, "http://a-website.com/bonjour/");
    }
}
