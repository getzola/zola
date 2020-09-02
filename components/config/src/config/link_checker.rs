use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LinkChecker {
    /// Skip link checking for these URL prefixes
    pub skip_prefixes: Vec<String>,
    /// Skip anchor checking for these URL prefixes
    pub skip_anchor_prefixes: Vec<String>,
}

impl Default for LinkChecker {
    fn default() -> LinkChecker {
        LinkChecker { skip_prefixes: Vec::new(), skip_anchor_prefixes: Vec::new() }
    }
}
