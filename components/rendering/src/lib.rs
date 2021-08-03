mod codeblock;
mod context;
mod markdown;
mod shortcode;
mod table_of_contents;

use shortcode::parse::{ fetch_shortcodes, Shortcode };

use errors::Result;

pub struct ContentProcessingContext {
}

impl ContentProcessingContext {
    fn new(_shortcodes: Vec<Shortcode>) -> Self {
        unimplemented!()
    }

    fn has_shortcodes(&self) -> bool {
        unimplemented!()
    }
}

fn render_md_shortcodes(_context: &RenderContext, _processing_context: &mut ContentProcessingContext) -> Result<markdown::Rendered> {
    unimplemented!()
}

fn render_html_shortcodes(_context: &RenderContext, _processing_context: &mut ContentProcessingContext) -> Result<(markdown::Rendered, Vec<String>)> {
    unimplemented!()
}



pub use context::RenderContext;
use markdown::markdown_to_html;
pub use table_of_contents::Heading;

pub fn render_content(content: &str, context: &RenderContext) -> Result<markdown::Rendered> {
    // Shortcode render order:
    // 1. MD shortcodes
    // 2. Embedded MD shortcodes
    // 3. MD -> HTML
    // 4. HTML shortcodes
    // 5. Embedded HTML shortcodes

    // Locate all shortcodes in the current content.
    // TODO: REMOVE UNWRAP
    let shortcodes = fetch_shortcodes(content).unwrap();

    // If no shortcodes are present just render the document.
    if shortcodes.is_empty() {
        return markdown_to_html(content, &context);
    }

    // Declare a context which will keep track of everything.
    let mut processing_context = ContentProcessingContext::new(shortcodes);


    // This will render both top-level and embedded MD shortcodes (Step 1, 2).
    let content = render_md_shortcodes(&context, &mut processing_context)?;

    // Turn the MD into HTML (Step 3).
    let content = markdown_to_html(&content.body, &context)?;

    if !processing_context.has_shortcodes() {
        return Ok(content);
    }

    // This will render both top-level and embedded HTML shortcodes (Step 4, 5).
    let (content, _unprocessed_md) =
        render_html_shortcodes(&context, &mut processing_context)?;

    // TODO: Implement this
    // if do_warn_about_unprocessed_md {
    //     warn_about_unprocessed_md(unprocessed_md);
    // }

    Ok(content)
}
