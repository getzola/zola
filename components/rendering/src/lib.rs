mod context;
mod markdown;
mod shortcode;
mod table_of_contents;

use errors::Result;

pub use context::RenderContext;
use markdown::markdown_to_html;
pub use shortcode::render_shortcodes;
pub use table_of_contents::Heading;

pub fn render_plain_content(content: &str, context: &RenderContext) -> Result<String> {
    // Don't do shortcodes if there is nothing like a shortcode in the content
    if content.contains("{{") || content.contains("{%") {
        return render_shortcodes(content, context);
    }
    Ok(content.to_string())
}

pub fn render_html_content(content: &str, context: &RenderContext) -> Result<markdown::Rendered> {
    markdown_to_html(&content, context)
}

/// The functions render_plain_content and render_html_content
/// were split so the intermediate result may be used.
/// Tnis is left for testing purposes
pub fn render_content(content: &str, context: &RenderContext) -> Result<markdown::Rendered> {
    render_plain_content(content, context)
        .and_then(|res| render_html_content(&res, context))
}
