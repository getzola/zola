use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InsertAnchor {
    /// To the left of the content
    Left,
    /// To the right of the content
    Right,
    /// Wraps the entire content in a <a>..</a> without any icons
    Heading,
    None,
}

impl InsertAnchor {
    pub fn uses_template(&self) -> bool {
        matches!(self, InsertAnchor::Left | InsertAnchor::Right)
    }
}
