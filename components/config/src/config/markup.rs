use giallo::{HighlightOptions, Registry, ThemeVariant};
use serde::{Deserialize, Serialize};
use std::path::Path;

use errors::{Result, bail};
use utils::types::InsertAnchor;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HighlightStyle {
    Inline,
    Class,
}

impl Default for HighlightStyle {
    fn default() -> HighlightStyle {
        HighlightStyle::Inline
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HighlightConfig {
    Single { theme: String },
    Dual { light_theme: String, dark_theme: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Highlighting {
    /// Emit an error for missing highlight languages. Defaults to false
    #[serde(default)]
    pub error_on_missing_language: bool,
    #[serde(default)]
    pub style: HighlightStyle,
    #[serde(flatten)]
    pub theme: HighlightConfig,
    #[serde(default)]
    pub extra_grammars: Vec<String>,
    #[serde(default)]
    pub extra_themes: Vec<String>,
    #[serde(skip, default)]
    pub registry: Registry,
}

impl Highlighting {
    pub fn init(&mut self, config_dir: &Path) -> Result<()> {
        let mut registry = Registry::builtin()?;

        for grammar in &self.extra_grammars {
            registry.add_grammar_from_path(config_dir.join(grammar))?;
        }

        for theme in &self.extra_themes {
            registry.add_theme_from_path(config_dir.join(theme))?;
        }

        registry.link_grammars();

        match &self.theme {
            HighlightConfig::Single { theme } => {
                if !registry.contains_theme(&theme) {
                    bail!("Theme `{theme}` does not exist");
                }
            }
            HighlightConfig::Dual { light_theme, dark_theme } => {
                if !registry.contains_theme(&light_theme) {
                    bail!("Theme `{light_theme}` does not exist");
                }

                if !registry.contains_theme(&dark_theme) {
                    bail!("Theme `{dark_theme}` does not exist");
                }
            }
        }

        self.registry = registry;

        Ok(())
    }

    pub fn uses_classes(&self) -> bool {
        self.style == HighlightStyle::Class
    }

    pub fn generate_themes_css(&self) -> Vec<(&'static str, String)> {
        let mut out = Vec::new();

        if self.style == HighlightStyle::Inline {
            return out;
        }

        // we know themes are present so unwrap
        match &self.theme {
            HighlightConfig::Single { theme } => {
                out.push((
                    "giallo.css",
                    self.registry.generate_css(theme, "z-").expect("theme to be present"),
                ));
            }
            HighlightConfig::Dual { light_theme, dark_theme } => {
                out.push((
                    "giallo-light.css",
                    self.registry.generate_css(light_theme, "z-").expect("theme to be present"),
                ));
                out.push((
                    "giallo-dark.css",
                    self.registry.generate_css(dark_theme, "z-").expect("theme to be present"),
                ));
            }
        }

        out
    }

    pub fn highlight_options<'a>(&'a self, lang: &'a str) -> HighlightOptions {
        let mut opt = match &self.theme {
            HighlightConfig::Single { theme } => {
                HighlightOptions::new(lang, ThemeVariant::Single(theme))
            }
            HighlightConfig::Dual { light_theme, dark_theme } => HighlightOptions::new(
                lang,
                ThemeVariant::Dual { light: light_theme, dark: dark_theme },
            ),
        };

        if !self.error_on_missing_language {
            opt = opt.fallback_to_plain(true);
        }

        opt
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Markdown {
    /// Syntax highlighting option
    pub highlighting: Option<Highlighting>,
    /// Whether to render emoji aliases (e.g.: :smile: => 😄) in the markdown files
    pub render_emoji: bool,
    /// CSS class to add to external links
    pub external_links_class: Option<String>,
    /// Whether external links are to be opened in a new tab
    /// If this is true, a `rel="noopener"` will always automatically be added for security reasons
    pub external_links_target_blank: bool,
    /// Whether to set rel="nofollow" for all external links
    pub external_links_no_follow: bool,
    /// Whether to set rel="noreferrer" for all external links
    pub external_links_no_referrer: bool,
    /// Whether to set rel="external" for all external links
    pub external_links_external: bool,
    /// Whether smart punctuation is enabled (changing quotes, dashes, dots etc in their typographic form)
    pub smart_punctuation: bool,
    /// Whether parsing of definition lists is enabled
    pub definition_list: bool,
    /// Whether footnotes are rendered at the bottom in the style of GitHub.
    pub bottom_footnotes: bool,
    /// Add loading="lazy" decoding="async" to img tags. When turned on, the alt text must be plain text. Defaults to false
    pub lazy_async_image: bool,
    /// Whether to insert a link for each header like the ones you can see in this site if you hover one
    /// The default template can be overridden by creating a `anchor-link.html` in the `templates` directory
    pub insert_anchor_links: InsertAnchor,
    /// Whether to enable GitHub-style alerts
    pub github_alerts: bool,
}

impl Markdown {
    pub fn validate_external_links_class(&self) -> Result<()> {
        // Validate external link class doesn't contain quotes which would break HTML and aren't valid in CSS
        if let Some(class) = &self.external_links_class
            && (class.contains('"') || class.contains('\''))
        {
            bail!("External link class '{}' cannot contain quotes", class)
        }
        Ok(())
    }

    pub fn has_external_link_tweaks(&self) -> bool {
        self.external_links_target_blank
            || self.external_links_no_follow
            || self.external_links_no_referrer
            || self.external_links_external
            || self.external_links_class.is_some()
    }

    pub fn construct_external_link_tag(&self, url: &str, title: &str) -> String {
        let mut rel_opts = Vec::new();
        let mut target = "".to_owned();
        let title = if title.is_empty() { "".to_owned() } else { format!("title=\"{}\" ", title) };

        let class = self
            .external_links_class
            .as_ref()
            .map_or("".to_owned(), |c| format!("class=\"{}\" ", c));

        if self.external_links_target_blank {
            // Security risk otherwise
            rel_opts.push("noopener");
            target = "target=\"_blank\" ".to_owned();
        }
        if self.external_links_no_follow {
            rel_opts.push("nofollow");
        }
        if self.external_links_no_referrer {
            rel_opts.push("noreferrer");
        }
        if self.external_links_external {
            rel_opts.push("external");
        }
        let rel = if rel_opts.is_empty() {
            "".to_owned()
        } else {
            format!("rel=\"{}\" ", rel_opts.join(" "))
        };

        format!("<a {}{}{}{}href=\"{}\">", class, rel, target, title, url)
    }
}

impl Default for Markdown {
    fn default() -> Markdown {
        Markdown {
            highlighting: None,
            render_emoji: false,
            external_links_class: None,
            external_links_target_blank: false,
            external_links_no_follow: false,
            external_links_no_referrer: false,
            external_links_external: true,
            smart_punctuation: false,
            definition_list: false,
            bottom_footnotes: false,
            lazy_async_image: false,
            insert_anchor_links: InsertAnchor::None,
            github_alerts: false,
        }
    }
}
