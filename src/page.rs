/// A page, can be a blog post or a basic page
use std::collections::HashMap;

use pulldown_cmark as cmark;
use regex::Regex;
use toml::Parser;

use errors::{Result, ErrorKind};


lazy_static! {
    static ref DELIM_RE: Regex = Regex::new(r"\+\+\+\s*\r?\n").unwrap();
}


#[derive(Debug, PartialEq)]
struct Page {
    // <title> of the page
    title: String,
    // the url the page appears at (slug form)
    url: String,
    // the actual content of the page
    content: String,
    // tags, not to be confused with categories
    tags: Vec<String>,
    // any extra parameter present in the front matter
    // it will be passed to the template context
    extra: HashMap<String, String>,

    // only one category allowed
    category: Option<String>,
    // optional date if we want to order pages (ie block)
    date: Option<bool>,
    // optional layout, if we want to specify which html to render for that page
    layout: Option<String>,
    // description that appears when linked, e.g. on twitter
    description: Option<String>,
}


impl Page {
    // Parse a page given the content of the .md file
    // Files without front matter or with invalid front matter are considered
    // erroneous
    pub fn from_str(filename: &str, content: &str) -> Result<()> {
        // 1. separate front matter from content
        if !DELIM_RE.is_match(content) {
            return Err(ErrorKind::InvalidFrontMatter(filename.to_string()).into());
        }

        // 2. extract the front matter and the content
        let splits: Vec<&str> = DELIM_RE.splitn(content, 2).collect();
        let front_matter = splits[0];
        let content = splits[1];

        // 2. parse front matter
        let mut parser = Parser::new(&front_matter);
        if let Some(value) = parser.parse() {

        } else {
            // TODO: handle error in parsing TOML
            println!("parse errors: {:?}", parser.errors);
        }

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_can_extract_front_matter() {

    }
}
