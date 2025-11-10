use std::collections::HashMap;
use std::sync::Arc;

use libs::tera::{Function as TeraFn, Result as TeraResult, Value, to_value};

use super::shortcode_registry::ShortcodeRegistry;

/// Tera function for rendering shortcodes
/// Registered for each shortcode, captures the shortcode name and registry
pub struct ShortcodeFunction {
    name: String,
    registry: Arc<ShortcodeRegistry>,
}

impl ShortcodeFunction {
    pub fn new(name: String, registry: Arc<ShortcodeRegistry>) -> Self {
        Self { name, registry }
    }
}

impl TeraFn for ShortcodeFunction {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        // Get the current rendering context from thread-local storage
        let mut context = crate::shortcode_context::get_context().ok_or_else(|| {
            format!(
                "Shortcode '{}' called outside of rendering context. \
                 This is a bug in Zola - shortcodes should only be called during template rendering.",
                self.name
            )
        })?;

        // Add all shortcode arguments to the context
        for (key, value) in args {
            context.insert(key, value);
        }

        // Render the shortcode using the registry
        let html = self
            .registry
            .render_for_html(&self.name, &context)
            .map_err(|e| format!("Failed to render shortcode '{}': {}", self.name, e))?;

        // Return as HTML value (Tera will not escape it)
        Ok(to_value(html).unwrap())
    }
}

