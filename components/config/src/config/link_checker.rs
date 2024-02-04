use libs::globset::GlobSet;
use serde::{Deserialize, Serialize};

use errors::Result;
use utils::globs::build_ignore_glob_set;

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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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
    /// A list of file glob patterns to skip link checking on
    pub ignored_files: Vec<String>,
    #[serde(skip_serializing, skip_deserializing)] // not a typo, 2 are needed
    pub ignored_files_globset: Option<GlobSet>,
}

impl LinkChecker {
    pub fn resolve_globset(&mut self) -> Result<()> {
        let glob_set = build_ignore_glob_set(&self.ignored_files, "files")?;
        self.ignored_files_globset = Some(glob_set);
        Ok(())
    }
}
