mod codeblock;
mod context;
mod markdown;
mod shortcode;
mod table_of_contents;

use shortcode::{extract_shortcodes, insert_md_shortcodes};

use errors::Result;

pub use context::RenderContext;
use markdown::markdown_to_html;
pub use markdown::Rendered;
pub use table_of_contents::Heading;

pub fn render_content(content: &str, context: &RenderContext) -> Result<markdown::Rendered> {
    // avoid parsing the content if needed
    if !content.contains("{{") && !content.contains("{%") {
        return markdown_to_html(content, context, Vec::new());
    }

    let definitions = context.shortcode_definitions.as_ref();
    // Extract all the defined shortcodes
    let (content, shortcodes) = extract_shortcodes(content, definitions)?;

    // Step 1: we render the MD shortcodes before rendering the markdown so they can get processed
    let (content, html_shortcodes) =
        insert_md_shortcodes(content, shortcodes, &context.tera_context, &context.tera)?;

    // Step 2: we render the markdown and the HTML markdown at the same time
    let html_context = markdown_to_html(&content, context, html_shortcodes)?;

    // TODO: Here issue #1418 could be implemented
    // if do_warn_about_unprocessed_md {
    //     warn_about_unprocessed_md(unprocessed_md);
    // }

    Ok(html_context)
}
