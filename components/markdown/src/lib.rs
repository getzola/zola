mod context;
mod markdown;
mod markdown2;

use errors::Result;

pub use crate::markdown::Rendered;
use crate::markdown::markdown_to_html;
pub use context::MarkdownContext;

pub fn render_content(content: &str, context: &MarkdownContext) -> Result<Rendered> {
    markdown_to_html(content, context)
}
pub fn render_content2(content: &str, context: &MarkdownContext) -> Result<markdown2::Rendered> {
    markdown2::State::default().render(content, context)
}
