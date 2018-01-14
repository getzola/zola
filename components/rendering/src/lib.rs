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

extern crate errors;
extern crate front_matter;
extern crate highlighting;
extern crate utils;

#[cfg(test)]
extern crate templates;

mod ast;
mod ast_html;
mod collect_while;
mod context;
mod markdown;
mod short_code;
mod table_of_contents;

pub use context::Context;
pub use markdown::markdown_to_html;
pub use table_of_contents::Header;
