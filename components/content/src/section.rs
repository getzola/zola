use std::collections::HashMap;
use std::path::{Path, PathBuf};

use libs::tera::{Context as TeraContext, Tera};

use config::Config;
use errors::{Context, Result};
use markdown::{render_content, RenderContext};
use utils::fs::read_file;
use utils::table_of_contents::Heading;
use utils::templates::{render_template, ShortcodeDefinition};

use crate::file_info::FileInfo;
use crate::front_matter::{split_section_content, SectionFrontMatter};
use crate::library::Library;
use crate::ser::{SectionSerMode, SerializingSection};
use crate::utils::{find_related_assets, get_reading_analytics, has_anchor};
use crate::Taxonomy;

// Default is used to create a default index section if there is no _index.md in the root content directory
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Section {
    /// All info about the actual file
    pub file: FileInfo,
    /// The front matter meta-data
    pub meta: SectionFrontMatter,
    /// The URL path of the page, always starting with a slash
    pub path: String,
    /// The components for the path of that page
    pub components: Vec<String>,
    /// The full URL for that page
    pub permalink: String,
    /// The actual content of the page, in markdown
    pub raw_content: String,
    /// The HTML rendered of the page
    pub content: String,
    /// All the non-md files we found next to the .md file
    pub assets: Vec<PathBuf>,
    /// All the non-md files we found next to the .md file as string
    pub serialized_assets: Vec<String>,
    /// All direct pages of that section
    pub pages: Vec<PathBuf>,
    /// All pages that cannot be sorted in this section
    pub ignored_pages: Vec<PathBuf>,
    /// The list of parent sections relative paths
    pub ancestors: Vec<String>,
    /// All direct subsections
    pub subsections: Vec<PathBuf>,
    /// Toc made from the headings of the markdown file
    pub toc: Vec<Heading>,
    /// How many words in the raw content
    pub word_count: Option<usize>,
    /// How long would it take to read the raw content.
    /// See `get_reading_analytics` on how it is calculated
    pub reading_time: Option<usize>,
    /// The language of that section. Equal to the default lang if the user doesn't setup `languages` in config.
    /// Corresponds to the lang in the _index.{lang}.md file scheme
    pub lang: String,
    /// The list of all internal links (as path to markdown file), with optional anchor fragments.
    /// We can only check the anchor after all pages have been built and their ToC compiled.
    /// The page itself should exist otherwise it would have errored before getting there.
    pub internal_links: Vec<(String, Option<String>)>,
    /// The list of all links to external webpages. They can be validated by the `link_checker`.
    pub external_links: Vec<String>,
    /// Taxonomies specific to the section (i.e. all pages in this section and sub-sections)
    pub taxonomies: Vec<Taxonomy>,
}

impl Section {
    pub fn new<P: AsRef<Path>>(
        file_path: P,
        meta: SectionFrontMatter,
        base_path: &Path,
    ) -> Section {
        let file_path = file_path.as_ref();

        Section { file: FileInfo::new_section(file_path, base_path), meta, ..Self::default() }
    }

    pub fn parse(
        file_path: &Path,
        content: &str,
        config: &Config,
        base_path: &Path,
    ) -> Result<Section> {
        let (meta, content) = split_section_content(file_path, content)?;
        let mut section = Section::new(file_path, meta, base_path);
        section.lang = section
            .file
            .find_language(&config.default_language, &config.other_languages_codes())?;
        section.raw_content = content.to_string();
        let (word_count, reading_time) = get_reading_analytics(&section.raw_content);
        section.word_count = Some(word_count);
        section.reading_time = Some(reading_time);

        let path = section.file.components.join("/");
        let lang_path = if section.lang != config.default_language {
            format!("/{}", section.lang)
        } else {
            "".into()
        };
        section.path = if path.is_empty() {
            format!("{}/", lang_path)
        } else {
            format!("{}/{}/", lang_path, path)
        };

        section.components = section
            .path
            .split('/')
            .map(|p| p.to_string())
            .filter(|p| !p.is_empty())
            .collect::<Vec<_>>();
        section.permalink = config.make_permalink(&section.path);
        Ok(section)
    }

    /// Read and parse a .md file into a Section struct
    pub fn from_file<P: AsRef<Path>>(
        path: P,
        config: &Config,
        base_path: &Path,
    ) -> Result<Section> {
        let path = path.as_ref();
        let content = read_file(path)?;
        let mut section = Section::parse(path, &content, config, base_path)?;

        let parent_dir = path.parent().unwrap();
        section.assets = find_related_assets(parent_dir, config, false);
        section.serialized_assets = section.serialize_assets();

        Ok(section)
    }

    pub fn get_template_name(&self) -> &str {
        match self.meta.template {
            Some(ref l) => l,
            None => {
                if self.is_index() {
                    return "index.html";
                }
                "section.html"
            }
        }
    }

    /// We need access to all pages url to render links relative to content
    /// so that can't happen at the same time as parsing
    pub fn render_markdown(
        &mut self,
        permalinks: &HashMap<String, String>,
        tera: &Tera,
        config: &Config,
        shortcode_definitions: &HashMap<String, ShortcodeDefinition>,
    ) -> Result<()> {
        let mut context = RenderContext::new(
            tera,
            config,
            &self.lang,
            &self.permalink,
            permalinks,
            self.meta.insert_anchor_links,
        );
        context.set_shortcode_definitions(shortcode_definitions);
        context.set_current_page_path(&self.file.relative);
        context
            .tera_context
            .insert("section", &SerializingSection::new(self, SectionSerMode::ForMarkdown));

        let res = render_content(&self.raw_content, &context)
            .with_context(|| format!("Failed to render content of {}", self.file.path.display()))?;
        self.content = res.body;
        self.toc = res.toc;
        self.external_links = res.external_links;
        self.internal_links = res.internal_links;

        Ok(())
    }

    /// Renders the page using the default layout, unless specified in front-matter
    pub fn render_html(&self, tera: &Tera, config: &Config, library: &Library) -> Result<String> {
        let tpl_name = self.get_template_name();

        let mut context = TeraContext::new();
        context.insert("config", &config.serialize(&self.lang));
        context.insert("current_url", &self.permalink);
        context.insert("current_path", &self.path);
        context.insert("section", &SerializingSection::new(self, SectionSerMode::Full(library)));
        context.insert("lang", &self.lang);

        render_template(tpl_name, tera, context, &config.theme)
            .with_context(|| format!("Failed to render section '{}'", self.file.path.display()))
    }

    /// Is this the index section?
    pub fn is_index(&self) -> bool {
        self.file.components.is_empty()
    }

    /// Creates a vectors of asset URLs.
    fn serialize_assets(&self) -> Vec<String> {
        self.assets
            .iter()
            .filter_map(|asset| asset.strip_prefix(&self.file.path.parent().unwrap()).ok())
            .filter_map(|filename| filename.to_str())
            .map(|filename| format!("{}{}", self.path, filename))
            .collect()
    }

    pub fn has_anchor(&self, anchor: &str) -> bool {
        has_anchor(&self.toc, anchor)
    }

    pub fn paginate_by(&self) -> Option<usize> {
        match self.meta.paginate_by {
            None => None,
            Some(x) => match x {
                0 => None,
                _ => Some(x),
            },
        }
    }

    pub fn serialize<'a>(&'a self, library: &'a Library) -> SerializingSection<'a> {
        SerializingSection::new(self, SectionSerMode::Full(library))
    }

    pub fn serialize_basic<'a>(&'a self, library: &'a Library) -> SerializingSection<'a> {
        SerializingSection::new(self, SectionSerMode::MetadataOnly(library))
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{create_dir, create_dir_all, File};
    use std::io::Write;
    use std::path::{Path, PathBuf};

    use libs::globset::{Glob, GlobSetBuilder};
    use tempfile::tempdir;

    use super::Section;
    use config::{Config, LanguageOptions};

    #[test]
    fn section_with_assets_gets_right_info() {
        let tmp_dir = tempdir().expect("create temp dir");
        let path = tmp_dir.path();
        create_dir(&path.join("content")).expect("create content temp dir");
        create_dir(&path.join("content").join("posts")).expect("create posts temp dir");
        let nested_path = path.join("content").join("posts").join("with-assets");
        create_dir(&nested_path).expect("create nested temp dir");
        let mut f = File::create(nested_path.join("_index.md")).unwrap();
        f.write_all(b"+++\n+++\n").unwrap();
        File::create(nested_path.join("example.js")).unwrap();
        File::create(nested_path.join("graph.jpg")).unwrap();
        File::create(nested_path.join("fail.png")).unwrap();

        let res = Section::from_file(
            nested_path.join("_index.md").as_path(),
            &Config::default(),
            &PathBuf::new(),
        );
        assert!(res.is_ok());
        let section = res.unwrap();
        assert_eq!(section.assets.len(), 3);
        assert!(section.serialized_assets[0].starts_with('/'));
        assert_eq!(section.permalink, "http://a-website.com/posts/with-assets/");
    }

    #[test]
    fn section_with_ignored_assets_filters_out_correct_files() {
        let tmp_dir = tempdir().expect("create temp dir");
        let path = tmp_dir.path();
        let article_path = path.join("content/posts/with-assets");
        create_dir_all(path.join(&article_path).join("foo/bar/baz/quux"))
            .expect("create nested temp dir");
        create_dir_all(path.join(&article_path).join("foo/baz/quux"))
            .expect("create nested temp dir");
        let mut f = File::create(article_path.join("_index.md")).unwrap();
        f.write_all(b"+++\nslug=\"hey\"\n+++\n").unwrap();
        File::create(article_path.join("example.js")).unwrap();
        File::create(article_path.join("graph.jpg")).unwrap();
        File::create(article_path.join("fail.png")).unwrap();
        File::create(article_path.join("foo/bar/baz/quux/quo.xlsx")).unwrap();
        File::create(article_path.join("foo/bar/baz/quux/quo.docx")).unwrap();

        let mut gsb = GlobSetBuilder::new();
        gsb.add(Glob::new("*.{js,png}").unwrap());
        gsb.add(Glob::new("foo/**/baz").unwrap());
        let mut config = Config::default();
        config.ignored_content_globset = Some(gsb.build().unwrap());

        let res =
            Section::from_file(article_path.join("_index.md").as_path(), &config, &PathBuf::new());

        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.assets.len(), 1);
        assert_eq!(page.assets[0].file_name().unwrap().to_str(), Some("graph.jpg"));
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
        let res = Section::parse(
            Path::new("content/hello/nested/_index.fr.md"),
            &content,
            &config,
            &PathBuf::new(),
        );
        assert!(res.is_ok());
        let section = res.unwrap();
        assert_eq!(section.lang, "fr".to_string());
        assert_eq!(section.permalink, "http://a-website.com/fr/hello/nested/");
    }

    // https://zola.discourse.group/t/rfc-i18n/13/17?u=keats
    #[test]
    fn can_make_links_to_translated_sections_without_double_trailing_slash() {
        let mut config = Config::default();
        config.languages.insert("fr".to_owned(), LanguageOptions::default());
        let content = r#"
+++
+++
Bonjour le monde"#
            .to_string();
        let res =
            Section::parse(Path::new("content/_index.fr.md"), &content, &config, &PathBuf::new());
        assert!(res.is_ok());
        let section = res.unwrap();
        assert_eq!(section.lang, "fr".to_string());
        assert_eq!(section.permalink, "http://a-website.com/fr/");
    }

    #[test]
    fn can_make_links_to_translated_subsections_with_trailing_slash() {
        let mut config = Config::default();
        config.languages.insert("fr".to_owned(), LanguageOptions::default());
        let content = r#"
+++
+++
Bonjour le monde"#
            .to_string();
        let res = Section::parse(
            Path::new("content/subcontent/_index.fr.md"),
            &content,
            &config,
            &PathBuf::new(),
        );
        assert!(res.is_ok());
        let section = res.unwrap();
        assert_eq!(section.lang, "fr".to_string());
        assert_eq!(section.permalink, "http://a-website.com/fr/subcontent/");
    }
}
