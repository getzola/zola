mod context;
mod markdown;
mod shortcode;
mod table_of_contents;
mod extract_katex;

use errors::Result;

pub use context::RenderContext;
pub use extract_katex::render_katex;
use markdown::markdown_to_html;
pub use shortcode::render_shortcodes;
pub use table_of_contents::Heading;


pub fn render_content(content: &str, context: &RenderContext) -> Result<markdown::Rendered> {
    let mut preprocessed: String = content.to_string();

    // Don't do shortcodes if there is nothing like a shortcode in the content
    if preprocessed.contains("{{") || preprocessed.contains("{%") {
        preprocessed = render_shortcodes(&preprocessed, context)?;
    }

    preprocessed = render_katex(&preprocessed);

    let mut html = markdown_to_html(&preprocessed, context)?;
    html.body = html.body.replace("<!--\\n-->", "\n");
    return Ok(html);
}
