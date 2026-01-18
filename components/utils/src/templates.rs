use tera::{Context, Tera};

use errors::{Result, bail};

const DEFAULT_TPL: &str = include_str!("default_tpl.html");

macro_rules! render_default_tpl {
    ($filename: expr, $url: expr) => {{
        let mut context = Context::new();
        context.insert("filename", $filename);
        context.insert("url", $url);
        Tera::one_off(DEFAULT_TPL, &context, true).map_err(std::convert::Into::into)
    }};
}

/// Renders the given template with the given context.
/// Tera handles fallback resolution automatically via `fallback_prefixes`.
/// If it's a default template (index, section or page), it will return a placeholder
/// to avoid an error if there isn't a template with that name.
pub fn render_template(name: &str, tera: &Tera, context: Context) -> Result<String> {
    // First check if the template exists
    if tera.get_template(name).is_none() {
        // Template not found - use fallback for known types
        return match name {
            "index.html" | "section.html" => render_default_tpl!(
                name,
                "https://www.getzola.org/documentation/templates/pages-sections/#section-variables"
            ),
            "page.html" => render_default_tpl!(
                name,
                "https://www.getzola.org/documentation/templates/pages-sections/#page-variables"
            ),
            "single.html" | "list.html" => render_default_tpl!(
                name,
                "https://www.getzola.org/documentation/templates/taxonomies/"
            ),
            _ => bail!("Template `{}` not found", name),
        };
    }

    // Template exists, render it and propagate any errors
    tera.render(name, &context).map_err(Into::into)
}
