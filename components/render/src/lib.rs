mod cache;
mod pagination;
mod renderer;

use errors::{Context as _, Result, bail};
use tera::{Context, Tera};

pub use cache::RenderCache;
pub use pagination::{Pager, Paginator};
pub use renderer::{FeedInput, Renderer};

// TODO: add a custom default theme?

const DEFAULT_TPL: &str = include_str!("default_tpl.html");

macro_rules! render_default_tpl {
    ($filename: expr, $url: expr) => {{
        let mut context = Context::new();
        context.insert("filename", $filename);
        context.insert("url", $url);
        Tera::one_off(DEFAULT_TPL, &context, true).map_err(std::convert::Into::into)
    }};
}

pub(crate) fn render_template(name: &str, tera: &Tera, context: Context) -> Result<String> {
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

pub fn render_anchor_link(tera: &Tera, id: &str, level: u32, lang: &str) -> Result<String> {
    let mut context = Context::new();
    context.insert("id", id);
    context.insert("level", &level);
    context.insert("lang", lang);
    tera.render("anchor-link.html", &context).context("Failed to render anchor link template")
}

pub fn render_summary_cutoff(tera: &Tera, summary: &str, lang: &str) -> Result<String> {
    let mut context = Context::new();
    context.insert("summary", summary);
    context.insert("lang", lang);
    tera.render("summary-cutoff.html", &context).context("Failed to render summary cutoff template")
}
