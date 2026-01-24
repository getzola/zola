mod context;
mod markdown;

use errors::Result;

pub use crate::markdown::Rendered;
use crate::markdown::markdown_to_html;
pub use context::MarkdownContext;

pub fn render_content(content: &str, context: &MarkdownContext) -> Result<Rendered> {
    markdown_to_html(content, context)
}
