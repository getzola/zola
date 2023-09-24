pub mod languages;
pub mod link_checker;
pub mod markup;
pub mod search;
pub mod slugify;
pub mod taxonomies;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use libs::globset::{Glob, GlobSet, GlobSetBuilder};
use libs::toml::Value as Toml;
use serde::{Deserialize, Serialize};

use crate::theme::Theme;
use errors::{anyhow, bail, Result};
use utils::fs::read_file;
use utils::slugs::slugify_paths;

// We want a default base url for tests
static DEFAULT_BASE_URL: &str = "http://a-website.com";

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Build,
    Serve,
    Check,
}

fn build_ignore_glob_set(ignore: &Vec<String>, name: &str) -> Result<GlobSet> {
    let mut glob_set_builder = GlobSetBuilder::new();
    for pat in ignore {
        let glob = match Glob::new(pat) {
            Ok(g) => g,
            Err(e) => bail!("Invalid ignored_{} glob pattern: {}, error = {}", name, pat, e),
        };
        glob_set_builder.add(glob);
    }
    Ok(glob_set_builder.build().unwrap_or_else(|_| panic!("Bad ignored_{} in config file.", name)))
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Base URL of the site, the only required config argument
    pub base_url: String,

    /// Theme to use
    pub theme: Option<String>,
    /// Title of the site. Defaults to None
    pub title: Option<String>,
    /// Description of the site
    pub description: Option<String>,

    /// The language used in the site. Defaults to "en"
    pub default_language: String,
    /// The list of supported languages outside of the default one
    pub languages: HashMap<String, languages::LanguageOptions>,
    /// The translations strings for the default language
    translations: HashMap<String, String>,

    /// Whether to generate a feed. Defaults to false.
    pub generate_feed: bool,
    /// The number of articles to include in the feed. Defaults to including all items.
    pub feed_limit: Option<usize>,
    /// The filename to use for feeds. Used to find the template, too.
    /// Defaults to "atom.xml", with "rss.xml" also having a template provided out of the box.
    pub feed_filename: String,
    /// If set, files from static/ will be hardlinked instead of copied to the output dir.
    pub hard_link_static: bool,
    pub taxonomies: Vec<taxonomies::TaxonomyConfig>,
    /// The default author for pages.
    pub author: Option<String>,

    /// Whether to compile the `sass` directory and output the css files into the static folder
    pub compile_sass: bool,
    /// Whether to minify the html output
    pub minify_html: bool,
    /// Whether to build the search index for the content
    pub build_search_index: bool,
    /// A list of file glob patterns to ignore when processing the content folder. Defaults to none.
    /// Had to remove the PartialEq derive because GlobSet does not implement it. No impact
    /// because it's unused anyway (who wants to sort Configs?).
    pub ignored_content: Vec<String>,
    #[serde(skip_serializing, skip_deserializing)] // not a typo, 2 are needed
    pub ignored_content_globset: Option<GlobSet>,

    /// A list of file glob patterns to ignore when processing the static folder. Defaults to none.
    pub ignored_static: Vec<String>,
    #[serde(skip_serializing, skip_deserializing)] // not a typo, 2 are needed
    pub ignored_static_globset: Option<GlobSet>,

    /// The mode Zola is currently being ran on. Some logging/feature can differ depending on the
    /// command being used.
    #[serde(skip_serializing)]
    pub mode: Mode,

    pub output_dir: String,
    /// Whether dotfiles inside the output directory are preserved when rebuilding the site
    pub preserve_dotfiles_in_output: bool,

    pub link_checker: link_checker::LinkChecker,
    /// The setup for which slugification strategies to use for paths, taxonomies and anchors
    pub slugify: slugify::Slugify,
    /// The search config, telling what to include in the search index
    pub search: search::Search,
    /// The config for the Markdown rendering: syntax highlighting and everything
    pub markdown: markup::Markdown,
    /// All user params set in `[extra]` in the config
    pub extra: HashMap<String, Toml>,
}

#[derive(Serialize)]
pub struct SerializedConfig<'a> {
    base_url: &'a str,
    mode: Mode,
    title: &'a Option<String>,
    description: &'a Option<String>,
    languages: HashMap<&'a String, &'a languages::LanguageOptions>,
    default_language: &'a str,
    generate_feed: bool,
    feed_filename: &'a str,
    taxonomies: &'a [taxonomies::TaxonomyConfig],
    author: &'a Option<String>,
    build_search_index: bool,
    extra: &'a HashMap<String, Toml>,
    markdown: &'a markup::Markdown,
    search: search::SerializedSearch<'a>,
}

impl Config {
    // any extra syntax and highlight themes have been loaded and validated already by the from_file method before parsing the config
    /// Parses a string containing TOML to our Config struct
    /// Any extra parameter will end up in the extra field
    pub fn parse(content: &str) -> Result<Config> {
        let mut config: Config = match libs::toml::from_str(content) {
            Ok(c) => c,
            Err(e) => bail!(e),
        };

        if config.base_url.is_empty() || config.base_url == DEFAULT_BASE_URL {
            bail!("A base URL is required in config.toml with key `base_url`");
        }

        languages::validate_code(&config.default_language)?;
        for code in config.languages.keys() {
            languages::validate_code(code)?;
        }

        config.add_default_language()?;
        config.slugify_taxonomies();

        if !config.ignored_content.is_empty() {
            // Convert the file glob strings into a compiled glob set matcher. We want to do this once,
            // at program initialization, rather than for every page, for example. We arrange for the
            // globset matcher to always exist (even though it has to be an inside an Option at the
            // moment because of the TOML serializer); if the glob set is empty the `is_match` function
            // of the globber always returns false.
            let glob_set = build_ignore_glob_set(&config.ignored_content, "content")?;
            config.ignored_content_globset = Some(glob_set);
        }

        if !config.ignored_static.is_empty() {
            let glob_set = build_ignore_glob_set(&config.ignored_static, "static")?;
            config.ignored_static_globset = Some(glob_set);
        }

        Ok(config)
    }

    pub fn default_for_test() -> Self {
        let mut config = Config::default();
        config.add_default_language().unwrap();
        config.slugify_taxonomies();
        config
    }

    /// Parses a config file from the given path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Config> {
        let path = path.as_ref();
        let content = read_file(path)?;

        let mut config = Config::parse(&content)?;
        let config_dir = path
            .parent()
            .ok_or_else(|| anyhow!("Failed to find directory containing the config file."))?;

        // this is the step at which missing extra syntax and highlighting themes are raised as errors
        config.markdown.init_extra_syntaxes_and_highlight_themes(config_dir)?;

        Ok(config)
    }

    pub fn slugify_taxonomies(&mut self) {
        for (_, lang_options) in self.languages.iter_mut() {
            for tax_def in lang_options.taxonomies.iter_mut() {
                tax_def.slug = slugify_paths(&tax_def.name, self.slugify.taxonomies);
            }
        }
    }

    /// Makes a url, taking into account that the base url might have a trailing slash
    pub fn make_permalink(&self, path: &str) -> String {
        let trailing_bit =
            if path.ends_with('/') || path.ends_with(&self.feed_filename) || path.is_empty() {
                ""
            } else {
                "/"
            };

        // Index section with a base url that has a trailing slash
        if self.base_url.ends_with('/') && path == "/" {
            self.base_url.clone()
        } else if path == "/" {
            // index section with a base url that doesn't have a trailing slash
            format!("{}/", self.base_url)
        } else if self.base_url.ends_with('/') && path.starts_with('/') {
            format!("{}{}{}", self.base_url, &path[1..], trailing_bit)
        } else if self.base_url.ends_with('/') || path.starts_with('/') {
            format!("{}{}{}", self.base_url, path, trailing_bit)
        } else {
            format!("{}/{}{}", self.base_url, path, trailing_bit)
        }
    }

    /// Adds the default language to the list of languages if options for it are specified at base level of config.toml.
    /// If section for the same language also exists, the options at this section and base are merged and then adds it
    /// to list.
    pub fn add_default_language(&mut self) -> Result<()> {
        let mut base_language_options = languages::LanguageOptions {
            title: self.title.clone(),
            description: self.description.clone(),
            generate_feed: self.generate_feed,
            feed_filename: self.feed_filename.clone(),
            build_search_index: self.build_search_index,
            taxonomies: self.taxonomies.clone(),
            search: self.search.clone(),
            translations: self.translations.clone(),
        };

        if let Some(section_language_options) = self.languages.get(&self.default_language) {
            if base_language_options == languages::LanguageOptions::default() {
                return Ok(());
            }
            println!("Warning: config.toml contains both default language specific information at base and under section `[languages.{}]`, \
                which may cause merge conflicts. Please use only one to specify language specific information", self.default_language);
            base_language_options.merge(section_language_options)?;
        }
        self.languages.insert(self.default_language.clone(), base_language_options);

        Ok(())
    }

    /// Merges the extra data from the theme with the config extra data
    fn add_theme_extra(&mut self, theme: &Theme) -> Result<()> {
        for (key, val) in &theme.extra {
            if !self.extra.contains_key(key) {
                // The key is not overridden in site config, insert it
                self.extra.insert(key.to_string(), val.clone());
                continue;
            }
            merge(self.extra.get_mut(key).unwrap(), val)?;
        }
        Ok(())
    }

    /// Parse the theme.toml file and merges the extra data from the theme
    /// with the config extra data
    pub fn merge_with_theme(&mut self, path: PathBuf, theme_name: &str) -> Result<()> {
        let theme = Theme::from_file(&path, theme_name)?;
        self.add_theme_extra(&theme)
    }

    /// Returns all the languages settings for languages other than the default one
    pub fn other_languages(&self) -> HashMap<&str, &languages::LanguageOptions> {
        let mut others = HashMap::new();
        for (k, v) in &self.languages {
            if k == &self.default_language {
                continue;
            }
            others.insert(k.as_str(), v);
        }
        others
    }

    pub fn other_languages_codes(&self) -> Vec<&str> {
        self.languages.keys().filter(|k| *k != &self.default_language).map(|k| k.as_str()).collect()
    }

    /// Is this site using i18n?
    pub fn is_multilingual(&self) -> bool {
        !self.other_languages().is_empty()
    }

    pub fn is_in_check_mode(&self) -> bool {
        self.mode == Mode::Check
    }

    pub fn enable_serve_mode(&mut self) {
        self.mode = Mode::Serve;
    }

    pub fn enable_check_mode(&mut self) {
        self.mode = Mode::Check;
        // Disable syntax highlighting since the results won't be used and it is slow
        self.markdown.highlight_code = false;
    }

    pub fn get_translation(&self, lang: &str, key: &str) -> Result<String> {
        if let Some(options) = self.languages.get(lang) {
            options
                .translations
                .get(key)
                .ok_or_else(|| {
                    anyhow!("Translation key '{}' for language '{}' is missing", key, lang)
                })
                .map(|term| term.to_string())
        } else {
            bail!("Language '{}' not found.", lang)
        }
    }

    pub fn has_taxonomy(&self, name: &str, lang: &str) -> bool {
        if let Some(lang_options) = self.languages.get(lang) {
            lang_options.taxonomies.iter().any(|t| t.name == name)
        } else {
            false
        }
    }

    pub fn serialize(&self, lang: &str) -> SerializedConfig {
        let options = &self.languages[lang];

        SerializedConfig {
            base_url: &self.base_url,
            mode: self.mode,
            title: &options.title,
            description: &options.description,
            languages: self.languages.iter().filter(|(k, _)| k.as_str() != lang).collect(),
            default_language: &self.default_language,
            generate_feed: options.generate_feed,
            feed_filename: &options.feed_filename,
            taxonomies: &options.taxonomies,
            author: &self.author,
            build_search_index: options.build_search_index,
            extra: &self.extra,
            markdown: &self.markdown,
            search: self.search.serialize(),
        }
    }
}

// merge TOML data that can be a table, or anything else
pub fn merge(into: &mut Toml, from: &Toml) -> Result<()> {
    match (from.is_table(), into.is_table()) {
        (false, false) => {
            // These are not tables so we have nothing to merge
            Ok(())
        }
        (true, true) => {
            // Recursively merge these tables
            let into_table = into.as_table_mut().unwrap();
            for (key, val) in from.as_table().unwrap() {
                if !into_table.contains_key(key) {
                    // An entry was missing in the first table, insert it
                    into_table.insert(key.to_string(), val.clone());
                    continue;
                }
                // Two entries to compare, recurse
                merge(into_table.get_mut(key).unwrap(), val)?;
            }
            Ok(())
        }
        _ => {
            // Trying to merge a table with something else
            Err(anyhow!("Cannot merge config.toml with theme.toml because the following values have incompatibles types:\n- {}\n - {}", into, from))
        }
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            base_url: DEFAULT_BASE_URL.to_string(),
            title: None,
            description: None,
            theme: None,
            default_language: "en".to_string(),
            languages: HashMap::new(),
            generate_feed: false,
            feed_limit: None,
            feed_filename: "atom.xml".to_string(),
            hard_link_static: false,
            taxonomies: Vec::new(),
            author: None,
            compile_sass: false,
            minify_html: false,
            mode: Mode::Build,
            build_search_index: false,
            ignored_content: Vec::new(),
            ignored_content_globset: None,
            ignored_static: Vec::new(),
            ignored_static_globset: None,
            translations: HashMap::new(),
            output_dir: "public".to_string(),
            preserve_dotfiles_in_output: false,
            link_checker: link_checker::LinkChecker::default(),
            slugify: slugify::Slugify::default(),
            search: search::Search::default(),
            markdown: markup::Markdown::default(),
            extra: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::slugs::SlugifyStrategy;

    #[test]
    fn can_add_default_language_with_data_only_at_base_section() {
        let title_base = Some("Base section title".to_string());
        let description_base = Some("Base section description".to_string());

        let mut config = Config::default();
        config.title = title_base.clone();
        config.description = description_base.clone();
        config.add_default_language().unwrap();

        let default_language_options =
            config.languages.get(&config.default_language).unwrap().clone();
        assert_eq!(default_language_options.title, title_base);
        assert_eq!(default_language_options.description, description_base);
    }

    #[test]
    fn can_add_default_language_with_data_at_base_and_language_section() {
        let title_base = Some("Base section title".to_string());
        let description_lang_section = Some("Language section description".to_string());

        let mut config = Config::default();
        config.title = title_base.clone();
        config.languages.insert(
            config.default_language.clone(),
            languages::LanguageOptions {
                title: None,
                description: description_lang_section.clone(),
                generate_feed: true,
                feed_filename: config.feed_filename.clone(),
                taxonomies: config.taxonomies.clone(),
                build_search_index: false,
                search: search::Search::default(),
                translations: config.translations.clone(),
            },
        );
        config.add_default_language().unwrap();

        let default_language_options =
            config.languages.get(&config.default_language).unwrap().clone();
        assert_eq!(default_language_options.title, title_base);
        assert_eq!(default_language_options.description, description_lang_section);
    }

    #[test]
    fn errors_when_same_field_present_at_base_and_language_section() {
        let title_base = Some("Base section title".to_string());
        let title_lang_section = Some("Language section title".to_string());

        let mut config = Config::default();
        config.title = title_base.clone();
        config.languages.insert(
            config.default_language.clone(),
            languages::LanguageOptions {
                title: title_lang_section.clone(),
                description: None,
                generate_feed: true,
                feed_filename: config.feed_filename.clone(),
                taxonomies: config.taxonomies.clone(),
                build_search_index: false,
                search: search::Search::default(),
                translations: config.translations.clone(),
            },
        );
        let result = config.add_default_language();
        assert!(result.is_err());
    }

    #[test]
    fn can_import_valid_config() {
        let config = r#"
title = "My site"
base_url = "https://replace-this-with-your-url.com"
        "#;

        let config = Config::parse(config).unwrap();
        assert_eq!(config.title.unwrap(), "My site".to_string());
    }

    #[test]
    fn errors_when_invalid_type() {
        let config = r#"
title = 1
base_url = "https://replace-this-with-your-url.com"
        "#;

        let config = Config::parse(config);
        assert!(config.is_err());
    }

    #[test]
    fn errors_when_missing_required_field() {
        // base_url is required
        let config = r#"
title = ""
        "#;

        let config = Config::parse(config);
        assert!(config.is_err());
    }

    #[test]
    fn can_add_extra_values() {
        let config = r#"
title = "My site"
base_url = "https://replace-this-with-your-url.com"

[extra]
hello = "world"
        "#;

        let config = Config::parse(config);
        assert!(config.is_ok());
        assert_eq!(config.unwrap().extra.get("hello").unwrap().as_str().unwrap(), "world");
    }

    #[test]
    fn can_make_url_index_page_with_non_trailing_slash_url() {
        let config = Config { base_url: "http://vincent.is".to_string(), ..Default::default() };
        assert_eq!(config.make_permalink(""), "http://vincent.is/");
    }

    #[test]
    fn can_make_url_index_page_with_railing_slash_url() {
        let config = Config { base_url: "http://vincent.is".to_string(), ..Default::default() };
        assert_eq!(config.make_permalink(""), "http://vincent.is/");
    }

    #[test]
    fn can_make_url_with_non_trailing_slash_base_url() {
        let config = Config { base_url: "http://vincent.is".to_string(), ..Default::default() };
        assert_eq!(config.make_permalink("hello"), "http://vincent.is/hello/");
    }

    #[test]
    fn can_make_url_with_trailing_slash_path() {
        let config = Config { base_url: "http://vincent.is".to_string(), ..Default::default() };
        assert_eq!(config.make_permalink("/hello"), "http://vincent.is/hello/");
    }

    #[test]
    fn can_make_url_with_localhost() {
        let config = Config { base_url: "http://127.0.0.1:1111".to_string(), ..Default::default() };
        assert_eq!(config.make_permalink("/tags/rust"), "http://127.0.0.1:1111/tags/rust/");
    }

    // https://github.com/Keats/gutenberg/issues/486
    #[test]
    fn doesnt_add_trailing_slash_to_feed() {
        let config = Config { base_url: "http://vincent.is".to_string(), ..Default::default() };
        assert_eq!(config.make_permalink("atom.xml"), "http://vincent.is/atom.xml");
    }

    #[test]
    fn can_merge_with_theme_data_and_preserve_config_value() {
        let config_str = r#"
title = "My site"
base_url = "https://replace-this-with-your-url.com"

[extra]
hello = "world"
[extra.sub]
foo = "bar"
[extra.sub.sub]
foo = "bar"
        "#;
        let mut config = Config::parse(config_str).unwrap();
        let theme_str = r#"
[extra]
hello = "foo"
a_value = 10
[extra.sub]
foo = "default"
truc = "default"
[extra.sub.sub]
foo = "default"
truc = "default"
        "#;
        let theme = Theme::parse(theme_str).unwrap();
        assert!(config.add_theme_extra(&theme).is_ok());
        let extra = config.extra;
        assert_eq!(extra["hello"].as_str().unwrap(), "world".to_string());
        assert_eq!(extra["a_value"].as_integer().unwrap(), 10);
        assert_eq!(extra["sub"]["foo"].as_str().unwrap(), "bar".to_string());
        assert_eq!(extra["sub"].get("truc").expect("The whole extra.sub table was overridden by theme data, discarding extra.sub.truc").as_str().unwrap(), "default".to_string());
        assert_eq!(extra["sub"]["sub"]["foo"].as_str().unwrap(), "bar".to_string());
        assert_eq!(
            extra["sub"]["sub"]
                .get("truc")
                .expect("Failed to merge subsubtable extra.sub.sub")
                .as_str()
                .unwrap(),
            "default".to_string()
        );
    }

    const CONFIG_TRANSLATION: &str = r#"
base_url = "https://remplace-par-ton-url.fr"
default_language = "fr"

[translations]
title = "Un titre"

[languages.en]
[languages.en.translations]
title = "A title"
"#;

    #[test]
    fn can_use_present_translation() {
        let config = Config::parse(CONFIG_TRANSLATION).unwrap();
        assert!(config.languages.contains_key("fr"));
        assert_eq!(config.get_translation("fr", "title").unwrap(), "Un titre");
        assert_eq!(config.get_translation("en", "title").unwrap(), "A title");
    }

    #[test]
    fn error_on_absent_translation_lang() {
        let config = Config::parse(CONFIG_TRANSLATION).unwrap();
        let error = config.get_translation("absent", "key").unwrap_err();

        assert_eq!("Language 'absent' not found.", format!("{}", error));
    }

    #[test]
    fn error_on_absent_translation_key() {
        let config = Config::parse(CONFIG_TRANSLATION).unwrap();
        let error = config.get_translation("en", "absent").unwrap_err();

        assert_eq!("Translation key 'absent' for language 'en' is missing", format!("{}", error));
    }

    #[test]
    fn missing_ignored_content_results_in_empty_vector_and_empty_globset() {
        let config_str = r#"
title = "My site"
base_url = "example.com"
        "#;

        let config = Config::parse(config_str).unwrap();
        let v = config.ignored_content;
        assert_eq!(v.len(), 0);
        assert!(config.ignored_content_globset.is_none());
    }

    #[test]
    fn missing_ignored_static_results_in_empty_vector_and_empty_globset() {
        let config_str = r#"
title = "My site"
base_url = "example.com"
        "#;
        let config = Config::parse(config_str).unwrap();
        let v = config.ignored_static;
        assert_eq!(v.len(), 0);
        assert!(config.ignored_static_globset.is_none());
    }

    #[test]
    fn empty_ignored_content_results_in_empty_vector_and_empty_globset() {
        let config_str = r#"
title = "My site"
base_url = "example.com"
ignored_content = []
        "#;

        let config = Config::parse(config_str).unwrap();
        assert_eq!(config.ignored_content.len(), 0);
        assert!(config.ignored_content_globset.is_none());
    }

    #[test]
    fn empty_ignored_static_results_in_empty_vector_and_empty_globset() {
        let config_str = r#"
title = "My site"
base_url = "example.com"
ignored_static = []
        "#;

        let config = Config::parse(config_str).unwrap();
        assert_eq!(config.ignored_static.len(), 0);
        assert!(config.ignored_static_globset.is_none());
    }

    #[test]
    fn non_empty_ignored_content_results_in_vector_of_patterns_and_configured_globset() {
        let config_str = r#"
title = "My site"
base_url = "example.com"
ignored_content = ["*.{graphml,iso}", "*.py?", "**/{target,temp_folder}"]
        "#;

        let config = Config::parse(config_str).unwrap();
        let v = config.ignored_content;
        assert_eq!(v, vec!["*.{graphml,iso}", "*.py?", "**/{target,temp_folder}"]);

        let g = config.ignored_content_globset.unwrap();
        assert_eq!(g.len(), 3);
        assert!(g.is_match("foo.graphml"));
        assert!(g.is_match("foo/bar/foo.graphml"));
        assert!(g.is_match("foo.iso"));
        assert!(!g.is_match("foo.png"));
        assert!(g.is_match("foo.py2"));
        assert!(g.is_match("foo.py3"));
        assert!(!g.is_match("foo.py"));
        assert!(g.is_match("foo/bar/target"));
        assert!(g.is_match("foo/bar/baz/temp_folder"));
        assert!(g.is_match("foo/bar/baz/temp_folder/target"));
        assert!(g.is_match("temp_folder"));
        assert!(g.is_match("my/isos/foo.iso"));
        assert!(g.is_match("content/poetry/zen.py2"));
    }

    #[test]
    fn non_empty_ignored_static_results_in_vector_of_patterns_and_configured_globset() {
        let config_str = r#"
title = "My site"
base_url = "example.com"
ignored_static = ["*.{graphml,iso}", "*.py?", "**/{target,temp_folder}"]
        "#;

        let config = Config::parse(config_str).unwrap();
        let v = config.ignored_static;
        assert_eq!(v, vec!["*.{graphml,iso}", "*.py?", "**/{target,temp_folder}"]);

        let g = config.ignored_static_globset.unwrap();
        assert_eq!(g.len(), 3);
        assert!(g.is_match("foo.graphml"));
        assert!(g.is_match("foo/bar/foo.graphml"));
        assert!(g.is_match("foo.iso"));
        assert!(!g.is_match("foo.png"));
        assert!(g.is_match("foo.py2"));
        assert!(g.is_match("foo.py3"));
        assert!(!g.is_match("foo.py"));
        assert!(g.is_match("foo/bar/target"));
        assert!(g.is_match("foo/bar/baz/temp_folder"));
        assert!(g.is_match("foo/bar/baz/temp_folder/target"));
        assert!(g.is_match("temp_folder"));
        assert!(g.is_match("my/isos/foo.iso"));
        assert!(g.is_match("content/poetry/zen.py2"));
    }

    #[test]
    fn link_checker_skip_anchor_prefixes() {
        let config_str = r#"
title = "My site"
base_url = "example.com"

[link_checker]
skip_anchor_prefixes = [
    "https://caniuse.com/#feat=",
    "https://github.com/rust-lang/rust/blob/",
]
        "#;

        let config = Config::parse(config_str).unwrap();
        assert_eq!(
            config.link_checker.skip_anchor_prefixes,
            vec!["https://caniuse.com/#feat=", "https://github.com/rust-lang/rust/blob/"]
        );
    }

    #[test]
    fn link_checker_skip_prefixes() {
        let config_str = r#"
title = "My site"
base_url = "example.com"

[link_checker]
skip_prefixes = [
    "http://[2001:db8::]/",
    "https://www.example.com/path",
]
        "#;

        let config = Config::parse(config_str).unwrap();
        assert_eq!(
            config.link_checker.skip_prefixes,
            vec!["http://[2001:db8::]/", "https://www.example.com/path",]
        );
    }

    #[test]
    fn slugify_strategies() {
        let config_str = r#"
title = "My site"
base_url = "example.com"

[slugify]
paths = "on"
taxonomies = "safe"
anchors = "off"
        "#;

        let config = Config::parse(config_str).unwrap();
        assert_eq!(config.slugify.paths, SlugifyStrategy::On);
        assert_eq!(config.slugify.paths_keep_dates, false);
        assert_eq!(config.slugify.taxonomies, SlugifyStrategy::Safe);
        assert_eq!(config.slugify.anchors, SlugifyStrategy::Off);
    }

    #[test]
    fn slugify_paths_keep_dates() {
        let config_str = r#"
title = "My site"
base_url = "example.com"

[slugify]
paths_keep_dates = true
taxonomies = "off"
anchors = "safe"
        "#;

        let config = Config::parse(config_str).unwrap();
        assert_eq!(config.slugify.paths, SlugifyStrategy::On);
        assert_eq!(config.slugify.paths_keep_dates, true);
        assert_eq!(config.slugify.taxonomies, SlugifyStrategy::Off);
        assert_eq!(config.slugify.anchors, SlugifyStrategy::Safe);
    }

    #[test]
    fn cannot_overwrite_theme_mapping_with_invalid_type() {
        let config_str = r#"
base_url = "http://localhost:1312"
default_language = "fr"
[extra]
foo = "bar"
        "#;
        let mut config = Config::parse(config_str).unwrap();
        let theme_str = r#"
[extra]
[extra.foo]
bar = "baz"
        "#;
        let theme = Theme::parse(theme_str).unwrap();
        // We expect an error here
        assert!(config.add_theme_extra(&theme).is_err());
    }

    #[test]
    fn default_output_dir() {
        let config = r#"
title = "My site"
base_url = "https://replace-this-with-your-url.com"
        "#;

        let config = Config::parse(config).unwrap();
        assert_eq!(config.output_dir, "public".to_string());
    }

    #[test]
    fn can_add_output_dir() {
        let config = r#"
title = "My site"
base_url = "https://replace-this-with-your-url.com"
output_dir = "docs"
        "#;

        let config = Config::parse(config).unwrap();
        assert_eq!(config.output_dir, "docs".to_string());
    }

    // TODO: Tests for valid themes; need extra scaffolding (test site) for custom themes.

    #[test]
    fn invalid_highlight_theme() {
        let config = r#"
[markup]
highlight_code = true
highlight_theme = "asdf"
    "#;

        let config = Config::parse(config);
        assert!(config.is_err());
    }

    #[test]
    fn invalid_highlight_theme_css_export() {
        let config = r#"
[markup]
highlight_code = true
highlight_themes_css = [
  { theme = "asdf", filename = "asdf.css" },
]
    "#;

        let config = Config::parse(config);
        assert!(config.is_err());
    }

    // https://github.com/getzola/zola/issues/1687
    #[test]
    fn regression_config_default_lang_data() {
        let config = r#"
base_url = "https://www.getzola.org/"
title = "Zola"
    "#;

        let config = Config::parse(config).unwrap();
        let serialised = config.serialize(&config.default_language);
        assert_eq!(serialised.title, &config.title);
    }

    #[test]
    fn markdown_config_in_serializedconfig() {
        let config = r#"
base_url = "https://www.getzola.org/"
title = "Zola"
[markdown]
highlight_code = true
highlight_theme = "css"
    "#;

        let config = Config::parse(config).unwrap();
        let serialised = config.serialize(&config.default_language);
        assert_eq!(serialised.markdown.highlight_theme, config.markdown.highlight_theme);
    }

    #[test]
    fn sets_default_author_if_present() {
        let config = r#"
title = "My Site"
base_url = "example.com"
author = "person@example.com (Some Person)"
"#;
        let config = Config::parse(config).unwrap();
        assert_eq!(config.author, Some("person@example.com (Some Person)".to_owned()))
    }
}
