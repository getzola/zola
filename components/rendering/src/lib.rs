extern crate tera;
extern crate syntect;
extern crate pulldown_cmark;
extern crate slug;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate pest;
#[macro_use]
extern crate pest_derive;

#[macro_use]
extern crate errors;
extern crate front_matter;
extern crate highlighting;
extern crate utils;
extern crate config;

#[cfg(test)]
extern crate templates;

mod context;
mod markdown;
mod table_of_contents;
mod shortcode;

use errors::Result;

use markdown::markdown_to_html;
pub use table_of_contents::Header;
pub use shortcode::render_shortcodes;
pub use context::RenderContext;

pub fn render_content(content: &str,  context: &RenderContext) -> Result<(String, Vec<Header>)> {
    // Don't do anything if there is nothing like a shortcode in the content
    if content.contains("{{") || content.contains("{%") {
        let rendered = render_shortcodes(content, context.tera, context.config)?;
        return markdown_to_html(&rendered, context);
    }

    markdown_to_html(&content, context)
}
