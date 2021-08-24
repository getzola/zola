mod codeblock;
mod context;
mod markdown;
mod shortcode;
mod table_of_contents;
mod transform;
mod range_relation;

use shortcode::{insert_shortcodes, ShortcodeFileType};

use errors::Result;

pub use context::RenderContext;
use markdown::markdown_to_html;
pub use table_of_contents::Heading;

use std::collections::HashMap;

pub fn render_content(content: &str, context: &RenderContext) -> Result<markdown::Rendered> {
    // Shortcode render order:
    // 1. MD shortcodes
    // 2. Embedded MD shortcodes
    // 3. MD -> HTML
    // 4. HTML shortcodes
    // 5. Embedded HTML shortcodes

    // Fetch all the defined shortcodes
    // TODO: Actually fetch these. This should maybe be handed down by the RenderContext?
    let shortcode_definitions = &context.shortcode_definitions;

    // This will render both top-level and embedded MD shortcodes (Step 1, 2).
    let (content, _) = insert_shortcodes(content, shortcode_definitions, ShortcodeFileType::Markdown, &context.tera_context)
        .map_err(Into::<errors::Error>::into)?;

    // Turn the MD into HTML (Step 3).
    let html_context = markdown_to_html(&content, &context)?;

    // This will render both top-level and embedded HTML shortcodes (Step 4, 5).
    let (content, html_transforms) = insert_shortcodes(&html_context.body, shortcode_definitions, ShortcodeFileType::HTML, &context.tera_context)
        .map_err(Into::<errors::Error>::into)?;

    // TODO: Here issue #1418 could be implemented
    // if do_warn_about_unprocessed_md {
    //     warn_about_unprocessed_md(unprocessed_md);
    // }
    
    Ok(markdown::Rendered::new_with_transforms(&content, html_context, html_transforms))
}
