use std::collections::HashMap;

use tera::{Context, Tera};
use unic_langid::LanguageIdentifier;

use config::Config;
use front_matter::InsertAnchor;

/// All the information from the zola site that is needed to render HTML from markdown
#[derive(Debug)]
pub struct RenderContext<'a> {
    pub tera: &'a Tera,
    pub config: &'a Config,
    pub tera_context: Context,
    pub current_page_permalink: &'a str,
    pub permalinks: &'a HashMap<String, String>,
    pub insert_anchor: InsertAnchor,
    pub lang: &'a LanguageIdentifier,
}

impl<'a> RenderContext<'a> {
    pub fn new(
        tera: &'a Tera,
        config: &'a Config,
        current_page_permalink: &'a str,
        permalinks: &'a HashMap<String, String>,
        insert_anchor: InsertAnchor,
        lang: &'a LanguageIdentifier,
    ) -> RenderContext<'a> {
        let mut tera_context = Context::new();
        tera_context.insert("config", &config.get_localized(lang).expect("`lang` in config"));
        RenderContext {
            tera,
            tera_context,
            current_page_permalink,
            permalinks,
            insert_anchor,
            config,
            lang,
        }
    }
}
