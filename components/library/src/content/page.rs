/// A page, can be a blog post or a basic page
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use slotmap::DefaultKey;
use tera::{Context as TeraContext, Tera};
use unic_langid::LanguageIdentifier;

use crate::library::Library;
use config::Config;
use errors::{bail, Error, Result};
use front_matter::{split_page_content, InsertAnchor, PageFrontMatter};
use rendering::{render_content, Heading, RenderContext};
use utils::fs::{find_related_assets, read_file};
use utils::site::get_reading_analytics;
use utils::templates::render_template;

use crate::content::file_info::FileInfo;
use crate::content::has_anchor;
use crate::content::ser::SerializingPage;
use utils::slugs::slugify_paths;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Page {
    /// All info about the actual file
    pub file: FileInfo,
    /// The front matter meta-data
    pub meta: PageFrontMatter,
    /// The list of parent sections
    pub ancestors: Vec<DefaultKey>,
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
    /// The earlier page, for pages sorted by date
    pub earlier: Option<DefaultKey>,
    /// The later page, for pages sorted by date
    pub later: Option<DefaultKey>,
    /// The lighter page, for pages sorted by weight
    pub lighter: Option<DefaultKey>,
    /// The heavier page, for pages sorted by weight
    pub heavier: Option<DefaultKey>,
    /// Toc made from the headings of the markdown file
    pub toc: Vec<Heading>,
    /// How many words in the raw content
    pub word_count: Option<usize>,
    /// How long would it take to read the raw content.
    /// See `get_reading_analytics` on how it is calculated
    pub reading_time: Option<usize>,
    /// The language of that page. Equal to the default lang if the user doesn't setup `languages` in config.
    pub lang: LanguageIdentifier,
    /// How `lang` is to be displayed in filenames/URLs
    /// Corresponds to the lang in the {slug}.{lang}.md naming scheme, and set to
    /// `lang.to_string()` if not set.
    pub language_alias: String,
    /// Contains all the translated version of that page
    pub translations: Vec<DefaultKey>,
    /// Contains the internal links that have an anchor: we can only check the anchor
    /// after all pages have been built and their ToC compiled. The page itself should exist otherwise
    /// it would have errored before getting there
    /// (path to markdown, anchor value)
    pub internal_links_with_anchors: Vec<(String, String)>,
    /// Contains the external links that need to be checked
    pub external_links: Vec<String>,
}

impl Page {
    pub fn new<P: AsRef<Path>>(file_path: P, meta: PageFrontMatter, base_path: &PathBuf) -> Page {
        let file_path = file_path.as_ref();

        Page { file: FileInfo::new_page(file_path, base_path), meta, ..Self::default() }
    }

    pub fn is_draft(&self) -> bool {
        self.meta.draft
    }

    /// Parse a page given the content of the .md file
    /// Files without front matter or with invalid front matter are considered
    /// erroneous
    pub fn parse(
        file_path: &Path,
        content: &str,
        config: &Config,
        base_path: &PathBuf,
    ) -> Result<Page> {
        let (meta, content) = split_page_content(file_path, content)?;
        let mut page = Page::new(file_path, meta, base_path);

        if let Some(ref l) = page.file.maybe_lang {
            if !config.is_multilingual() {
                // TODO: add test
                bail!(
                    "Page `{}` has a language set, but `languages` in config.toml is empty. Note that file names must not contain dots.",
                    file_path.display()
                );
            } else if l.is_empty() {
                // TODO: add test
                bail!("Page `{}` has an empty language. Did you mistakenly put 2 dots before the extension?", file_path.display());
            } else if l == &config.default_language_options.language_alias {
                bail!("Pages in the default language should not set it in their title (caused by `{}`).", file_path.display());
            } else {
                page.language_alias = l.clone();
                // When writing tests for this, please make sure that `language_alias` has been manually
                // set for each element in `config.languages`
                page.lang = config
                    .get_language_identifier(&l)
                    .ok_or_else(||
                        format!("File `{}` has a language of `{}` which isn't present in config.toml. Not that if a `language_alias` was specified, you must use that.\nHint: Possible values are {:?}.", file_path.display(), l, config.language_aliases())
                        )?.clone();
            }
        } else {
            page.lang = config.default_language.clone();
            page.language_alias = config.default_language_options.language_alias.clone();
        }

        // Catch programming errors
        // `lang` and `language_alias` must be non-empty
        assert_ne!(page.lang, LanguageIdentifier::default());
        assert!(!page.language_alias.is_empty());

        page.raw_content = content.to_string();
        let (word_count, reading_time) = get_reading_analytics(&page.raw_content);
        page.word_count = Some(word_count);
        page.reading_time = Some(reading_time);

        if let Some(ref date) = page.file.date {
            if page.meta.date.is_none() {
                // TODO: do not fail silently on invalid formats
                page.meta.date = Some(date.clone());
                page.meta.date_to_datetime();
            }
        }

        page.slug = {
            if let Some(ref slug) = page.meta.slug {
                slugify_paths(slug, config.slugify.paths)
            } else if page.file.name == "index" {
                if let Some(parent) = page.file.path.parent() {
                    slugify_paths(
                        &super::file_info::clear_filename(
                            parent.file_name().unwrap().to_str().unwrap(),
                        ),
                        config.slugify.paths,
                    )
                } else {
                    slugify_paths(&page.file.name, config.slugify.paths)
                }
            } else {
                slugify_paths(&page.file.name, config.slugify.paths)
            }
        };
        assert!(!page.slug.is_empty());

        page.path = if let Some(ref p) = page.meta.path {
            let path = p.trim();

            assert!(!path.is_empty());
            if path.starts_with('/') {
                path.into()
            } else {
                format!("/{}", path)
            }
        } else {
            let mut path = if page.file.components.is_empty() {
                page.slug.clone()
            } else {
                format!("{}/{}", page.file.components.join("/"), page.slug)
            };

            if page.lang != config.default_language {
                path = format!("{}/{}", page.language_alias, path);
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

    /// Read and parse a .md file into a Page struct
    pub fn from_file<P: AsRef<Path>>(
        path: P,
        config: &Config,
        base_path: &PathBuf,
    ) -> Result<Page> {
        let path = path.as_ref();
        let content = read_file(path)?;
        let mut page = Page::parse(path, &content, config, base_path)?;

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
                        None => false,
                        Some(file) => !globset.is_match(file),
                    })
                    .collect();
            } else {
                page.assets = assets;
            }

            page.serialized_assets = page.serialize_assets(&base_path);
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
            &self.lang,
        );

        context.tera_context.insert("page", &SerializingPage::from_page_basic(self, None));

        let res = render_content(&self.raw_content, &context).map_err(|e| {
            Error::chain(format!("Failed to render content of {}", self.file.path.display()), e)
        })?;

        self.summary = res.summary_len.map(|l| res.body[0..l].to_owned());
        self.content = res.body;
        self.toc = res.toc;
        self.external_links = res.external_links;
        self.internal_links_with_anchors = res.internal_links_with_anchors;

        Ok(())
    }

    /// Renders the page using the default layout, unless specified in front-matter
    pub fn render_html(&self, tera: &Tera, config: &Config, library: &Library) -> Result<String> {
        let tpl_name = match self.meta.template {
            Some(ref l) => l,
            None => "page.html",
        };

        let mut context = TeraContext::new();
        context.insert("config", &config.get_localized(&self.lang).expect("`lang` in config"));
        context.insert("current_url", &self.permalink);
        context.insert("current_path", &self.path);
        context.insert("page", &self.to_serialized(library));
        context.insert("lang", &self.lang);
        context.insert("language_alias", &self.language_alias);

        render_template(&tpl_name, tera, &context, &config.theme).map_err(|e| {
            Error::chain(format!("Failed to render page '{}'", self.file.path.display()), e)
        })
    }

    /// Creates a vectors of asset URLs.
    fn serialize_assets(&self, base_path: &PathBuf) -> Vec<String> {
        self.assets
            .iter()
            .filter_map(|asset| asset.file_name())
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
            .map(|path| path.to_string_lossy().to_string())
            .collect()
    }

    pub fn has_anchor(&self, anchor: &str) -> bool {
        has_anchor(&self.toc, anchor)
    }

    pub fn to_serialized<'a>(&'a self, library: &'a Library) -> SerializingPage<'a> {
        SerializingPage::from_page(self, library)
    }

    pub fn to_serialized_basic<'a>(&'a self, library: &'a Library) -> SerializingPage<'a> {
        SerializingPage::from_page_basic(self, Some(library))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs::{create_dir, File};
    use std::io::Write;
    use std::path::{Path, PathBuf};

    use globset::{Glob, GlobSetBuilder};
    use tempfile::tempdir;
    use tera::Tera;
    use unic_langid::langid;

    use super::Page;
    use config::{Config, LocaleOptions};
    use front_matter::InsertAnchor;
    use utils::slugs::SlugifyStrategy;

    #[test]
    fn test_can_parse_a_valid_page() {
        let content = r#"
+++
title = "Hello"
description = "hey there"
slug = "hello-world"
+++
Hello world"#;
        let res = Page::parse(Path::new("post.md"), content, &Config::default(), &PathBuf::new());
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
            Page::parse(Path::new(" file with space.md"), "+++\n+++", &config, &PathBuf::new());
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.slug, "file-with-space");
        assert_eq!(page.permalink, config.make_permalink(&page.slug));
    }

    #[test]
    fn can_make_slug_from_non_slug_filename_with_lang_and_date() {
        let mut config = Config::default();
        config.slugify.paths = SlugifyStrategy::On;
        let mut fr = LocaleOptions::default();
        fr.language_alias = "fr".to_string();
        config.languages.insert(langid!("fr"), fr);

        let res = Page::parse(
            Path::new("2020-08-14- file with space.fr.md"),
            "+++\n+++",
            &config,
            &PathBuf::new(),
        );
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.slug, "file-with-space");
        assert_eq!(page.lang, langid!("fr"));
        assert_eq!(page.meta.date, Some("2020-08-14".to_string()));
        assert_eq!(page.permalink, "http://a-website.com/fr/file-with-space/");
    }

    #[test]
    fn can_make_path_from_utf8_filename() {
        let mut config = Config::default();
        config.slugify.paths = SlugifyStrategy::Safe;
        let res = Page::parse(Path::new("日本.md"), "+++\n++++", &config, &PathBuf::new());
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.slug, "日本");
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
        let res = Page::parse(Path::new("hello.md"), &content, &config, &PathBuf::new());
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

        let res = Page::from_file(
            nested_path.join("index.md").as_path(),
            &Config::default(),
            &path.to_path_buf(),
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
            &path.to_path_buf(),
        );
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

        let res = Page::from_file(
            nested_path.join("index.md").as_path(),
            &Config::default(),
            &path.to_path_buf(),
        );
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

        let res = Page::from_file(
            nested_path.join("index.md").as_path(),
            &Config::default(),
            &path.to_path_buf(),
        );
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

        let res =
            Page::from_file(nested_path.join("index.md").as_path(), &config, &path.to_path_buf());

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
        let res = Page::parse(Path::new("2018-10-08_hello.md"), &content, &config, &PathBuf::new());
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
        config.languages.insert(
            langid!("fr"),
            LocaleOptions { language_alias: "fr".to_string(), ..LocaleOptions::default() },
        );
        let content = r#"
+++
+++
Bonjour le monde"#
            .to_string();
        let res = Page::parse(Path::new("hello.fr.md"), &content, &config, &PathBuf::new());
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.lang, langid!("fr"));
        assert_eq!(page.slug, "hello");
        assert_eq!(page.permalink, "http://a-website.com/fr/hello/");
    }

    #[test]
    fn can_specify_language_in_filename_with_date() {
        let mut config = Config::default();
        config.languages.insert(
            langid!("fr"),
            LocaleOptions { language_alias: "fr".to_string(), ..LocaleOptions::default() },
        );
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
        assert_eq!(page.lang, langid!("fr"));
        assert_eq!(page.slug, "hello");
        assert_eq!(page.permalink, "http://a-website.com/fr/hello/");
    }

    #[test]
    fn i18n_frontmatter_path_overrides_default_permalink() {
        let mut config = Config::default();
        config.languages.insert(
            langid!("fr"),
            LocaleOptions { language_alias: "fr".to_string(), ..LocaleOptions::default() },
        );
        let content = r#"
+++
path = "bonjour"
+++
Bonjour le monde"#
            .to_string();
        let res = Page::parse(Path::new("hello.fr.md"), &content, &config, &PathBuf::new());
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.lang, langid!("fr"));
        assert_eq!(page.slug, "hello");
        assert_eq!(page.permalink, "http://a-website.com/bonjour/");
    }
}
