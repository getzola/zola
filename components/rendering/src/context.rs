use std::collections::HashMap;

use tera::Tera;

use front_matter::InsertAnchor;


/// All the information from the gutenberg site that is needed to render HTML from markdown
#[derive(Debug)]
pub struct Context<'a> {
    pub tera: &'a Tera,
    pub highlight_code: bool,
    pub highlight_theme: String,
    pub current_page_permalink: String,
    pub permalinks: &'a HashMap<String, String>,
    pub insert_anchor: InsertAnchor,
}

impl<'a> Context<'a> {
    pub fn new(
        tera: &'a Tera,
        highlight_code: bool,
        highlight_theme: String,
        current_page_permalink: &str,
        permalinks: &'a HashMap<String, String>,
        insert_anchor: InsertAnchor,
    ) -> Context<'a> {
        Context {
            tera,
            current_page_permalink: current_page_permalink.to_string(),
            permalinks,
            insert_anchor,
            highlight_code,
            highlight_theme,
        }
    }

    pub fn should_insert_anchor(&self) -> bool {
        self.insert_anchor != InsertAnchor::None
    }
}
