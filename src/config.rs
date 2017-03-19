use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::collections::HashMap;

use toml::{Value as Toml, self};

use errors::{Result, ResultExt};


// TO ADD:
// highlight code theme
// generate_tags_pages
// generate_categories_pages

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// Title of the site
    pub title: String,
    /// Base URL of the site
    pub base_url: String,

    /// Whether to highlight all code blocks found in markdown files. Defaults to false
    pub highlight_code: Option<bool>,
    /// Description of the site
    pub description: Option<String>,
    /// The language used in the site. Defaults to "en"
    pub language_code: Option<String>,
    /// Whether to generate RSS, defaults to false
    pub generate_rss: Option<bool>,

    /// All user params set in [extra] in the config
    pub extra: Option<HashMap<String, Toml>>,
}

impl Config {
    /// Parses a string containing TOML to our Config struct
    /// Any extra parameter will end up in the extra field
    pub fn parse(content: &str) -> Result<Config> {
        let mut config: Config = match toml::from_str(content) {
            Ok(c) => c,
            Err(e) => bail!(e)
        };

        if config.language_code.is_none() {
            config.language_code = Some("en".to_string());
        }

        if config.highlight_code.is_none() {
            config.highlight_code = Some(false);
        }

        if config.generate_rss.is_none() {
            config.generate_rss = Some(false);
        }

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
}

impl Default for Config {
    /// Exists for testing purposes
    fn default() -> Config {
        Config {
            title: "".to_string(),
            base_url: "http://a-website.com/".to_string(),
            highlight_code: Some(true),
            description: None,
            language_code: Some("en".to_string()),
            generate_rss: Some(false),
            extra: None,
        }
    }
}


/// Get and parse the config.
/// If it doesn't succeed, exit
pub fn get_config(path: &Path) -> Config {
    match Config::from_file(path.join("config.toml")) {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to load config.toml");
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
