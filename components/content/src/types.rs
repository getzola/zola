use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortBy {
    /// Most recent to oldest
    Date,
    /// Most recent to oldest
    UpdateDate,
    /// Sort by title lexicographically
    Title,
    /// Lower weight comes first
    Weight,
    /// No sorting
    None,
}
