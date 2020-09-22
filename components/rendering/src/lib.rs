mod context;
mod markdown;
mod shortcode;
mod table_of_contents;

use errors::Result;

pub use context::RenderContext;
use markdown::markdown_to_html;
pub use shortcode::render_shortcodes;
pub use table_of_contents::Heading;

pub fn render_content(content: &str, context: &RenderContext) -> Result<markdown::Rendered> {
    // Don't do shortcodes if there is nothing like a shortcode in the content
    if content.contains("{{") || content.contains("{%") {
        let rendered = render_shortcodes(content, context)?;
        let html = markdown_to_html(&rendered, context)?;
        return Ok(html);
    }

    markdown_to_html(&content, context)
}
