mod context;
mod markdown;

use errors::Result;

pub use crate::markdown::Rendered;
use crate::markdown::markdown_to_html;
pub use context::RenderContext;

pub fn render_content(content: &str, context: &RenderContext) -> Result<markdown::Rendered> {
    markdown_to_html(content, context)
}
