use tera::{Tera, Context};

use errors::Result;

/// Renders the given template with the given context, but also ensures that, if the default file
/// is not found, it will look up for the equivalent template for the current theme if there is one
pub fn render_template(name: &str, tera: &Tera, context: &Context, theme: Option<String>) -> Result<String> {
    if tera.templates.contains_key(name) {
        return tera
            .render(name, context)
            .map_err(|e| e.into());
    }

    if let Some(ref t) = theme {
        return tera
            .render(&format!("{}/templates/{}", t, name), context)
            .map_err(|e| e.into());
    }

    bail!("Tried to render `{}` but the template wasn't found", name)
}
