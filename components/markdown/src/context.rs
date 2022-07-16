use std::borrow::Cow;
use std::collections::HashMap;

use config::Config;
use libs::tera::{Context, Tera};
use utils::templates::ShortcodeDefinition;
use utils::types::InsertAnchor;

/// All the information from the zola site that is needed to render HTML from markdown
#[derive(Debug)]
pub struct RenderContext<'a> {
    pub tera: Cow<'a, Tera>,
    pub config: &'a Config,
    pub tera_context: Context,
    pub current_page_path: Option<&'a str>,
    pub current_page_permalink: &'a str,
    pub permalinks: Cow<'a, HashMap<String, String>>,
    pub insert_anchor: InsertAnchor,
    pub lang: &'a str,
    pub shortcode_definitions: Cow<'a, HashMap<String, ShortcodeDefinition>>,
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
        tera_context.insert("lang", lang);

        Self {
            tera: Cow::Borrowed(tera),
            tera_context,
            current_page_path: None,
            current_page_permalink,
            permalinks: Cow::Borrowed(permalinks),
            insert_anchor,
            config,
            lang,
            shortcode_definitions: Cow::Owned(HashMap::new()),
        }
    }

    /// Set in another step so we don't add one more arg to new.
    /// And it's only used when rendering pages/section anyway
    pub fn set_shortcode_definitions(&mut self, def: &'a HashMap<String, ShortcodeDefinition>) {
        self.shortcode_definitions = Cow::Borrowed(def);
    }

    /// Same as above
    pub fn set_current_page_path(&mut self, path: &'a str) {
        self.current_page_path = Some(path);
    }

    // In use in the markdown filter
    // NOTE: This RenderContext is not i18n-aware, see MarkdownFilter::filter for details
    // If this function is ever used outside of MarkdownFilter, take this into consideration
    pub fn from_config(config: &'a Config) -> RenderContext<'a> {
        Self {
            tera: Cow::Owned(Tera::default()),
            tera_context: Context::new(),
            current_page_path: None,
            current_page_permalink: "",
            permalinks: Cow::Owned(HashMap::new()),
            insert_anchor: InsertAnchor::None,
            config,
            lang: &config.default_language,
            shortcode_definitions: Cow::Owned(HashMap::new()),
        }
    }
}
