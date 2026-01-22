use errors::{Context as _, Result};
use tera::{Context, Tera};

// TODO: move that in render if cyclic dependency is fixed (render / content / markdown)

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
