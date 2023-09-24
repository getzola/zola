use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum IndexFormat {
    ElasticlunrJson,
    #[default]
    ElasticlunrJavascript,
}

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
    /// Include the path of the page in the search index. `false` by default.
    pub include_path: bool,
    /// Foramt of the search index to be produced. Javascript by default
    pub index_format: IndexFormat,
}

impl Default for Search {
    fn default() -> Self {
        Search {
            include_title: true,
            include_content: true,
            include_description: false,
            include_path: false,
            truncate_content_length: None,
            index_format: Default::default(),
        }
    }
}

impl Search {
    pub fn serialize(&self) -> SerializedSearch {
        SerializedSearch { index_format: &self.index_format }
    }
}

#[derive(Serialize)]
pub struct SerializedSearch<'a> {
    pub index_format: &'a IndexFormat,
}
