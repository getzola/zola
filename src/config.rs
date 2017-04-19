use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::collections::HashMap;

use toml::{Value as Toml, self};

use errors::{Result, ResultExt};
use markdown::SETUP;


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
    /// Whether to generate RSS, defaults to false
    pub generate_rss: Option<bool>,
    /// Whether to generate tags and individual tag pages if some pages have them. Defaults to true
    pub generate_tags_pages: Option<bool>,
    /// Whether to generate categories and individual tag categories if some pages have them. Defaults to true
    pub generate_categories_pages: Option<bool>,
    /// Whether to insert a link for each header like in Github READMEs. Defaults to false
    /// The default template can be overridden by creating a `anchor-link.html` template and CSS will need to be
    /// written if you turn that on.
    pub insert_anchor_links: Option<bool>,

    // Pagination
    /// How many pages to be displayed per paginated page. Defaults to `5`.
    pub paginate_by: Option<usize>,
    /// Path to be used by pagination: the page number will be appended after it. Defaults to `page`.
    pub paginate_path: Option<String>,

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

        match config.highlight_theme {
            Some(ref t) => {
                if !SETUP.theme_set.themes.contains_key(t) {
                    bail!("Theme {} not available", t)
                }
            },
            None => config.highlight_theme = Some("base16-ocean-dark".to_string())
        };

        set_default!(config.generate_rss, false);
        set_default!(config.generate_tags_pages, true);
        set_default!(config.generate_categories_pages, true);
        set_default!(config.insert_anchor_links, false);
        set_default!(config.paginate_by, 5);
        set_default!(config.paginate_path, "page".to_string());

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
        if self.base_url.ends_with('/') {
            format!("{}{}", self.base_url, path)
        } else {
            format!("{}/{}", self.base_url, path)
        }
    }
}

impl Default for Config {
    /// Exists for testing purposes
    fn default() -> Config {
        Config {
            title: "".to_string(),
            base_url: "http://a-website.com/".to_string(),
            highlight_code: Some(true),
            highlight_theme: Some("base16-ocean-dark".to_string()),
            description: None,
            language_code: Some("en".to_string()),
            generate_rss: Some(false),
            generate_tags_pages: Some(true),
            generate_categories_pages: Some(true),
            insert_anchor_links: Some(false),

            paginate_by: None,
            paginate_path: Some("page".to_string()),

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
    fn test_can_import_valid_config() {
        let config = r#"
title = "My site"
base_url = "https://replace-this-with-your-url.com"
        "#;

        let config = Config::parse(config).unwrap();
        assert_eq!(config.title, "My site".to_string());
    }

    #[test]
    fn test_errors_when_invalid_type() {
        let config = r#"
title = 1
base_url = "https://replace-this-with-your-url.com"
        "#;

        let config = Config::parse(config);
        assert!(config.is_err());
    }

    #[test]
    fn test_errors_when_missing_required_field() {
        let config = r#"
title = ""
        "#;

        let config = Config::parse(config);
        assert!(config.is_err());
    }

    #[test]
    fn test_can_add_extra_values() {
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
    fn test_language_defaults_to_en() {
        let config = r#"
title = "My site"
base_url = "https://replace-this-with-your-url.com""#;

        let config = Config::parse(config);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.language_code.unwrap(), "en");
    }
}
