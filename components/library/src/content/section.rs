use std::collections::HashMap;
use std::path::{Path, PathBuf};

use slotmap::DefaultKey;
use tera::{Context as TeraContext, Tera};

use config::Config;
use errors::{Error, Result};
use front_matter::{split_section_content, SectionFrontMatter};
use rendering::{render_content, Heading, RenderContext};
use utils::fs::{find_related_assets, read_file};
use utils::site::get_reading_analytics;
use utils::templates::render_template;

use crate::content::file_info::FileInfo;
use crate::content::has_anchor;
use crate::content::ser::SerializingSection;
use crate::library::Library;

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
    /// All the non-md files we found next to the .md file as string for use in templates
    pub serialized_assets: Vec<String>,
    /// All direct pages of that section
    pub pages: Vec<DefaultKey>,
    /// All pages that cannot be sorted in this section
    pub ignored_pages: Vec<DefaultKey>,
    /// The list of parent sections
    pub ancestors: Vec<DefaultKey>,
    /// All direct subsections
    pub subsections: Vec<DefaultKey>,
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
    /// Contains the internal links that have an anchor: we can only check the anchor
    /// after all pages have been built and their ToC compiled. The page itself should exist otherwise
    /// it would have errored before getting there
    /// (path to markdown, anchor value)
    pub internal_links_with_anchors: Vec<(String, String)>,
    /// Contains the external links that need to be checked
    pub external_links: Vec<String>,
}

impl Section {
    pub fn new<P: AsRef<Path>>(
        file_path: P,
        meta: SectionFrontMatter,
        base_path: &PathBuf,
    ) -> Section {
        let file_path = file_path.as_ref();

        Section { file: FileInfo::new_section(file_path, base_path), meta, ..Self::default() }
    }

    pub fn parse(
        file_path: &Path,
        content: &str,
        config: &Config,
        base_path: &PathBuf,
    ) -> Result<Section> {
        let (meta, content) = split_section_content(file_path, content)?;
        let mut section = Section::new(file_path, meta, base_path);
        section.lang = section.file.find_language(config)?;
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
        base_path: &PathBuf,
    ) -> Result<Section> {
        let path = path.as_ref();
        let content = read_file(path)?;
        let mut section = Section::parse(path, &content, config, base_path)?;

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
            section.assets = assets
                .into_iter()
                .filter(|path| match path.file_name() {
                    None => false,
                    Some(file) => !globset.is_match(file),
                })
                .collect();
        } else {
            section.assets = assets;
        }

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
    ) -> Result<()> {
        let mut context = RenderContext::new(
            tera,
            config,
            &self.permalink,
            permalinks,
            self.meta.insert_anchor_links,
        );

        context.tera_context.insert("section", &SerializingSection::from_section_basic(self, None));

        let res = render_content(&self.raw_content, &context).map_err(|e| {
            Error::chain(format!("Failed to render content of {}", self.file.path.display()), e)
        })?;
        self.content = res.body;
        self.toc = res.toc;
        self.external_links = res.external_links;
        self.internal_links_with_anchors = res.internal_links_with_anchors;

        Ok(())
    }

    /// Renders the page using the default layout, unless specified in front-matter
    pub fn render_html(&self, tera: &Tera, config: &Config, library: &Library) -> Result<String> {
        let tpl_name = self.get_template_name();

        let mut context = TeraContext::new();
        context.insert("config", config);
        context.insert("current_url", &self.permalink);
        context.insert("current_path", &self.path);
        context.insert("section", &self.to_serialized(library));
        context.insert("lang", &self.lang);

        render_template(tpl_name, tera, context, &config.theme).map_err(|e| {
            Error::chain(format!("Failed to render section '{}'", self.file.path.display()), e)
        })
    }

    /// Is this the index section?
    pub fn is_index(&self) -> bool {
        self.file.components.is_empty()
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

    pub fn has_anchor(&self, anchor: &str) -> bool {
        has_anchor(&self.toc, anchor)
    }

    pub fn to_serialized<'a>(&'a self, library: &'a Library) -> SerializingSection<'a> {
        SerializingSection::from_section(self, library)
    }

    pub fn to_serialized_basic<'a>(&'a self, library: &'a Library) -> SerializingSection<'a> {
        SerializingSection::from_section_basic(self, Some(library))
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
}

#[cfg(test)]
mod tests {
    use std::fs::{create_dir, File};
    use std::io::Write;
    use std::path::{Path, PathBuf};

    use globset::{Glob, GlobSetBuilder};
    use tempfile::tempdir;

    use super::Section;
    use config::{Config, Language};

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
        assert_eq!(section.permalink, "http://a-website.com/posts/with-assets/");
    }

    #[test]
    fn section_with_ignored_assets_filters_out_correct_files() {
        let tmp_dir = tempdir().expect("create temp dir");
        let path = tmp_dir.path();
        create_dir(&path.join("content")).expect("create content temp dir");
        create_dir(&path.join("content").join("posts")).expect("create posts temp dir");
        let nested_path = path.join("content").join("posts").join("with-assets");
        create_dir(&nested_path).expect("create nested temp dir");
        let mut f = File::create(nested_path.join("_index.md")).unwrap();
        f.write_all(b"+++\nslug=\"hey\"\n+++\n").unwrap();
        File::create(nested_path.join("example.js")).unwrap();
        File::create(nested_path.join("graph.jpg")).unwrap();
        File::create(nested_path.join("fail.png")).unwrap();

        let mut gsb = GlobSetBuilder::new();
        gsb.add(Glob::new("*.{js,png}").unwrap());
        let mut config = Config::default();
        config.ignored_content_globset = Some(gsb.build().unwrap());

        let res =
            Section::from_file(nested_path.join("_index.md").as_path(), &config, &PathBuf::new());

        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.assets.len(), 1);
        assert_eq!(page.assets[0].file_name().unwrap().to_str(), Some("graph.jpg"));
    }

    #[test]
    fn can_specify_language_in_filename() {
        let mut config = Config::default();
        config.languages.push(Language { code: String::from("fr"), feed: false, search: false });
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
        config.languages.push(Language { code: String::from("fr"), feed: false, search: false });
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
        config.languages.push(Language { code: String::from("fr"), feed: false, search: false });
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
