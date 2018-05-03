#[macro_use]
extern crate lazy_static;
extern crate regex;
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
mod short_code;

pub use context::Context;
pub use markdown::markdown_to_html;
pub use table_of_contents::Header;
pub use shortcode::render_shortcodes;
