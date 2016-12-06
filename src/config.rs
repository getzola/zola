use std::default::Default;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use toml::Parser;

use errors::{Result, ErrorKind};


#[derive(Debug, PartialEq)]
pub struct Config {
    pub title: String,
    pub base_url: String,

    pub favicon: Option<String>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            title: "".to_string(),
            base_url: "".to_string(),

            favicon: None,
        }
    }
}

impl Config {
    pub fn from_str(content: &str) -> Result<Config> {
        let mut parser = Parser::new(&content);

        if let Some(value) = parser.parse() {
            let mut config = Config::default();

            for (key, value) in value.iter() {
                if key == "title" {
                    config.title = value.as_str().ok_or(ErrorKind::InvalidConfig)?.to_string();
                } else if key == "base_url" {
                    config.base_url = value.as_str().ok_or(ErrorKind::InvalidConfig)?.to_string();
                } else if key == "favicon" {
                    config.favicon = Some(value.as_str().ok_or(ErrorKind::InvalidConfig)?.to_string());
                }
            }

            return Ok(config);
        } else {
            // TODO: handle error in parsing TOML
            println!("parse errors: {:?}", parser.errors);
        }

        unreachable!()
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Config> {
        let mut content = String::new();
        File::open(path)?.read_to_string(&mut content)?;

        Config::from_str(&content)
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

        let config = Config::from_str(config).unwrap();
        assert_eq!(config.title, "My site".to_string());
    }

    #[test]
    fn test_errors_when_invalid_type() {
        let config = r#"
title = 1
base_url = "https://replace-this-with-your-url.com"
        "#;

        let config = Config::from_str(config);
        assert!(config.is_err());
    }
}
