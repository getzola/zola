use std::collections::HashMap;

use tera::Tera;
use front_matter::InsertAnchor;
use config::Config;


/// All the information from the gutenberg site that is needed to render HTML from markdown
#[derive(Debug)]
pub struct RenderContext<'a> {
    pub tera: &'a Tera,
    pub config: &'a Config,
    pub current_page_permalink: &'a str,
    pub permalinks: &'a HashMap<String, String>,
    pub insert_anchor: InsertAnchor,
}

impl<'a> RenderContext<'a> {
    pub fn new(
        tera: &'a Tera,
        config: &'a Config,
        current_page_permalink: &'a str,
        permalinks: &'a HashMap<String, String>,
        insert_anchor: InsertAnchor,
    ) -> RenderContext<'a> {
        RenderContext {
            tera,
            current_page_permalink,
            permalinks,
            insert_anchor,
            config,
        }
    }

}
