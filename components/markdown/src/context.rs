use std::collections::HashMap;

use config::Config;
use tera::Tera;
use utils::types::InsertAnchor;

/// All the information from the zola site that is needed to render HTML from markdown
pub struct MarkdownContext<'a> {
    pub tera: &'a Tera,
    pub config: &'a Config,
    pub permalinks: &'a HashMap<String, String>,
    pub lang: &'a str,
    pub current_permalink: &'a str,
    pub current_path: &'a str,
    pub insert_anchor: InsertAnchor,
}
