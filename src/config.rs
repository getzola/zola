use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::collections::HashMap;

use toml::{Value as Toml, self};

use errors::{Result, ResultExt};
use rendering::highlighting::THEME_SET;


#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// Title of the site
    pub title: String,
    /// Base URL of the site
    pub base_url: String,

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
    /// Whether to insert a link for each header like in Github READMEs. Defaults to false
    /// The default template can be overridden by creating a `anchor-link.html` template and CSS will need to be
    /// written if you turn that on.
    pub insert_anchor_links: Option<bool>,

    /// All user params set in [extra] in the config
    pub extra: Option<HashMap<String, Toml>>,
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
        set_default!(config.rss_limit, <usize>::max_value());
        set_default!(config.generate_tags_pages, false);
        set_default!(config.generate_categories_pages, false);
        set_default!(config.insert_anchor_links, false);

        match config.highlight_theme {
            Some(ref t) => {
                if !THEME_SET.themes.contains_key(t) {
                    bail!("Theme {} not available", t)
                }
            },
            None => config.highlight_theme = Some("base16-ocean-dark".to_string())
        };

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
        let trailing_bit = if path.ends_with('/') { "" } else { "/" };

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
}

/// Exists only for testing purposes
#[doc(hidden)]
impl Default for Config {
    fn default() -> Config {
        Config {
            title: "".to_string(),
            base_url: "http://a-website.com/".to_string(),
            highlight_code: Some(true),
            highlight_theme: Some("base16-ocean-dark".to_string()),
            description: None,
            language_code: Some("en".to_string()),
            generate_rss: Some(false),
            rss_limit: Some(10000),
            generate_tags_pages: Some(true),
            generate_categories_pages: Some(true),
            insert_anchor_links: Some(false),
            extra: None,
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
    use super::{Config};

    #[test]
    fn can_import_valid_config() {
        let config = r#"
title = "My site"
base_url = "https://replace-this-with-your-url.com"
        "#;

        let config = Config::parse(config).unwrap();
        assert_eq!(config.title, "My site".to_string());
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
}
