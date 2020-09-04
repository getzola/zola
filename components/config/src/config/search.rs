use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Search {
    /// Include the title of the page in the search index. `true` by default.
    pub include_title: bool,
    /// Includes the whole content in the search index. Ok for small sites but becomes
    /// too big on large sites. `true` by default.
    pub include_content: bool,
    /// Optionally truncate the content down to `n` chars. This might cut content in a word
    pub truncate_content_length: Option<usize>,
    /// Includes the description in the search index. When the site becomes too large, you can switch
    /// to that instead. `false` by default
    pub include_description: bool,
}

impl Default for Search {
    fn default() -> Self {
        Search {
            include_title: true,
            include_content: true,
            include_description: false,
            truncate_content_length: None,
        }
    }
}
