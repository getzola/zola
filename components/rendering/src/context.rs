use std::borrow::Cow;
use std::collections::HashMap;

use config::Config;
use front_matter::InsertAnchor;
use tera::{Context, Tera};

use crate::shortcode::{ShortcodeDefinition, ShortcodeFileType};

/// All the information from the zola site that is needed to render HTML from markdown
#[derive(Debug)]
pub struct RenderContext<'a> {
    pub tera: Cow<'a, Tera>,
    pub config: &'a Config,
    pub tera_context: Context,
    pub current_page_permalink: &'a str,
    pub permalinks: Cow<'a, HashMap<String, String>>,
    pub insert_anchor: InsertAnchor,
    pub shortcode_definitions: HashMap<String, ShortcodeDefinition>,
}

impl<'a> RenderContext<'a> {
    pub fn new(
        tera: &'a Tera,
        config: &'a Config,
        lang: &'a str,
        current_page_permalink: &'a str,
        permalinks: &'a HashMap<String, String>,
        insert_anchor: InsertAnchor,
    ) -> RenderContext<'a> {
        let mut tera_context = Context::new();
        tera_context.insert("config", &config.serialize(lang));
        Self {
            tera: Cow::Borrowed(tera),
            tera_context,
            current_page_permalink,
            permalinks: Cow::Borrowed(permalinks),
            insert_anchor,
            config,
            shortcode_definitions: HashMap::new(),
        }
    }

    // In use in the markdown filter
    pub fn from_config(config: &'a Config) -> RenderContext<'a> {
        Self {
            tera: Cow::Owned(Tera::default()),
            tera_context: Context::new(),
            current_page_permalink: "",
            permalinks: Cow::Owned(HashMap::new()),
            insert_anchor: InsertAnchor::None,
            config,
            shortcode_definitions: HashMap::new(),
        }
    }

    /// Add shortcode definition
    pub fn add_shortcode_def(&mut self, name: &str, is_md: bool, content: &str) {
        self.shortcode_definitions.insert(
            name.to_string(),
            ShortcodeDefinition::new(
                if is_md { ShortcodeFileType::Markdown } else { ShortcodeFileType::HTML },
                content,
            ),
        );
    }
}
