use std::sync::{Arc, Mutex};

use errors::{Context as ErrorContext, Result, anyhow};
use libs::dashmap::DashMap;
use libs::tera::{Context, Tera};

/// Registry of all shortcodes available in the site
#[derive(Debug)]
pub struct ShortcodeRegistry {
    /// Map of shortcode name to its templates
    pub(crate) shortcodes: DashMap<String, ShortcodeEntry>,
    /// Tera instance for rendering shortcode templates
    tera: Arc<Mutex<Tera>>,
}

/// Entry for a single shortcode, may have markdown and/or HTML templates
#[derive(Clone, Debug)]
pub struct ShortcodeEntry {
    /// Markdown template (if .md file exists)
    pub markdown_template: Option<String>,
    /// HTML template (from .html file or computed from .md)
    pub html_template: Option<HtmlTemplate>,
}

/// Source of HTML template for a shortcode
#[derive(Clone, Debug)]
pub enum HtmlTemplate {
    /// Loaded from shortcodes/name.html on disk
    FromDisk(String),
    /// Generated from markdown_template by applying markdown filter
    Computed(String),
    /// Future: cached rendered output with expiry
    #[allow(dead_code)]
    Cached(String),
}

impl ShortcodeRegistry {
    /// Create a new shortcode registry by scanning Tera templates
    /// Note: This needs a mutable Tera to register computed templates
    pub fn new(tera: &mut libs::tera::Tera) -> Result<Self> {
        let mut registry = Self { shortcodes: DashMap::new(), tera: Arc::new(Mutex::new(tera.clone())) };

        // Scan Tera templates for shortcodes
        registry.scan_shortcodes_and_register(tera)?;

        Ok(registry)
    }

    /// Scan Tera templates and populate the registry
    /// Also registers computed HTML templates for MD-only shortcodes
    fn scan_shortcodes_and_register(&mut self, tera: &mut Tera) -> Result<()> {
        use std::collections::HashMap;

        // Group templates by shortcode name
        let mut grouped: HashMap<String, (Option<String>, Option<String>)> = HashMap::new();

        for (identifier, template) in tera.templates.iter() {
            // Check if this is a shortcode template
            let shortcode_name = if template.name.starts_with("shortcodes/") {
                let head_len = "shortcodes/".len();
                Some(&identifier[head_len..])
            } else if template.name.starts_with("__zola_builtins/shortcodes/") {
                let head_len = "__zola_builtins/shortcodes/".len();
                Some(&identifier[head_len..])
            } else {
                None
            };

            if let Some(name_with_ext) = shortcode_name {
                // Extract name without extension
                let (name, is_md) = if name_with_ext.ends_with(".md") {
                    (&name_with_ext[..name_with_ext.len() - 3], true)
                } else if name_with_ext.ends_with(".html") {
                    (&name_with_ext[..name_with_ext.len() - 5], false)
                } else {
                    continue; // Unknown extension
                };

                let entry = grouped.entry(name.to_string()).or_insert((None, None));

                if is_md {
                    // User shortcodes override builtins
                    if entry.0.is_none() || !template.name.starts_with("__zola_builtins/") {
                        entry.0 = Some(template.name.clone());
                    }
                } else {
                    // User shortcodes override builtins
                    if entry.1.is_none() || !template.name.starts_with("__zola_builtins/") {
                        entry.1 = Some(template.name.clone());
                    }
                }
            }
        }

        // Create ShortcodeEntry for each shortcode
        for (name, (md_template, html_template)) in grouped {
            let entry = self.create_entry_and_register(name.clone(), md_template, html_template, tera)?;
            self.shortcodes.insert(name, entry);
        }

        Ok(())
    }

    /// Create a ShortcodeEntry from template names
    fn create_entry_and_register(
        &self,
        name: String,
        md_template: Option<String>,
        html_template: Option<String>,
        _tera: &mut Tera,
    ) -> Result<ShortcodeEntry> {
        let html_tpl = match (&html_template, &md_template) {
            // Both exist: use HTML from disk
            (Some(html), _) => Some(HtmlTemplate::FromDisk(html.clone())),
            // Only MD exists: store MD template name for computed rendering
            (None, Some(md)) => Some(HtmlTemplate::Computed(md.clone())),
            // Neither exists: shouldn't happen due to scan logic
            (None, None) => None,
        };

        if html_template.is_some() && md_template.is_some() {
            console::warn(&format!(
                "Shortcode '{}' has both .md and .html templates. Both will be available depending on context.",
                name
            ));
        }

        Ok(ShortcodeEntry { markdown_template: md_template, html_template: html_tpl })
    }

    /// Get a shortcode entry by name
    pub fn get(&self, name: &str) -> Option<libs::dashmap::mapref::one::Ref<'_, String, ShortcodeEntry>> {
        self.shortcodes.get(name)
    }

    /// Render a shortcode for markdown context (uses .md template if available)
    pub fn render_for_markdown(
        &self,
        name: &str,
        context: &Context,
    ) -> Result<String> {
        let entry = self.shortcodes.get(name).ok_or_else(|| {
            anyhow!("Shortcode '{}' not found. Make sure a template shortcodes/{}.{{md,html}} exists.", name, name)
        })?;

        // Prefer markdown template if available
        let template_name = if let Some(ref md_tpl) = entry.markdown_template {
            md_tpl
        } else if let Some(ref html_tpl) = entry.html_template {
            match html_tpl {
                HtmlTemplate::FromDisk(tpl) => tpl,
                _ => {
                    return Err(anyhow!(
                        "Shortcode '{}' has no markdown template and computed HTML cannot be used in markdown context",
                        name
                    ));
                }
            }
        } else {
            return Err(anyhow!("Shortcode '{}' has no available templates", name));
        };

        self.tera
            .lock()
            .unwrap()
            .render(template_name, context)
            .with_context(|| format!("Failed to render shortcode '{}'", name))
    }

    /// Render a shortcode for HTML/Tera context (uses .html template or computed)
    pub fn render_for_html(
        &self,
        name: &str,
        context: &Context,
    ) -> Result<String> {
        let entry = self.shortcodes.get(name).ok_or_else(|| {
            anyhow!("Shortcode '{}' not found. Make sure a template shortcodes/{}.{{md,html}} exists.", name, name)
        })?;

        match &entry.html_template {
            Some(HtmlTemplate::FromDisk(tpl_name)) => {
                // Render HTML template directly
                self.tera
                    .lock()
                    .unwrap()
                    .render(tpl_name, context)
                    .with_context(|| format!("Failed to render shortcode '{}' for HTML context", name))
            }
            Some(HtmlTemplate::Computed(md_tpl_name)) => {
                // For computed: render MD template, then apply markdown filter
                // First render the markdown template
                let md_output = self.tera
                    .lock()
                    .unwrap()
                    .render(md_tpl_name, context)
                    .with_context(|| format!("Failed to render markdown template '{}' for shortcode '{}'", md_tpl_name, name))?;

                // Then apply the markdown filter by rendering it through a temporary template
                // We need to use the markdown filter which is already registered with Tera
                // Use inline=true to strip wrapping paragraph tags
                let filter_template = format!("{{{{ content | markdown(inline=true) | safe }}}}");
                let mut filter_context = Context::new();
                filter_context.insert("content", &md_output);
                // Copy all context for the filter to access config, etc.
                filter_context.extend(context.clone());

                self.tera
                    .lock()
                    .unwrap()
                    .render_str(&filter_template, &filter_context)
                    .with_context(|| format!("Failed to apply markdown filter for shortcode '{}'", name))
            }
            Some(HtmlTemplate::Cached(html)) => {
                // Future: return cached HTML directly
                Ok(html.clone())
            }
            None => {
                Err(anyhow!(
                    "Shortcode '{}' has no HTML template available",
                    name
                ))
            }
        }
    }

    /// Get list of all shortcode names
    pub fn shortcode_names(&self) -> Vec<String> {
        self.shortcodes.iter().map(|entry| entry.key().clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_registry_with_md_shortcode() {
        let mut tera = Tera::default();
        tera.add_raw_template("shortcodes/test.md", "**{{ text }}**").unwrap();

        let registry = ShortcodeRegistry::new(&mut tera).unwrap();
        let entry = registry.get("test").unwrap();

        assert!(entry.markdown_template.is_some());
        assert!(entry.html_template.is_some());
        match &entry.html_template {
            Some(HtmlTemplate::Computed(_)) => {} // Expected
            _ => panic!("Expected Computed HTML template"),
        }
    }

    #[test]
    fn can_create_registry_with_html_shortcode() {
        let mut tera = Tera::default();
        tera.add_raw_template("shortcodes/test.html", "<strong>{{ text }}</strong>").unwrap();

        let registry = ShortcodeRegistry::new(&mut tera).unwrap();
        let entry = registry.get("test").unwrap();

        assert!(entry.markdown_template.is_none());
        assert!(entry.html_template.is_some());
        match &entry.html_template {
            Some(HtmlTemplate::FromDisk(_)) => {} // Expected
            _ => panic!("Expected FromDisk HTML template"),
        }
    }

    #[test]
    fn can_create_registry_with_both_templates() {
        let mut tera = Tera::default();
        tera.add_raw_template("shortcodes/test.md", "**{{ text }}**").unwrap();
        tera.add_raw_template("shortcodes/test.html", "<strong>{{ text }}</strong>").unwrap();

        let registry = ShortcodeRegistry::new(&mut tera).unwrap();
        let entry = registry.get("test").unwrap();

        assert!(entry.markdown_template.is_some());
        assert!(entry.html_template.is_some());
        match &entry.html_template {
            Some(HtmlTemplate::FromDisk(_)) => {} // Expected - HTML template takes precedence
            _ => panic!("Expected FromDisk HTML template when both exist"),
        }
    }
}
