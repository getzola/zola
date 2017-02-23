use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::collections::HashMap;

use toml::{Value as Toml, self};

use errors::{Result, ResultExt};

// TODO: disable tag(s)/category(ies) page generation
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// Title of the site
    pub title: String,
    /// Base URL of the site
    pub base_url: String,
    /// Description of the site
    pub description: Option<String>,
    /// The language used in the site. Defaults to "en"
    pub language_code: Option<String>,
    /// Whether to disable RSS generation, defaults to None (== generate RSS)
    pub disable_rss: Option<bool>,
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
