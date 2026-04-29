use std::collections::HashMap;

use config::Config;
use pulldown_cmark::Options;
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

impl<'a> MarkdownContext<'a> {
    pub fn options(&self) -> Options {
        let mut opts = Options::empty();
        opts.insert(Options::ENABLE_TABLES);
        opts.insert(Options::ENABLE_FOOTNOTES);
        opts.insert(Options::ENABLE_STRIKETHROUGH);
        opts.insert(Options::ENABLE_TASKLISTS);
        opts.insert(Options::ENABLE_HEADING_ATTRIBUTES);
        // TODO: enable it later
        // opts.insert(Options::ENABLE_WIKILINKS);

        if self.config.markdown.smart_punctuation {
            opts.insert(Options::ENABLE_SMART_PUNCTUATION);
        }
        if self.config.markdown.definition_list {
            opts.insert(Options::ENABLE_DEFINITION_LIST);
        }
        if self.config.markdown.github_alerts {
            opts.insert(Options::ENABLE_GFM);
        }
        opts
    }
}
