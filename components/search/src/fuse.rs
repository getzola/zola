use config::Search;
use content::Library;
use errors::Result;

use crate::{clean_and_truncate_body, collect_index_items};

/// build index in Fuse.js format.
pub fn build_index(lang: &str, library: &Library, config: &Search) -> Result<String> {
    #[derive(serde::Serialize)]
    struct FuseIndexItem<'a> {
        url: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        body: Option<String>, // AMMONIA.clean has to allocate anyway
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<&'a str>,
    }
    let mut index: Vec<FuseIndexItem> = Vec::new();
    let items = collect_index_items(lang, library);
    for item in &items {
        index.push(FuseIndexItem {
            url: item.url,
            title: match config.include_title {
                true => Some(item.title.as_deref().unwrap_or_default()),
                false => None,
            },
            description: match config.include_description {
                true => Some(item.description.as_deref().unwrap_or_default()),
                false => None,
            },
            body: match config.include_content {
                true => Some(clean_and_truncate_body(config.truncate_content_length, item.content)),
                false => None,
            },
            path: match config.include_path {
                true => Some(item.path),
                false => None,
            },
        });
    }
    Ok(serde_json::to_string(&index)?)
}
