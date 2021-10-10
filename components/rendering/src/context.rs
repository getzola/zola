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

        let mut shortcode_definitions = HashMap::new();

        for (identifier, template) in tera.templates.iter() {
            let (file_type, ext_len) = if template.name.ends_with(".md") {
                (ShortcodeFileType::Markdown, "md".len())
            } else {
                (ShortcodeFileType::HTML, "html".len())
            };

            if template.name.starts_with("shortcodes/") {
                let head_len = "shortcodes/".len();
                shortcode_definitions.insert(
                    identifier[head_len..(identifier.len() - ext_len - 1)].to_string(),
                    ShortcodeDefinition { file_type, tera_name: template.name.clone() },
                );
                continue;
            }

            if template.name.starts_with("__zola_builtins/shortcodes/") {
                let head_len = "__zola_builtins/shortcodes/".len();
                shortcode_definitions.insert(
                    identifier[head_len..(identifier.len() - ext_len - 1)].to_string(),
                    ShortcodeDefinition { file_type, tera_name: template.name.clone() },
                );
                continue;
            }
        }

        Self {
            tera: Cow::Borrowed(tera),
            tera_context,
            current_page_permalink,
            permalinks: Cow::Borrowed(permalinks),
            insert_anchor,
            config,
            shortcode_definitions,
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
}
