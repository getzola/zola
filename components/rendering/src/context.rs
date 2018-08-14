use std::collections::HashMap;
use std::path::Path;

use tera::{Tera, Context};
use front_matter::InsertAnchor;
use config::Config;


/// All the information from the gutenberg site that is needed to render HTML from markdown
#[derive(Debug)]
pub struct RenderContext<'a> {
    pub tera: &'a Tera,
    pub config: &'a Config,
    pub tera_context: Context,
    pub current_page_permalink: &'a str,
    pub permalinks: &'a HashMap<String, String>,
    pub base_path: &'a Path,
    pub insert_anchor: InsertAnchor,
}

impl<'a> RenderContext<'a> {
    pub fn new(
        tera: &'a Tera,
        config: &'a Config,
        current_page_permalink: &'a str,
        permalinks: &'a HashMap<String, String>,
        base_path: &'a Path,
        insert_anchor: InsertAnchor,
    ) -> RenderContext<'a> {
        let mut tera_context = Context::new();
        tera_context.insert("config", config);
        RenderContext {
            tera,
            tera_context,
            current_page_permalink,
            permalinks,
            insert_anchor,
            base_path,
            config,
        }
    }
}
