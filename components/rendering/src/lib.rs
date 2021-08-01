mod codeblock;
mod context;
mod markdown;
mod shortcode;
mod table_of_contents;

use errors::Result;

pub use context::RenderContext;
use markdown::markdown_to_html;
pub use shortcode::{locate_shortcodes};
pub use table_of_contents::Heading;

pub fn render_content(content: &str, context: &RenderContext) -> Result<markdown::Rendered> {
    // Shortcode render order:
    // 1. MD shortcodes
    // 2. Embedded MD shortcodes
    // 3. MD -> HTML
    // 4. HTML shortcodes
    // 5. Embedded HTML shortcodes

    // Locate all shortcodes in the current content.
    let shortcodes = locate_shortcodes(content);

    // Declare a context which will keep track of everything.
    let proccessing_context = ContentProcessingContext::new(shortcodes);

    // If no shortcodes are present just render the document.
    if shortcodes.is_empty() {
        return markdown_to_html(content, &context, &mut proccessing_context);
    }

    // This will render both top-level and embedded MD shortcodes (Step 1, 2).
    let content = render_md_shortcodes(content, &context, &mut proccessing_context)?;

    // Turn the MD into HTML (Step 3).
    let content = markdown_to_html(content, &context, &mut proccessing_context)?;

    if !proccessing_context.has_shortcodes() {
        return content;
    }

    // This will render both top-level and embedded HTML shortcodes (Step 4, 5).
    let (content, unprocessed_md) =
        render_html_shortcodes(content, &context, &mut processing_context)?;

    if do_warn_about_unprocessed_md {
        warn_about_unprocessed_md(unprocessed_md);
    }

    content
}
