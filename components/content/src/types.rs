use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SortBy {
    /// Most recent to oldest
    Date,
    /// Most recent to oldest
    #[serde(rename = "update_date")]
    UpdateDate,
    /// Sort by title lexicographically
    Title,
    /// Sort by titles using the bytes directly
    #[serde(rename = "title_bytes")]
    TitleBytes,
    /// Lower weight comes first
    Weight,
    /// Sort by slug
    Slug,
    /// No sorting
    None,
}
