use console;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkCheckerLevel {
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "warn")]
    Warn,
}

impl Default for LinkCheckerLevel {
    fn default() -> Self {
        Self::Error
    }
}

impl LinkCheckerLevel {
    pub fn log(&self, msg: String) {
        match self {
            LinkCheckerLevel::Error => {
                console::error(format!("{}{}", self.log_prefix(), msg).as_str())
            }
            LinkCheckerLevel::Warn => {
                console::warn(format!("{}{}", self.log_prefix(), msg).as_str())
            }
        }
    }
    pub fn log_prefix(&self) -> &str {
        match self {
            LinkCheckerLevel::Error => "Error: ",
            LinkCheckerLevel::Warn => "Warning: ",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LinkChecker {
    /// Skip link checking for these URL prefixes
    pub skip_prefixes: Vec<String>,
    /// Skip anchor checking for these URL prefixes
    pub skip_anchor_prefixes: Vec<String>,
    /// Emit either "error" or "warn" for broken internal links (including anchor links).
    pub internal_level: LinkCheckerLevel,
    /// Emit either "error" or "warn" for broken external links (including anchor links).
    pub external_level: LinkCheckerLevel,
}
