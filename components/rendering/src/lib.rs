extern crate pulldown_cmark;
extern crate slug;
extern crate syntect;
extern crate tera;
#[macro_use]
extern crate serde_derive;
extern crate pest;
extern crate serde;
#[macro_use]
extern crate pest_derive;
extern crate regex;
#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate errors;
extern crate config;
extern crate front_matter;
extern crate link_checker;
extern crate utils;

#[cfg(test)]
extern crate templates;

mod context;
mod markdown;
mod shortcode;
mod table_of_contents;

use errors::Result;

pub use context::RenderContext;
use markdown::markdown_to_html;
pub use shortcode::render_shortcodes;
pub use table_of_contents::Header;

pub fn render_content(content: &str, context: &RenderContext) -> Result<markdown::Rendered> {
    // Don't do shortcodes if there is nothing like a shortcode in the content
    if content.contains("{{") || content.contains("{%") {
        let rendered = render_shortcodes(content, context)?;
        return markdown_to_html(&rendered, context);
    }

    markdown_to_html(&content, context)
}
