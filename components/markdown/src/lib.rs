mod context;
mod markdown;
mod wikilinks;

use errors::Result;

pub use context::{MarkdownContext, TaxonomyPermalinks};
pub use markdown::Rendered;
pub use wikilinks::{WikilinkError, WikilinkResolver, WikilinkTarget};

pub fn render_content(content: &str, context: &MarkdownContext) -> Result<Rendered> {
    markdown::State::default().render(content, context)
}
