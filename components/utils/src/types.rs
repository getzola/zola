use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InsertAnchor {
    Left,
    Right,
    Heading,
    None,
}

impl InsertAnchor {
    pub fn uses_template(&self) -> bool {
        matches!(self, InsertAnchor::Left | InsertAnchor::Right)
    }
}
