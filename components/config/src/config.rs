use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::Utc;
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde_derive::{Deserialize, Serialize};
use syntect::parsing::{SyntaxSet, SyntaxSetBuilder};
use toml;
use toml::Value as Toml;

use crate::highlighting::THEME_SET;
use crate::theme::Theme;
use errors::{bail, Error, Result};
use utils::fs::read_file_with_error;
use utils::slugs::SlugifyStrategy;

// We want a default base url for tests
static DEFAULT_BASE_URL: &str = "http://a-website.com";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Build,
    Serve,
    Check,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Slugify {
    pub paths: SlugifyStrategy,
    pub taxonomies: SlugifyStrategy,
    pub anchors: SlugifyStrategy,
}

impl Default for Slugify {
    fn default() -> Self {
        Slugify {
            paths: SlugifyStrategy::On,
            taxonomies: SlugifyStrategy::On,
            anchors: SlugifyStrategy::On,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Language {
    /// The language code
    pub code: String,
    /// Whether to generate a RSS feed for that language, defaults to `false`
    pub rss: bool,
    /// Whether to generate search index for that language, defaults to `false`
    pub search: bool,
}

impl Default for Language {
    fn default() -> Self {
        Language { code: String::new(), rss: false, search: false }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Taxonomy {
    /// The name used in the URL, usually the plural
    pub name: String,
    /// If this is set, the list of individual taxonomy term page will be paginated
    /// by this much
    pub paginate_by: Option<usize>,
    pub paginate_path: Option<String>,
    /// Whether to generate a RSS feed only for each taxonomy term, defaults to false
    pub rss: bool,
    /// The language for that taxonomy, only used in multilingual sites.
    /// Defaults to the config `default_language` if not set
    pub lang: String,
}

impl Taxonomy {
    pub fn is_paginated(&self) -> bool {
        if let Some(paginate_by) = self.paginate_by {
            paginate_by > 0
        } else {
            false
        }
    }

    pub fn paginate_path(&self) -> &str {
        if let Some(ref path) = self.paginate_path {
            path
        } else {
            "page"
        }
    }
}

impl Default for Taxonomy {
    fn default() -> Self {
        Taxonomy {
            name: String::new(),
            paginate_by: None,
            paginate_path: None,
            rss: false,
            lang: String::new(),
        }
    }
}

type TranslateTerm = HashMap<String, String>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LinkChecker {
    /// Skip link checking for these URL prefixes
    pub skip_prefixes: Vec<String>,
    /// Skip anchor checking for these URL prefixes
    pub skip_anchor_prefixes: Vec<String>,
}

impl Default for LinkChecker {
    fn default() -> LinkChecker {
        LinkChecker { skip_prefixes: Vec::new(), skip_anchor_prefixes: Vec::new() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    pub languages: Vec<Language>,

    /// Languages list and translated strings
    ///
    /// The `String` key of `HashMap` is a language name, the value should be toml crate `Table`
    /// with String key representing term and value another `String` representing its translation.
    ///
    /// The attribute is intentionally not public, use `get_translation()` method for translating
    /// key into different language.
    translations: HashMap<String, TranslateTerm>,

    /// Whether to highlight all code blocks found in markdown files. Defaults to false
    pub highlight_code: bool,
    /// Which themes to use for code highlighting. See Readme for supported themes
    /// Defaults to "base16-ocean-dark"
    pub highlight_theme: String,

    /// Whether to generate RSS. Defaults to false
    pub generate_rss: bool,
    /// The number of articles to include in the RSS feed. Defaults to including all items.
    pub rss_limit: Option<usize>,
    /// If set, files from static/ will be hardlinked instead of copied to the output dir.
    pub hard_link_static: bool,

    pub taxonomies: Vec<Taxonomy>,

    /// Whether to compile the `sass` directory and output the css files into the static folder
    pub compile_sass: bool,
    /// Whether to build the search index for the content
    pub build_search_index: bool,
    /// A list of file glob patterns to ignore when processing the content folder. Defaults to none.
    /// Had to remove the PartialEq derive because GlobSet does not implement it. No impact
    /// because it's unused anyway (who wants to sort Configs?).
    pub ignored_content: Vec<String>,
    #[serde(skip_serializing, skip_deserializing)] // not a typo, 2 are needed
    pub ignored_content_globset: Option<GlobSet>,

    /// The mode Zola is currently being ran on. Some logging/feature can differ depending on the
    /// command being used.
    #[serde(skip_serializing)]
    pub mode: Mode,

    /// A list of directories to search for additional `.sublime-syntax` files in.
    pub extra_syntaxes: Vec<String>,
    /// The compiled extra syntaxes into a syntax set
    #[serde(skip_serializing, skip_deserializing)] // not a typo, 2 are need
    pub extra_syntax_set: Option<SyntaxSet>,

    pub link_checker: LinkChecker,

    /// The setup for which slugification strategies to use for paths, taxonomies and anchors
    pub slugify: Slugify,

    /// All user params set in [extra] in the config
    pub extra: HashMap<String, Toml>,

    /// Set automatically when instantiating the config. Used for cachebusting
    pub build_timestamp: Option<i64>,
}

impl Config {
    /// Parses a string containing TOML to our Config struct
    /// Any extra parameter will end up in the extra field
    pub fn parse(content: &str) -> Result<Config> {
        let mut config: Config = match toml::from_str(content) {
            Ok(c) => c,
            Err(e) => bail!(e),
        };

        if config.base_url.is_empty() || config.base_url == DEFAULT_BASE_URL {
            bail!("A base URL is required in config.toml with key `base_url`");
        }

        if !THEME_SET.themes.contains_key(&config.highlight_theme) {
            bail!("Highlight theme {} not available", config.highlight_theme)
        }

        if config.languages.iter().any(|l| l.code == config.default_language) {
            bail!("Default language `{}` should not appear both in `config.default_language` and `config.languages`", config.default_language)
        }

        config.build_timestamp = Some(Utc::now().timestamp());

        if !config.ignored_content.is_empty() {
            // Convert the file glob strings into a compiled glob set matcher. We want to do this once,
            // at program initialization, rather than for every page, for example. We arrange for the
            // globset matcher to always exist (even though it has to be an inside an Option at the
            // moment because of the TOML serializer); if the glob set is empty the `is_match` function
            // of the globber always returns false.
            let mut glob_set_builder = GlobSetBuilder::new();
            for pat in &config.ignored_content {
                let glob = match Glob::new(pat) {
                    Ok(g) => g,
                    Err(e) => bail!("Invalid ignored_content glob pattern: {}, error = {}", pat, e),
                };
                glob_set_builder.add(glob);
            }
            config.ignored_content_globset =
                Some(glob_set_builder.build().expect("Bad ignored_content in config file."));
        }

        for taxonomy in config.taxonomies.iter_mut() {
            if taxonomy.lang.is_empty() {
                taxonomy.lang = config.default_language.clone();
            }
        }

        Ok(config)
    }

    /// Parses a config file from the given path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Config> {
        let path = path.as_ref();
        let file_name = path.file_name().unwrap();
        let content = read_file_with_error(
            path,
            &format!("No `{:?}` file found. Are you in the right directory?", file_name),
        )?;
        Config::parse(&content)
    }

    /// Attempt to load any extra syntax found in the extra syntaxes of the config
    pub fn load_extra_syntaxes(&mut self, base_path: &Path) -> Result<()> {
        if self.extra_syntaxes.is_empty() {
            return Ok(());
        }

        let mut ss = SyntaxSetBuilder::new();
        for dir in &self.extra_syntaxes {
            ss.add_from_folder(base_path.join(dir), true)?;
        }
        self.extra_syntax_set = Some(ss.build());

        Ok(())
    }

    /// Makes a url, taking into account that the base url might have a trailing slash
    pub fn make_permalink(&self, path: &str) -> String {
        let trailing_bit = if path.ends_with('/') || path.ends_with("rss.xml") || path.is_empty() {
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

    /// Merges the extra data from the theme with the config extra data
    fn add_theme_extra(&mut self, theme: &Theme) -> Result<()> {
        // 3 pass merging
        // 1. save config to preserve user
        let original = self.extra.clone();
        // 2. inject theme extra values
        for (key, val) in &theme.extra {
            self.extra.entry(key.to_string()).or_insert_with(|| val.clone());
        }

        // 3. overwrite with original config
        for (key, val) in &original {
            self.extra.entry(key.to_string()).or_insert_with(|| val.clone());
        }

        Ok(())
    }

    /// Parse the theme.toml file and merges the extra data from the theme
    /// with the config extra data
    pub fn merge_with_theme(&mut self, path: &PathBuf) -> Result<()> {
        let theme = Theme::from_file(path)?;
        self.add_theme_extra(&theme)
    }

    /// Is this site using i18n?
    pub fn is_multilingual(&self) -> bool {
        !self.languages.is_empty()
    }

    /// Returns the codes of all additional languages
    pub fn languages_codes(&self) -> Vec<&str> {
        self.languages.iter().map(|l| l.code.as_ref()).collect()
    }

    pub fn is_in_build_mode(&self) -> bool {
        self.mode == Mode::Build
    }

    pub fn is_in_serve_mode(&self) -> bool {
        self.mode == Mode::Serve
    }

    pub fn is_in_check_mode(&self) -> bool {
        self.mode == Mode::Check
    }

    pub fn enable_serve_mode(&mut self) {
        self.mode = Mode::Serve;
    }

    pub fn enable_check_mode(&mut self) {
        self.mode = Mode::Check;
        // Disable syntax highlighting since the results won't be used
        // and this operation can be expensive.
        self.highlight_code = false;
    }

    pub fn get_translation<S: AsRef<str>>(&self, lang: S, key: S) -> Result<String> {
        let terms = self.translations.get(lang.as_ref()).ok_or_else(|| {
            Error::msg(format!("Translation for language '{}' is missing", lang.as_ref()))
        })?;

        terms
            .get(key.as_ref())
            .ok_or_else(|| {
                Error::msg(format!(
                    "Translation key '{}' for language '{}' is missing",
                    key.as_ref(),
                    lang.as_ref()
                ))
            })
            .map(|term| term.to_string())
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            base_url: DEFAULT_BASE_URL.to_string(),
            title: None,
            description: None,
            theme: None,
            highlight_code: false,
            highlight_theme: "base16-ocean-dark".to_string(),
            default_language: "en".to_string(),
            languages: Vec::new(),
            generate_rss: false,
            rss_limit: None,
            hard_link_static: false,
            taxonomies: Vec::new(),
            compile_sass: false,
            mode: Mode::Build,
            build_search_index: false,
            ignored_content: Vec::new(),
            ignored_content_globset: None,
            translations: HashMap::new(),
            extra_syntaxes: Vec::new(),
            extra_syntax_set: None,
            link_checker: LinkChecker::default(),
            slugify: Slugify::default(),
            extra: HashMap::new(),
            build_timestamp: Some(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, SlugifyStrategy, Theme};

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
        let mut config = Config::default();
        config.base_url = "http://vincent.is".to_string();
        assert_eq!(config.make_permalink(""), "http://vincent.is/");
    }

    #[test]
    fn can_make_url_index_page_with_railing_slash_url() {
        let mut config = Config::default();
        config.base_url = "http://vincent.is/".to_string();
        assert_eq!(config.make_permalink(""), "http://vincent.is/");
    }

    #[test]
    fn can_make_url_with_non_trailing_slash_base_url() {
        let mut config = Config::default();
        config.base_url = "http://vincent.is".to_string();
        assert_eq!(config.make_permalink("hello"), "http://vincent.is/hello/");
    }

    #[test]
    fn can_make_url_with_trailing_slash_path() {
        let mut config = Config::default();
        config.base_url = "http://vincent.is/".to_string();
        assert_eq!(config.make_permalink("/hello"), "http://vincent.is/hello/");
    }

    #[test]
    fn can_make_url_with_localhost() {
        let mut config = Config::default();
        config.base_url = "http://127.0.0.1:1111".to_string();
        assert_eq!(config.make_permalink("/tags/rust"), "http://127.0.0.1:1111/tags/rust/");
    }

    // https://github.com/Keats/gutenberg/issues/486
    #[test]
    fn doesnt_add_trailing_slash_to_rss() {
        let mut config = Config::default();
        config.base_url = "http://vincent.is/".to_string();
        assert_eq!(config.make_permalink("rss.xml"), "http://vincent.is/rss.xml");
    }

    #[test]
    fn can_merge_with_theme_data_and_preserve_config_value() {
        let config_str = r#"
title = "My site"
base_url = "https://replace-this-with-your-url.com"

[extra]
hello = "world"
        "#;
        let mut config = Config::parse(config_str).unwrap();
        let theme_str = r#"
[extra]
hello = "foo"
a_value = 10
        "#;
        let theme = Theme::parse(theme_str).unwrap();
        assert!(config.add_theme_extra(&theme).is_ok());
        let extra = config.extra;
        assert_eq!(extra["hello"].as_str().unwrap(), "world".to_string());
        assert_eq!(extra["a_value"].as_integer().unwrap(), 10);
    }

    const CONFIG_TRANSLATION: &str = r#"
base_url = "https://remplace-par-ton-url.fr"
default_language = "fr"

[translations]
[translations.fr]
title = "Un titre"

[translations.en]
title = "A title"
        "#;

    #[test]
    fn can_use_present_translation() {
        let config = Config::parse(CONFIG_TRANSLATION).unwrap();
        assert_eq!(config.get_translation("fr", "title").unwrap(), "Un titre");
        assert_eq!(config.get_translation("en", "title").unwrap(), "A title");
    }

    #[test]
    fn error_on_absent_translation_lang() {
        let config = Config::parse(CONFIG_TRANSLATION).unwrap();
        let error = config.get_translation("absent", "key").unwrap_err();

        assert_eq!("Translation for language 'absent' is missing", format!("{}", error));
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
    fn non_empty_ignored_content_results_in_vector_of_patterns_and_configured_globset() {
        let config_str = r#"
title = "My site"
base_url = "example.com"
ignored_content = ["*.{graphml,iso}", "*.py?"]
        "#;

        let config = Config::parse(config_str).unwrap();
        let v = config.ignored_content;
        assert_eq!(v, vec!["*.{graphml,iso}", "*.py?"]);

        let g = config.ignored_content_globset.unwrap();
        assert_eq!(g.len(), 2);
        assert!(g.is_match("foo.graphml"));
        assert!(g.is_match("foo.iso"));
        assert!(!g.is_match("foo.png"));
        assert!(g.is_match("foo.py2"));
        assert!(g.is_match("foo.py3"));
        assert!(!g.is_match("foo.py"));
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
        assert_eq!(config.slugify.taxonomies, SlugifyStrategy::Safe);
        assert_eq!(config.slugify.anchors, SlugifyStrategy::Off);
    }

    #[test]
    fn error_on_language_set_twice() {
        let config_str = r#"
base_url = "https://remplace-par-ton-url.fr"
default_language = "fr"
languages = [
    { code = "fr" },
    { code = "en" },
]
        "#;
        let config = Config::parse(config_str);
        let err = config.unwrap_err();
        assert_eq!("Default language `fr` should not appear both in `config.default_language` and `config.languages`", format!("{}", err));
    }
}
