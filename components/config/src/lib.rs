#[macro_use]
extern crate serde_derive;
extern crate toml;
#[macro_use]
extern crate errors;
extern crate highlighting;
extern crate chrono;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use toml::{Value as Toml};
use chrono::Utc;

use errors::{Result, ResultExt};
use highlighting::THEME_SET;


mod theme;

use theme::Theme;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// Base URL of the site, the only required config argument
    pub base_url: String,

    /// Theme to use
    pub theme: Option<String>,
    /// Title of the site. Defaults to None
    pub title: Option<String>,
    /// Whether to highlight all code blocks found in markdown files. Defaults to false
    pub highlight_code: Option<bool>,
    /// Which themes to use for code highlighting. See Readme for supported themes
    pub highlight_theme: Option<String>,
    /// Description of the site
    pub description: Option<String>,
    /// The language used in the site. Defaults to "en"
    pub language_code: Option<String>,
    /// Whether to generate RSS. Defaults to false
    pub generate_rss: Option<bool>,
    /// The number of articles to include in the RSS feed. Defaults to unlimited
    pub rss_limit: Option<usize>,
    /// Whether to generate tags and individual tag pages if some pages have them. Defaults to true
    pub generate_tags_pages: Option<bool>,
    /// Whether to generate categories and individual tag categories if some pages have them. Defaults to true
    pub generate_categories_pages: Option<bool>,
    /// Whether to compile the `sass` directory and output the css files into the static folder
    pub compile_sass: Option<bool>,

    /// All user params set in [extra] in the config
    pub extra: Option<HashMap<String, Toml>>,

    /// Set automatically when instantiating the config. Used for cachebusting
    pub build_timestamp: Option<i64>,
}

macro_rules! set_default {
    ($key: expr, $default: expr) => {
        if $key.is_none() {
            $key = Some($default);
        }
    }
}

impl Config {
    /// Parses a string containing TOML to our Config struct
    /// Any extra parameter will end up in the extra field
    pub fn parse(content: &str) -> Result<Config> {
        let mut config: Config = match toml::from_str(content) {
            Ok(c) => c,
            Err(e) => bail!(e)
        };

        set_default!(config.language_code, "en".to_string());
        set_default!(config.highlight_code, false);
        set_default!(config.generate_rss, false);
        set_default!(config.rss_limit, 20);
        set_default!(config.generate_tags_pages, false);
        set_default!(config.generate_categories_pages, false);
        set_default!(config.compile_sass, false);
        set_default!(config.extra, HashMap::new());

        match config.highlight_theme {
            Some(ref t) => {
                if !THEME_SET.themes.contains_key(t) {
                    bail!("Theme {} not available", t)
                }
            }
            None => config.highlight_theme = Some("base16-ocean-dark".to_string())
        };

        config.build_timestamp = Some(Utc::now().timestamp());
        Ok(config)
    }

    /// Parses a config file from the given path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Config> {
        let mut content = String::new();
        File::open(path)
            .chain_err(|| "No `config.toml` file found. Are you in the right directory?")?
            .read_to_string(&mut content)?;

        Config::parse(&content)
    }

    /// Makes a url, taking into account that the base url might have a trailing slash
    pub fn make_permalink(&self, path: &str) -> String {
        let trailing_bit = if path.ends_with('/') || path.is_empty() { "" } else { "/" };

        // Index section with a base url that has a trailing slash
        if self.base_url.ends_with('/') && path == "/" {
            self.base_url.clone()
        } else if path == "/" {
            // index section with a base url that doesn't have a trailing slash
            format!("{}/", self.base_url)
        } else if self.base_url.ends_with('/') && path.starts_with('/') {
            format!("{}{}{}", self.base_url, &path[1..], trailing_bit)
        } else if self.base_url.ends_with('/') {
            format!("{}{}{}", self.base_url, path, trailing_bit)
        } else {
            format!("{}/{}{}", self.base_url, path, trailing_bit)
        }
    }

    /// Merges the extra data from the theme with the config extra data
    fn add_theme_extra(&mut self, theme: &Theme) -> Result<()> {
        if let Some(ref mut config_extra) = self.extra {
            // 3 pass merging
            // 1. save config to preserve user
            let original = config_extra.clone();
            // 2. inject theme extra values
            for (key, val) in &theme.extra {
                config_extra.entry(key.to_string()).or_insert_with(|| val.clone());
            }

            // 3. overwrite with original config
            for (key, val) in &original {
                config_extra.entry(key.to_string()).or_insert_with(|| val.clone());
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
}

/// Exists only for testing purposes
#[doc(hidden)]
impl Default for Config {
    fn default() -> Config {
        Config {
            title: Some("".to_string()),
            theme: None,
            base_url: "http://a-website.com/".to_string(),
            highlight_code: Some(true),
            highlight_theme: Some("base16-ocean-dark".to_string()),
            description: None,
            language_code: Some("en".to_string()),
            generate_rss: Some(false),
            rss_limit: Some(10_000),
            generate_tags_pages: Some(true),
            generate_categories_pages: Some(true),
            compile_sass: Some(false),
            extra: None,
            build_timestamp: Some(1),
        }
    }
}


/// Get and parse the config.
/// If it doesn't succeed, exit
pub fn get_config(path: &Path, filename: &str) -> Config {
    match Config::from_file(path.join(filename)) {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to load {}", filename);
            println!("Error: {}", e);
            ::std::process::exit(1);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::{Config, Theme};

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
        assert_eq!(config.unwrap().extra.unwrap().get("hello").unwrap().as_str().unwrap(), "world");
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
        let extra = config.extra.unwrap();
        assert_eq!(extra["hello"].as_str().unwrap(), "world".to_string());
        assert_eq!(extra["a_value"].as_integer().unwrap(), 10);
    }
}
