/// A page, can be a blog post or a basic page
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use libs::once_cell::sync::Lazy;
use libs::regex::Regex;
use libs::tera::{Context as TeraContext, Tera};

use config::Config;
use errors::{Context, Result};
use markdown::{render_content, RenderContext};
use utils::slugs::slugify_paths;
use utils::table_of_contents::Heading;
use utils::templates::{render_template, ShortcodeDefinition};
use utils::types::InsertAnchor;

use crate::file_info::FileInfo;
use crate::front_matter::{split_page_content, PageFrontMatter};
use crate::library::Library;
use crate::ser::SerializingPage;
use crate::utils::get_reading_analytics;
use crate::utils::{find_related_assets, has_anchor};
use utils::anchors::has_anchor_id;
use utils::fs::read_file;

// Based on https://regex101.com/r/H2n38Z/1/tests
// A regex parsing RFC3339 date followed by {_,-} and some characters
static RFC3339_DATE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^(?P<datetime>(\d{4})-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])(T([01][0-9]|2[0-3]):([0-5][0-9]):([0-5][0-9]|60)(\.[0-9]+)?(Z|(\+|-)([01][0-9]|2[0-3]):([0-5][0-9])))?)\s?(_|-)(?P<slug>.+$)"
    ).unwrap()
});

static FOOTNOTES_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"<sup class="footnote-reference"><a href=\s*.*?>\s*.*?</a></sup>"#).unwrap()
});

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Page {
    /// All info about the actual file
    pub file: FileInfo,
    /// The front matter meta-data
    pub meta: PageFrontMatter,
    /// The list of parent sections relative paths
    pub ancestors: Vec<String>,
    /// The actual content of the page, in markdown
    pub raw_content: String,
    /// All the non-md files we found next to the .md file
    pub assets: Vec<PathBuf>,
    /// All the non-md files we found next to the .md file
    pub serialized_assets: Vec<String>,
    /// The HTML rendered of the page
    pub content: String,
    /// The slug of that page.
    /// First tries to find the slug in the meta and defaults to filename otherwise
    pub slug: String,
    /// The URL path of the page, always starting with a slash
    pub path: String,
    /// The components of the path of the page
    pub components: Vec<String>,
    /// The full URL for that page
    pub permalink: String,
    /// The summary for the article, defaults to None
    /// When <!-- more --> is found in the text, will take the content up to that part
    /// as summary
    pub summary: Option<String>,
    /// The previous page when sorting: earlier/earlier_updated/lighter/prev
    pub lower: Option<PathBuf>,
    /// The next page when sorting: later/later_updated/heavier/next
    pub higher: Option<PathBuf>,
    /// Toc made from the headings of the markdown file
    pub toc: Vec<Heading>,
    /// How many words in the raw content
    pub word_count: Option<usize>,
    /// How long would it take to read the raw content.
    /// See `get_reading_analytics` on how it is calculated
    pub reading_time: Option<usize>,
    /// The language of that page. Equal to the default lang if the user doesn't setup `languages` in config.
    /// Corresponds to the lang in the {slug}.{lang}.md file scheme
    pub lang: String,
    /// Contains all the translated version of that page
    pub translations: Vec<PathBuf>,
    /// The list of all internal links (as path to markdown file), with optional anchor fragments.
    /// We can only check the anchor after all pages have been built and their ToC compiled.
    /// The page itself should exist otherwise it would have errored before getting there.
    pub internal_links: Vec<(String, Option<String>)>,
    /// The list of all links to external webpages. They can be validated by the `link_checker`.
    pub external_links: Vec<String>,
}

impl Page {
    pub fn new<P: AsRef<Path>>(file_path: P, meta: PageFrontMatter, base_path: &Path) -> Page {
        let file_path = file_path.as_ref();

        Page { file: FileInfo::new_page(file_path, base_path), meta, ..Self::default() }
    }

    /// Parse a page given the content of the .md file
    /// Files without front matter or with invalid front matter are considered
    /// erroneous
    pub fn parse(
        file_path: &Path,
        content: &str,
        config: &Config,
        base_path: &Path,
    ) -> Result<Page> {
        let (meta, content) = split_page_content(file_path, content)?;
        let mut page = Page::new(file_path, meta, base_path);

        page.lang =
            page.file.find_language(&config.default_language, &config.other_languages_codes())?;

        page.raw_content = content.to_string();
        let (word_count, reading_time) = get_reading_analytics(&page.raw_content);
        page.word_count = Some(word_count);
        page.reading_time = Some(reading_time);

        let mut slug_from_dated_filename = None;

        let file_path_for_slug = if page.file.name == "index" {
            if let Some(parent) = page.file.path.parent() {
                parent.file_name().unwrap().to_str().unwrap().to_string()
            } else {
                page.file.name.to_string()
            }
        } else {
            page.file.name.to_string()
        };

        if let Some(ref caps) = RFC3339_DATE.captures(&file_path_for_slug) {
            if !config.slugify.paths_keep_dates {
                slug_from_dated_filename = Some(caps.name("slug").unwrap().as_str().to_string());
            }
            if page.meta.date.is_none() {
                page.meta.date = Some(caps.name("datetime").unwrap().as_str().to_string());
                page.meta.date_to_datetime();
            }
        }

        page.slug = {
            if let Some(ref slug) = page.meta.slug {
                slugify_paths(slug, config.slugify.paths)
            } else if let Some(slug) = slug_from_dated_filename {
                slugify_paths(&slug, config.slugify.paths)
            } else {
                slugify_paths(&file_path_for_slug, config.slugify.paths)
            }
        };

        page.path = if let Some(ref p) = page.meta.path {
            let path = p.trim();

            if path.starts_with('/') {
                path.into()
            } else {
                format!("/{}", path)
            }
        } else {
            let mut path = if page.file.components.is_empty() {
                if page.file.name == "index" && page.file.colocated_path.is_none() {
                    String::new()
                } else {
                    page.slug.clone()
                }
            } else {
                format!("{}/{}", page.file.components.join("/"), page.slug)
            };

            if page.lang != config.default_language {
                path = format!("{}/{}", page.lang, path);
            }

            format!("/{}", path)
        };

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

    pub fn find_language(&mut self) {}

    /// Read and parse a .md file into a Page struct
    pub fn from_file<P: AsRef<Path>>(path: P, config: &Config, base_path: &Path) -> Result<Page> {
        let path = path.as_ref();
        let content = read_file(path)?;
        let mut page = Page::parse(path, &content, config, base_path)?;

        if page.file.name == "index" {
            let parent_dir = path.parent().unwrap();
            page.assets = find_related_assets(parent_dir, config, true);
            page.serialized_assets = page.serialize_assets(base_path);
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
        shortcode_definitions: &HashMap<String, ShortcodeDefinition>,
    ) -> Result<()> {
        let mut context = RenderContext::new(
            tera,
            config,
            &self.lang,
            &self.permalink,
            permalinks,
            anchor_insert,
        );
        context.set_shortcode_definitions(shortcode_definitions);
        context.set_current_page_path(&self.file.relative);
        context.tera_context.insert("page", &SerializingPage::new(self, None, false));

        let res = render_content(&self.raw_content, &context)
            .with_context(|| format!("Failed to render content of {}", self.file.path.display()))?;

        self.summary = res
            .summary_len
            .map(|l| &res.body[0..l])
            .map(|s| FOOTNOTES_RE.replace_all(s, "").into_owned());
        self.content = res.body;
        self.toc = res.toc;
        self.external_links = res.external_links;
        self.internal_links = res.internal_links;

        Ok(())
    }

    /// Renders the page using the default layout, unless specified in front-matter
    pub fn render_html(&self, tera: &Tera, config: &Config, library: &Library) -> Result<String> {
        let tpl_name = match self.meta.template {
            Some(ref l) => l,
            None => "page.html",
        };

        let mut context = TeraContext::new();
        context.insert("config", &config.serialize(&self.lang));
        context.insert("current_url", &self.permalink);
        context.insert("current_path", &self.path);
        context.insert("page", &self.serialize(library));
        context.insert("lang", &self.lang);

        render_template(tpl_name, tera, context, &config.theme)
            .with_context(|| format!("Failed to render page '{}'", self.file.path.display()))
    }

    /// Creates a vectors of asset URLs.
    fn serialize_assets(&self, base_path: &Path) -> Vec<String> {
        self.assets
            .iter()
            .filter_map(|asset| asset.strip_prefix(self.file.path.parent().unwrap()).ok())
            .filter_map(|filename| filename.to_str())
            .map(|filename| {
                let mut path = self.file.path.clone();
                // Popping the index.md from the path since file.parent would be one level too high
                // for our need here
                path.pop();
                path.push(filename);
                path = path
                    .strip_prefix(&base_path.join("content"))
                    .expect("Should be able to stripe prefix")
                    .to_path_buf();
                path
            })
            .map(|path| format!("/{}", path.display()))
            .collect()
    }

    pub fn has_anchor(&self, anchor: &str) -> bool {
        has_anchor(&self.toc, anchor)
    }

    pub fn has_anchor_id(&self, id: &str) -> bool {
        has_anchor_id(&self.content, id)
    }

    pub fn serialize<'a>(&'a self, library: &'a Library) -> SerializingPage<'a> {
        SerializingPage::new(self, Some(library), true)
    }

    pub fn serialize_without_siblings<'a>(&'a self, library: &'a Library) -> SerializingPage<'a> {
        SerializingPage::new(self, Some(library), false)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs::{create_dir, File};
    use std::io::Write;
    use std::path::{Path, PathBuf};

    use libs::globset::{Glob, GlobSetBuilder};
    use libs::tera::Tera;
    use tempfile::tempdir;

    use crate::Page;
    use config::{Config, LanguageOptions};
    use utils::slugs::SlugifyStrategy;
    use utils::types::InsertAnchor;

    #[test]
    fn can_parse_a_valid_page() {
        let config = Config::default_for_test();
        let content = r#"
+++
title = "Hello"
description = "hey there"
slug = "hello-world"
+++
Hello world"#;
        let res = Page::parse(Path::new("post.md"), content, &config, &PathBuf::new());
        assert!(res.is_ok());
        let mut page = res.unwrap();
        page.render_markdown(
            &HashMap::default(),
            &Tera::default(),
            &config,
            InsertAnchor::None,
            &HashMap::new(),
        )
        .unwrap();

        assert_eq!(page.meta.title.unwrap(), "Hello".to_string());
        assert_eq!(page.meta.slug.unwrap(), "hello-world".to_string());
        assert_eq!(page.raw_content, "Hello world".to_string());
        assert_eq!(page.content, "<p>Hello world</p>\n".to_string());
    }

    #[test]
    fn can_parse_author() {
        let config = Config::default_for_test();
        let content = r#"
+++
title = "Hello"
description = "hey there"
authors = ["person@example.com (A. Person)"]
+++
Hello world"#;
        let res = Page::parse(Path::new("post.md"), content, &config, &PathBuf::new());
        assert!(res.is_ok());
        let mut page = res.unwrap();
        page.render_markdown(
            &HashMap::default(),
            &Tera::default(),
            &config,
            InsertAnchor::None,
            &HashMap::new(),
        )
        .unwrap();

        assert_eq!(1, page.meta.authors.len());
        assert_eq!("person@example.com (A. Person)", page.meta.authors.get(0).unwrap());
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
        let res =
            Page::parse(Path::new("content/posts/intro/start.md"), content, &conf, &PathBuf::new());
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.path, "/posts/intro/hello-world/");
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
        let res = Page::parse(Path::new("start.md"), content, &config, &PathBuf::new());
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.path, "/hello-world/");
        assert_eq!(page.components, vec!["hello-world"]);
        assert_eq!(page.permalink, config.make_permalink("hello-world"));
    }

    #[test]
    fn can_make_url_from_slug_only_with_no_special_chars() {
        let content = r#"
    +++
    slug = "hello-&-world"
    +++
    Hello world"#;
        let mut config = Config::default();
        config.slugify.paths = SlugifyStrategy::On;
        let res = Page::parse(Path::new("start.md"), content, &config, &PathBuf::new());
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.path, "/hello-world/");
        assert_eq!(page.components, vec!["hello-world"]);
        assert_eq!(page.permalink, config.make_permalink("hello-world"));
    }

    #[test]
    fn can_make_url_from_utf8_slug_frontmatter() {
        let content = r#"
    +++
    slug = "日本"
    +++
    Hello world"#;
        let mut config = Config::default();
        config.slugify.paths = SlugifyStrategy::Safe;
        let res = Page::parse(Path::new("start.md"), content, &config, &PathBuf::new());
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.path, "/日本/");
        assert_eq!(page.components, vec!["日本"]);
        assert_eq!(page.permalink, config.make_permalink("日本"));
    }

    #[test]
    fn can_make_url_from_path() {
        let content = r#"
    +++
    path = "hello-world"
    +++
    Hello world"#;
        let config = Config::default();
        let res = Page::parse(
            Path::new("content/posts/intro/start.md"),
            content,
            &config,
            &PathBuf::new(),
        );
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.path, "/hello-world/");
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
        let res = Page::parse(
            Path::new("content/posts/intro/start.md"),
            content,
            &config,
            &PathBuf::new(),
        );
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.path, "/hello-world/");
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
        let res = Page::parse(Path::new("start.md"), content, &Config::default(), &PathBuf::new());
        assert!(res.is_err());
    }

    #[test]
    fn can_make_slug_from_non_slug_filename() {
        let mut config = Config::default();
        config.slugify.paths = SlugifyStrategy::On;
        let res =
            Page::parse(Path::new(" file with space.md"), "+++\n+++\n", &config, &PathBuf::new());
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.slug, "file-with-space");
        assert_eq!(page.permalink, config.make_permalink(&page.slug));
    }

    #[test]
    fn can_make_path_from_utf8_filename() {
        let mut config = Config::default();
        config.slugify.paths = SlugifyStrategy::Safe;
        let res = Page::parse(Path::new("日本.md"), "+++\n+++\n", &config, &PathBuf::new());
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.slug, "日本");
        assert_eq!(page.permalink, config.make_permalink(&page.slug));
    }

    #[test]
    fn can_specify_summary() {
        let config = Config::default_for_test();
        let content = r#"
+++
+++
Hello world
<!-- more -->"#
            .to_string();
        let res = Page::parse(Path::new("hello.md"), &content, &config, &PathBuf::new());
        assert!(res.is_ok());
        let mut page = res.unwrap();
        page.render_markdown(
            &HashMap::default(),
            &Tera::default(),
            &config,
            InsertAnchor::None,
            &HashMap::new(),
        )
        .unwrap();
        assert_eq!(page.summary, Some("<p>Hello world</p>\n".to_string()));
    }

    #[test]
    fn strips_footnotes_in_summary() {
        let config = Config::default_for_test();
        let content = r#"
+++
+++
This page use <sup>1.5</sup> and has footnotes, here's one. [^1]

Here's another. [^2]

<!-- more -->

And here's another. [^3]

[^1]: This is the first footnote.

[^2]: This is the secund footnote.

[^3]: This is the third footnote."#
            .to_string();
        let res = Page::parse(Path::new("hello.md"), &content, &config, &PathBuf::new());
        assert!(res.is_ok());
        let mut page = res.unwrap();
        page.render_markdown(
            &HashMap::default(),
            &Tera::default(),
            &config,
            InsertAnchor::None,
            &HashMap::new(),
        )
        .unwrap();
        assert_eq!(
            page.summary,
            Some("<p>This page use <sup>1.5</sup> and has footnotes, here\'s one. </p>\n<p>Here's another. </p>\n".to_string())
        );
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

        let res = Page::from_file(nested_path.join("index.md").as_path(), &Config::default(), path);
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.file.parent, path.join("content").join("posts"));
        assert_eq!(page.slug, "with-assets");
        assert_eq!(page.assets.len(), 3);
        assert!(page.serialized_assets[0].starts_with('/'));
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

        let res = Page::from_file(nested_path.join("index.md").as_path(), &Config::default(), path);
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.file.parent, path.join("content").join("posts"));
        assert_eq!(page.slug, "hey");
        assert_eq!(page.assets.len(), 3);
        assert_eq!(page.permalink, "http://a-website.com/posts/hey/");
    }

    // https://github.com/getzola/zola/issues/674
    #[test]
    fn page_with_assets_uses_filepath_for_assets() {
        let tmp_dir = tempdir().expect("create temp dir");
        let path = tmp_dir.path();
        create_dir(&path.join("content")).expect("create content temp dir");
        create_dir(&path.join("content").join("posts")).expect("create posts temp dir");
        let nested_path = path.join("content").join("posts").join("with_assets");
        create_dir(&nested_path).expect("create nested temp dir");
        let mut f = File::create(nested_path.join("index.md")).unwrap();
        f.write_all(b"+++\n+++\n").unwrap();
        File::create(nested_path.join("example.js")).unwrap();
        File::create(nested_path.join("graph.jpg")).unwrap();
        File::create(nested_path.join("fail.png")).unwrap();

        let res = Page::from_file(nested_path.join("index.md").as_path(), &Config::default(), path);
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.file.parent, path.join("content").join("posts"));
        assert_eq!(page.assets.len(), 3);
        assert_eq!(page.serialized_assets.len(), 3);
        // We should not get with-assets since that's the slugified version
        assert!(page.serialized_assets[0].contains("with_assets"));
        assert_eq!(page.permalink, "http://a-website.com/posts/with-assets/");
    }

    // https://github.com/getzola/zola/issues/607
    #[test]
    fn page_with_assets_and_date_in_folder_name() {
        let tmp_dir = tempdir().expect("create temp dir");
        let path = tmp_dir.path();
        create_dir(&path.join("content")).expect("create content temp dir");
        create_dir(&path.join("content").join("posts")).expect("create posts temp dir");
        let nested_path = path.join("content").join("posts").join("2013-06-02_with-assets");
        create_dir(&nested_path).expect("create nested temp dir");
        let mut f = File::create(nested_path.join("index.md")).unwrap();
        f.write_all(b"+++\n\n+++\n").unwrap();
        File::create(nested_path.join("example.js")).unwrap();
        File::create(nested_path.join("graph.jpg")).unwrap();
        File::create(nested_path.join("fail.png")).unwrap();

        let res = Page::from_file(nested_path.join("index.md").as_path(), &Config::default(), path);
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.file.parent, path.join("content").join("posts"));
        assert_eq!(page.slug, "with-assets");
        assert_eq!(page.meta.date, Some("2013-06-02".to_string()));
        assert_eq!(page.assets.len(), 3);
        assert_eq!(page.permalink, "http://a-website.com/posts/with-assets/");
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

        let res = Page::from_file(nested_path.join("index.md").as_path(), &config, path);

        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.assets.len(), 1);
        assert_eq!(page.assets[0].file_name().unwrap().to_str(), Some("graph.jpg"));
    }

    // https://github.com/getzola/zola/issues/1566
    #[test]
    fn colocated_page_with_slug_and_date_in_path() {
        let tmp_dir = tempdir().expect("create temp dir");
        let path = tmp_dir.path();
        create_dir(&path.join("content")).expect("create content temp dir");
        let articles_path = path.join("content").join("articles");
        create_dir(&articles_path).expect("create posts temp dir");

        let config = Config::default();

        // first a non-colocated one
        let file_path = articles_path.join("2021-07-29-sample-article-1.md");
        let mut f = File::create(&file_path).unwrap();
        f.write_all(b"+++\nslug=\"hey\"\n+++\n").unwrap();
        let res = Page::from_file(&file_path, &config, path);
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.path, "/articles/hey/");

        // then a colocated one, it should still work
        let dir_path = articles_path.join("2021-07-29-sample-article-2.md");
        create_dir(&dir_path).expect("create posts temp dir");
        let mut f = File::create(&dir_path.join("index.md")).unwrap();
        f.write_all(b"+++\nslug=\"ho\"\n+++\n").unwrap();
        let res = Page::from_file(&dir_path.join("index.md"), &config, path);
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.path, "/articles/ho/");
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
        let res = Page::parse(Path::new("2018-10-08_hello.md"), &content, &config, &PathBuf::new());
        assert!(res.is_ok());
        let page = res.unwrap();

        assert_eq!(page.meta.date, Some("2018-10-08".to_string()));
        assert_eq!(page.slug, "hello");
    }

    // https://github.com/getzola/zola/pull/1323#issuecomment-779401063
    #[test]
    fn can_get_date_from_short_date_in_filename_respects_slugification_strategy() {
        let mut config = Config::default();
        config.slugify.paths = SlugifyStrategy::Off;
        let content = r#"
+++
+++
Hello world
<!-- more -->"#
            .to_string();
        let res =
            Page::parse(Path::new("2018-10-08_ こんにちは.md"), &content, &config, &PathBuf::new());
        assert!(res.is_ok());
        let page = res.unwrap();

        assert_eq!(page.meta.date, Some("2018-10-08".to_string()));
        assert_eq!(page.slug, " こんにちは");
    }

    #[test]
    fn can_get_date_from_filename_with_spaces() {
        let config = Config::default();
        let content = r#"
+++
+++
Hello world
<!-- more -->"#
            .to_string();
        let res =
            Page::parse(Path::new("2018-10-08 - hello.md"), &content, &config, &PathBuf::new());
        assert!(res.is_ok());
        let page = res.unwrap();

        assert_eq!(page.meta.date, Some("2018-10-08".to_string()));
        assert_eq!(page.slug, "hello");
    }

    #[test]
    fn can_get_date_from_filename_with_spaces_respects_slugification() {
        let mut config = Config::default();
        config.slugify.paths = SlugifyStrategy::Off;
        let content = r#"
+++
+++
Hello world
<!-- more -->"#
            .to_string();
        let res =
            Page::parse(Path::new("2018-10-08 - hello.md"), &content, &config, &PathBuf::new());
        assert!(res.is_ok());
        let page = res.unwrap();

        assert_eq!(page.meta.date, Some("2018-10-08".to_string()));
        assert_eq!(page.slug, " hello");
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
        let res = Page::parse(
            Path::new("2018-10-02T15:00:00Z-hello.md"),
            &content,
            &config,
            &PathBuf::new(),
        );
        assert!(res.is_ok());
        let page = res.unwrap();

        assert_eq!(page.meta.date, Some("2018-10-02T15:00:00Z".to_string()));
        assert_eq!(page.slug, "hello");
    }

    // https://github.com/getzola/zola/pull/1323#issuecomment-779401063
    #[test]
    fn can_get_date_from_full_rfc3339_date_in_filename_respects_slugification_strategy() {
        let mut config = Config::default();
        config.slugify.paths = SlugifyStrategy::Off;
        let content = r#"
+++
+++
Hello world
<!-- more -->"#
            .to_string();
        let res = Page::parse(
            Path::new("2018-10-02T15:00:00Z- こんにちは.md"),
            &content,
            &config,
            &PathBuf::new(),
        );
        assert!(res.is_ok());
        let page = res.unwrap();

        assert_eq!(page.meta.date, Some("2018-10-02T15:00:00Z".to_string()));
        assert_eq!(page.slug, " こんにちは");
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
        let res = Page::parse(Path::new("2018-10-08_hello.md"), &content, &config, &PathBuf::new());
        assert!(res.is_ok());
        let page = res.unwrap();

        assert_eq!(page.meta.date, Some("2018-09-09".to_string()));
        assert_eq!(page.slug, "hello");
    }

    #[test]
    fn can_specify_language_in_filename() {
        let mut config = Config::default();
        config.languages.insert("fr".to_owned(), LanguageOptions::default());
        let content = r#"
+++
+++
Bonjour le monde"#
            .to_string();
        let res = Page::parse(Path::new("hello.fr.md"), &content, &config, &PathBuf::new());
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.lang, "fr".to_string());
        assert_eq!(page.slug, "hello");
        assert_eq!(page.permalink, "http://a-website.com/fr/hello/");
    }

    #[test]
    fn can_specify_language_in_filename_with_date() {
        let mut config = Config::default();
        config.languages.insert("fr".to_owned(), LanguageOptions::default());
        let content = r#"
+++
+++
Bonjour le monde"#
            .to_string();
        let res =
            Page::parse(Path::new("2018-10-08_hello.fr.md"), &content, &config, &PathBuf::new());
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.meta.date, Some("2018-10-08".to_string()));
        assert_eq!(page.lang, "fr".to_string());
        assert_eq!(page.slug, "hello");
        assert_eq!(page.permalink, "http://a-website.com/fr/hello/");
    }

    #[test]
    fn i18n_frontmatter_path_overrides_default_permalink() {
        let mut config = Config::default();
        config.languages.insert("fr".to_owned(), LanguageOptions::default());
        let content = r#"
+++
path = "bonjour"
+++
Bonjour le monde"#
            .to_string();
        let res = Page::parse(Path::new("hello.fr.md"), &content, &config, &PathBuf::new());
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.lang, "fr".to_string());
        assert_eq!(page.slug, "hello");
        assert_eq!(page.permalink, "http://a-website.com/bonjour/");
    }
}
