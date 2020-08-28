pub mod languages;
pub mod link_checker;
pub mod search;
pub mod slugify;
pub mod taxonomies;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use globset::{Glob, GlobSet, GlobSetBuilder};
use serde_derive::{Deserialize, Serialize};
use syntect::parsing::{SyntaxSet, SyntaxSetBuilder};
use toml::Value as Toml;
use unic_langid::{langid, LanguageIdentifier};

use crate::highlighting::THEME_SET;
use crate::theme::Theme;
use errors::{bail, Error, Result};
use utils::fs::read_file_with_error;

// We want a default base url for tests
static DEFAULT_BASE_URL: &str = "http://a-website.com";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Build,
    Serve,
    Check,
}

/// Configuration with language-specific options set for a given language and translations merged
/// with the default values.
///
/// Instead of [Config], this should be exposed to rendering contexts.
///
/// It's a newtype struct wrapping [Config], where `0.default_language_options` contains the
/// settings for `0.lang`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LocalizedConfig(pub Config);

/// Contains the deserialized `config.toml`.
///
/// It will be processed into a [LocalizedConfig] to create a transparent and backwards-compatible
/// localization for templates.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Base URL of the site, the only required config argument
    pub base_url: String,

    /// Theme to use
    pub theme: Option<String>,

    /// The language that this config applies to
    ///
    /// This is done to make [LocalizedConfig] self-contained. For [Config], this is always equal
    /// to `default_language`.
    #[serde(skip_deserializing)]
    pub lang: LanguageIdentifier,

    /// The primary/default language of the site
    ///
    /// MUST be a valid language, specified using the [BCP 47] syntax. To retain backwards
    /// compatibility with other language naming systems, the [language_alias] option was added.
    pub default_language: LanguageIdentifier,
    /// Site-wide defaults for langauge-specific settings
    ///
    /// This is flattened when serializing/deserializing, i.e. its fields will appear as if they
    /// were at the top level alongside base_url. These values are used for the default language,
    /// and other languages will fall back to these values if they are not overridden in
    /// `languages`
    #[serde(flatten, default = "languages::LocaleOptions::default_for_default_language")]
    pub default_language_options: languages::LocaleOptions,

    /// Language-specific options for translations of this site
    ///
    /// The key of the TOML table (represented as a HashMap in Rust) is a valid language code, and
    /// its values can be used for overriding some of the site-wide settings.
    ///
    /// See [LocaleOptions] for what options can be specified here and what their default values
    /// are.
    ///
    /// Having an entry for the default language is treated as an error.
    pub languages: HashMap<LanguageIdentifier, languages::LocaleOptions>,

    /// Whether to highlight all code blocks found in markdown files. Defaults to false
    pub highlight_code: bool,
    /// Which themes to use for code highlighting. See Readme for supported themes
    /// Defaults to "base16-ocean-dark"
    pub highlight_theme: String,

    /// The filename to use for feeds. Used to find the template, too.
    /// Defaults to "atom.xml", with "rss.xml" also having a template provided out of the box.
    pub feed_filename: String,
    /// If set, files from static/ will be hardlinked instead of copied to the output dir.
    pub hard_link_static: bool,

    /// Whether to compile the `sass` directory and output the css files into the static folder
    pub compile_sass: bool,
    /// Whether to minify the html output
    pub minify_html: bool,
    /// Whether to build the search index for the content

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

    pub link_checker: link_checker::LinkChecker,

    /// The setup for which slugification strategies to use for paths, taxonomies and anchors
    pub slugify: slugify::Slugify,
}

impl Config {
    /// Parses a string containing TOML to our Config struct and does sanity checking
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

        config.lang = config.default_language.clone();

        // Find duplicate language aliases/names by first extracting them to a vector while also
        // setting aliases.
        let mut language_names: Vec<String> = Vec::new();

        if config.default_language_options.language_alias.is_empty() {
            config.default_language_options.language_alias = config.default_language.to_string();
        }
        language_names.push(config.default_language_options.language_alias.clone());
        for (identifier, options) in config.languages.iter_mut() {
            if options.language_alias == "" {
                options.language_alias = identifier.to_string();
            }
            language_names.push(options.language_alias.clone());
        }
        // https://stackoverflow.com/a/46766782
        // TODO: actually print which identifier/alias is causing a problem
        if (1..language_names.len()).any(|i| language_names[i..].contains(&language_names[i - 1])) {
            bail!("A language code or alias should not appear twice in config.toml");
        }

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

        for taxonomy in config.default_language_options.taxonomies.iter_mut() {
            taxonomy.lang = config.default_language.clone();
            taxonomy.language_alias = config.default_language_options.language_alias.clone();
        }
        for (identifier, options) in config.languages.iter_mut() {
            for taxonomy in options.taxonomies.iter_mut() {
                taxonomy.lang = identifier.clone();
                taxonomy.language_alias = options.language_alias.clone();
            }
        }

        // FIXME: until serde lets us set a different default for the flattened
        // `self.default_language_options`, we set fields with None value to their default vaues
        // here.
        let fallback = languages::LocaleOptions::default_for_default_lang();
        config.default_language_options.generate_feed =
            config.default_language_options.generate_feed.or(fallback.generate_feed);
        config.default_language_options.feed_limit =
            config.default_language_options.feed_limit.or(fallback.feed_limit);
        config.default_language_options.build_search_index =
            config.default_language_options.build_search_index.or(fallback.build_search_index);
        config.default_language_options.search =
            config.default_language_options.search.or(fallback.search);

        // TODO: re-enable once it's a bit more tested
        config.minify_html = false;

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

    /// Merges the extra data from the theme with the config extra data
    ///
    /// TODO: language negotiation (variants and stuff)
    fn add_theme_extra(&mut self, theme: &Theme) -> Result<()> {
        if let Some(t_identifier) = theme.default_language.clone() {
            // merge for theme.default_language
            if t_identifier == self.default_language {
                merge(
                    &mut self.default_language_options.extra,
                    &theme.default_language_options.extra,
                )?;
            } else if let Some(options) = self.languages.get_mut(&t_identifier) {
                merge(&mut options.extra, &theme.default_language_options.extra)?;
            }
        } else {
            merge(&mut self.default_language_options.extra, &theme.default_language_options.extra)?;
        }

        // merge other languages in theme.toml
        for (t_identifier, t_options) in theme.languages.iter() {
            if t_identifier == &self.default_language {
                merge(&mut self.default_language_options.extra, &t_options.extra)?;
            } else if let Some(c_options) = self.languages.get_mut(t_identifier) {
                merge(&mut c_options.extra, &t_options.extra)?;
            }
        }
        Ok(())
    }

    /// Parse the theme.toml file and merges the extra data from the theme
    /// with the config extra data
    pub fn merge_with_theme(&mut self, path: &PathBuf) -> Result<()> {
        let theme = Theme::from_file(path)?;
        self.add_theme_extra(&theme)
    }

    /// Add fallback values for missing fields in language options
    ///
    /// Should be called after merging with theme.
    ///
    /// How each field is handled during merging is described in [LocaleOptions]'s documentation.
    pub fn merge_languages_with_default(&mut self) -> Result<()> {
        let def_o = &self.default_language_options;
        for (_, opt) in self.languages.iter_mut() {
            opt.generate_feed = opt.generate_feed.or(def_o.generate_feed);
            opt.feed_limit = opt.feed_limit.or(def_o.feed_limit);
            opt.build_search_index = opt.build_search_index.or(def_o.build_search_index);
            opt.search = opt.search.clone().or_else(|| def_o.search.clone());
            merge(&mut opt.extra, &def_o.extra)?;
        }
        Ok(())
    }

    /// Is this site using i18n?
    pub fn is_multilingual(&self) -> bool {
        !self.languages.is_empty()
    }

    /// Return the language identifier of an additional language by its alias
    pub fn get_language_identifier<S: AsRef<str>>(&self, name: S) -> Option<LanguageIdentifier> {
        self.languages.iter().find_map(|(i, o)| {
            if o.language_alias == name.as_ref() {
                Some(i.clone())
            } else {
                None
            }
        })
    }

    /// Return the alias of a language by its identifier
    pub fn get_language_alias<L: AsRef<LanguageIdentifier>>(&self, lang: L) -> Option<&str> {
        if lang.as_ref() == &self.lang {
            Some(self.default_language_options.language_alias.as_str())
        } else if let Some(o) = self.languages.get(lang.as_ref()) {
            Some(o.language_alias.as_str())
        } else {
            None
        }
    }

    /// Returns the codes of all additional languages
    pub fn languages_codes(&self) -> Vec<&LanguageIdentifier> {
        self.languages.keys().collect()
    }

    /// Returns the aliases of all additional languages
    pub fn language_aliases(&self) -> Vec<&str> {
        self.languages.iter().map(|(_, o)| &*o.language_alias).collect()
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

    pub fn get_localized(&self, lang: &LanguageIdentifier) -> Result<LocalizedConfig> {
        if lang == &self.default_language {
            Ok(LocalizedConfig(self.clone()))
        } else if self.languages.contains_key(lang) {
            let mut languages = self.languages.clone();
            let (_, default_language_options) = languages.remove_entry(&lang).unwrap();
            languages.insert(self.default_language.clone(), self.default_language_options.clone());
            Ok(LocalizedConfig(Config {
                lang: lang.clone(),
                default_language_options,
                languages,
                ..self.clone()
            }))
        } else {
            bail!("`{}` was not found in `config.languages`", lang);
        }
    }
}

/// Merge HashMaps of Toml values
pub fn merge(into: &mut HashMap<String, Toml>, from: &HashMap<String, Toml>) -> Result<()> {
    for (key, val) in from {
        if let Some(into_val) = into.get_mut(key) {
            merge_table(into_val, val)?;
        } else {
            into.insert(key.to_string(), val.clone());
        }
    }
    Ok(())
}

/// Merge TOML data that can be a table, or anything else
pub fn merge_table(into: &mut Toml, from: &Toml) -> Result<()> {
    match (from.is_table(), into.is_table()) {
        (false, false) => {
            // These are not tables so we have nothing to merge
            Ok(())
        }
        (true, true) => {
            // Recursively merge these tables
            let into_table = into.as_table_mut().unwrap();
            for (key, from_val) in from.as_table().unwrap() {
                if let Some(into_val) = into_table.get_mut(key) {
                    merge_table(into_val, from_val)?;
                } else {
                    into_table.insert(key.to_string(), from_val.clone());
                }
            }
            Ok(())
        }
        _ => {
            // Trying to merge a table with something else
            Err(Error::msg(&format!("Cannot merge config.toml with theme.toml because the following values have incompatibles types:\n- {}\n - {}", into, from)))
        }
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            base_url: DEFAULT_BASE_URL.to_string(),
            theme: None,
            highlight_code: false,
            highlight_theme: "base16-ocean-dark".to_string(),
            lang: langid!("en"),
            default_language: langid!("en"),
            default_language_options: languages::LocaleOptions {
                language_alias: "en".to_string(),
                ..languages::LocaleOptions::default_for_default_lang()
            },
            languages: HashMap::new(),
            feed_filename: "atom.xml".to_string(),
            hard_link_static: false,
            compile_sass: false,
            minify_html: false,
            mode: Mode::Build,
            ignored_content: Vec::new(),
            ignored_content_globset: None,
            extra_syntaxes: Vec::new(),
            extra_syntax_set: None,
            link_checker: link_checker::LinkChecker::default(),
            slugify: slugify::Slugify::default(),
        }
    }
}

impl LocalizedConfig {
    /// Get an `[extra]` field for a specific language
    ///
    /// Only for backwards compatibility, since the config.extra values automatically get
    /// translated when LocalizedConfig is created.
    // #[deprecated]
    pub fn get_translation<S: AsRef<str>>(
        &self,
        lang: LanguageIdentifier,
        key: S,
    ) -> Result<&Toml> {
        let terms = if self.0.lang == lang {
            &self.0.default_language_options.extra
        } else {
            &self
                .0
                .languages
                .get(lang.as_ref())
                .ok_or_else(|| {
                    Error::msg(format!("Translation for language '{}' is missing", lang.as_ref()))
                })?
                .extra
        };

        terms.get(key.as_ref()).ok_or_else(|| {
            Error::msg(format!(
                "Translation key '{}' for language '{}' is missing",
                key.as_ref(),
                lang.as_ref()
            ))
        })
    }
}

impl Default for LocalizedConfig {
    fn default() -> Self {
        LocalizedConfig(Config::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::slugs::SlugifyStrategy;

    #[test]
    fn can_import_valid_config() {
        let config = r#"
title = "My site"
base_url = "https://replace-this-with-your-url.com"
        "#;

        let config = Config::parse(config).unwrap();
        assert_eq!(config.default_language_options.title.unwrap(), "My site".to_string());
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
        assert_eq!(
            config.unwrap().default_language_options.extra.get("hello").unwrap().as_str().unwrap(),
            "world"
        );
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
    fn doesnt_add_trailing_slash_to_feed() {
        let mut config = Config::default();
        config.base_url = "http://vincent.is/".to_string();
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
        let extra = config.default_language_options.extra;
        assert_eq!(extra["hello"].as_str().unwrap(), "world".to_string());
        assert_eq!(extra["a_value"].as_integer().unwrap(), 10);
        assert_eq!(extra["sub"]["foo"].as_str().unwrap(), "bar".to_string());
        assert_eq!(extra["sub"].get("truc").expect("The whole extra.sub table was overriden by theme data, discarding extra.sub.truc").as_str().unwrap(), "default".to_string());
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

    const CONFIG_MULTILINGUAL: &str = r#"
base_url = "https://remplace-par-ton-url.fr"
default_language = "fr"

[extra]
title = "Un titre"
foo = "bar"

[languages.en.extra]
title = "A title"
        "#;

    #[test]
    fn can_use_present_translation_legacy() {
        let config =
            Config::parse(CONFIG_MULTILINGUAL).unwrap().get_localized(&langid!("fr")).unwrap();
        assert_eq!(
            config.get_translation(langid!("fr"), "title").unwrap().as_str().unwrap(),
            "Un titre"
        );
        assert_eq!(
            config.get_translation(langid!("en"), "title").unwrap().as_str().unwrap(),
            "A title"
        );
    }

    #[test]
    fn error_on_absent_translation_lang_legacy() {
        let config =
            Config::parse(CONFIG_MULTILINGUAL).unwrap().get_localized(&langid!("fr")).unwrap();
        let error = config.get_translation(langid!("absent"), "key").unwrap_err();

        assert_eq!("Translation for language 'absent' is missing", format!("{}", error));
    }

    #[test]
    fn error_on_absent_translation_key_legacy() {
        let config =
            Config::parse(CONFIG_MULTILINGUAL).unwrap().get_localized(&langid!("fr")).unwrap();
        let error = config.get_translation(langid!("en"), "qux").unwrap_err();

        assert_eq!("Translation key 'qux' for language 'en' is missing", format!("{}", error));
    }

    #[test]
    fn can_fall_back_to_default_language() {
        let mut config = Config::parse(CONFIG_MULTILINGUAL).unwrap();
        let res = config.merge_languages_with_default();
        assert!(res.is_ok());
        let config = config.get_localized(&langid!("en")).unwrap();
        assert_eq!(config.get_translation(langid!("en"), "foo").unwrap().as_str().unwrap(), "bar");
    }

    const THEME_MULTILINGUAL: &str = r#"
default_language = "en"

[extra]
foo = "baz"
bar = "qux"

[languages.fr.extra]
baz = "qux"

[languages.de.extra]
title = "Ein Titel"
        "#;

    #[test]
    fn can_merge_multilingual_config_and_multilingual_theme() {
        let mut config = Config::parse(CONFIG_MULTILINGUAL).unwrap();
        let theme = Theme::parse(THEME_MULTILINGUAL).unwrap();
        let res = config.add_theme_extra(&theme);
        let en = config.languages.get(&langid!("en")).unwrap();

        assert!(
            res.is_ok(),
            "Should merge a valid config.toml and theme.toml given there are no type collisions"
        );
        assert_eq!(
            "bar",
            config.default_language_options.extra.get("foo").unwrap().as_str().unwrap()
        );
        assert!(config.default_language_options.extra.get("bar").is_none());
        assert_eq!(
            "qux",
            config.default_language_options.extra.get("baz").unwrap().as_str().unwrap()
        );
        assert_eq!("baz", en.extra.get("foo").unwrap().as_str().unwrap());
        assert_eq!("qux", en.extra.get("bar").unwrap().as_str().unwrap());
        assert!(config.languages.get(&langid!("en")).unwrap().extra.get("baz").is_none());
        assert!(
            config.languages.get(&langid!("de")).is_none(),
            "A theme should not add new languages to config.languages"
        );

        // Check if language fallback is done properly
        let res = config.merge_languages_with_default();
        assert!(res.is_ok(), "Can merge languages with default");
        let lc = config.get_localized(&langid!("en")).unwrap();
        assert_eq!(langid!("en"), lc.0.lang);
        assert_eq!(
            "baz",
            lc.0.default_language_options.extra.get("foo").unwrap().as_str().unwrap()
        );
        assert_eq!(
            "qux",
            lc.0.default_language_options.extra.get("bar").unwrap().as_str().unwrap()
        );
        assert_eq!(
            "qux",
            lc.0.default_language_options.extra.get("baz").unwrap().as_str().unwrap()
        );

        assert!(lc.0.languages.contains_key(&langid!("fr")));
    }

    #[test]
    fn can_merge_multilingual_config_and_unlocalized_theme() {
        let theme_str = r#"
[extra]
foo = "baz"
bar = "qux"
    "#;
        let mut config = Config::parse(CONFIG_MULTILINGUAL).unwrap();
        let theme = Theme::parse(theme_str).unwrap();
        let res = config.add_theme_extra(&theme);

        // An unlocalized theme's (without a default_language field) language should not be treated
        // as `en`, but rather as `default_language`.
        assert!(
            res.is_ok(),
            "Should merge a valid config.toml and theme.toml given there are no type collisions"
        );
        assert_eq!(
            "bar",
            config.default_language_options.extra.get("foo").unwrap().as_str().unwrap()
        );
        assert_eq!(
            "qux",
            config.default_language_options.extra.get("bar").unwrap().as_str().unwrap(),
            "Monolingual theme should merge with default_language_option"
        );
        assert!(
            config.languages.get(&langid!("en")).unwrap().extra.get("foo").is_none(),
            "Monolingual theme should only merge with default_language_options"
        );
    }

    #[test]
    fn can_merge_unlocalized_config_and_multilingual_theme() {
        let config_str = r#"
title = "My site"
base_url = "https://replace-this-with-your-url.com"

[extra]
foo = "bar"
    "#;
        let mut config = Config::parse(config_str).unwrap();
        let theme = Theme::parse(THEME_MULTILINGUAL).unwrap();
        let res = config.add_theme_extra(&theme);

        assert!(res.is_ok(), "Should merge an unlocalized config with a multilingual theme");
        assert_eq!(
            "bar",
            config.default_language_options.extra.get("foo").unwrap().as_str().unwrap()
        );
        assert_eq!(
            "qux",
            config.default_language_options.extra.get("bar").unwrap().as_str().unwrap()
        );
        assert!(
            config.default_language_options.extra.get("baz").is_none(),
            "Unlocalized `config.extra` should only be merged with `theme.extra`"
        );
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
    fn error_on_language_code_set_twice() {
        let config_str = r#"
base_url = "https://remplace-par-ton-url.fr"
default_language = "fr"

languages = { fr = {}, en = {} }
        "#;
        let config = Config::parse(config_str);
        let err = config.unwrap_err();
        assert_eq!(
            "A language code or alias should not appear twice in config.toml",
            format!("{}", err)
        );
    }

    #[test]
    fn error_on_language_alias_set_twice() {
        let config_str = r#"
base_url = "https://remplace-par-ton-url.fr"
default_language = "fr-FR"
language_alias = "fr"

languages = { fr-CA = { language_alias = "fr" } }
        "#;
        let config = Config::parse(config_str);
        let err = config.unwrap_err();
        assert_eq!(
            "A language code or alias should not appear twice in config.toml",
            format!("{}", err)
        );
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
        assert_eq!(false, config.add_theme_extra(&theme).is_ok());
    }
}
