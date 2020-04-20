use std::collections::HashMap;
use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};
use toml::Value as Toml;

use crate::config::TranslateTerm;
use errors::{bail, Result};
use utils::fs::read_file_with_error;

/// Holds the data from a `theme.toml` file.
/// There are other fields than `extra` in it but Zola
/// itself doesn't care about them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Theme {
    /// All user params set in [extra] in the theme.toml
    pub extra: HashMap<String, Toml>,
    pub translations: HashMap<String, TranslateTerm>,
}

impl Theme {
    /// Parses a TOML string to our Theme struct
    pub fn parse(content: &str) -> Result<Theme> {
        let theme = match content.parse::<Toml>() {
            Ok(t) => t,
            Err(e) => bail!(e),
        };

        let mut extra = HashMap::new();
        let mut translations = HashMap::new();
        if let Some(theme_table) = theme.as_table() {
            if let Some(ex) = theme_table.get("extra") {
                if ex.is_table() {
                    extra = ex.clone().try_into().unwrap();
                }
            }
            if let Some(t) = theme_table.get("translations") {
                if t.is_table() {
                    translations = t.clone().try_into().unwrap();
                }
            }
        } else {
            bail!("Expected the `theme.toml` to be a TOML table")
        }

        Ok(Theme { extra, translations })
    }

    /// Parses a theme file from the given path
    pub fn from_file(path: &PathBuf) -> Result<Theme> {
        let content = read_file_with_error(
            path,
            "No `theme.toml` file found. \
             Is the `theme` defined in your `config.toml present in the `themes` directory \
             and does it have a `theme.toml` inside?",
        )?;
        Theme::parse(&content)
    }
}
