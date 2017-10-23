use std::collections::HashMap;

use regex::Regex;
use tera::{Tera, Context};

use errors::{Result, ResultExt};

lazy_static!{
    pub static ref SHORTCODE_RE: Regex = Regex::new(r#"\{(?:%|\{)\s+([[:alnum:]]+?)\(([[:alnum:]]+?="?.+?"?)?\)\s+(?:%|\})\}"#).unwrap();
}

/// A shortcode that has a body
/// Called by having some content like {% ... %} body {% end %}
/// We need the struct to hold the data while we're processing the markdown
#[derive(Debug)]
pub struct ShortCode {
    name: String,
    args: HashMap<String, String>,
    body: String,
}

impl ShortCode {
    pub fn new(name: &str, args: HashMap<String, String>) -> ShortCode {
        ShortCode {
            name: name.to_string(),
            args: args,
            body: String::new(),
        }
    }

    pub fn append(&mut self, text: &str) {
        self.body.push_str(text)
    }

    pub fn render(&self, tera: &Tera) -> Result<String> {
        let mut context = Context::new();
        for (key, value) in &self.args {
            context.add(key, value);
        }
        context.add("body", &self.body);
        let tpl_name = format!("shortcodes/{}.html", self.name);
        tera.render(&tpl_name, &context)
            .chain_err(|| format!("Failed to render {} shortcode", self.name))
    }
}

/// Parse a shortcode without a body
pub fn parse_shortcode(input: &str) -> (String, HashMap<String, String>) {
    let mut args = HashMap::new();
    let caps = SHORTCODE_RE.captures(input).unwrap();
    // caps[0] is the full match
    let name = &caps[1];

    if let Some(arg_list) = caps.get(2) {
        for arg in arg_list.as_str().split(',') {
            let bits = arg.split('=').collect::<Vec<_>>();
            args.insert(bits[0].trim().to_string(), bits[1].replace("\"", ""));
        }
    }

    (name.to_string(), args)
}

/// Renders a shortcode or return an error
pub fn render_simple_shortcode(tera: &Tera, name: &str, args: &HashMap<String, String>) -> Result<String> {
    let mut context = Context::new();
    for (key, value) in args.iter() {
        context.add(key, value);
    }
    let tpl_name = format!("shortcodes/{}.html", name);

    tera.render(&tpl_name, &context).chain_err(|| format!("Failed to render {} shortcode", name))
}


#[cfg(test)]
mod tests {
    use super::parse_shortcode;

    #[test]
    fn can_parse_simple_shortcode_no_arg() {
        let (name, args) = parse_shortcode(r#"{{ basic() }}"#);
        assert_eq!(name, "basic");
        assert!(args.is_empty());
    }

    #[test]
    fn can_parse_simple_shortcode_one_arg() {
        let (name, args) = parse_shortcode(r#"{{ youtube(id="w7Ft2ymGmfc") }}"#);
        assert_eq!(name, "youtube");
        assert_eq!(args["id"], "w7Ft2ymGmfc");
    }

    #[test]
    fn can_parse_simple_shortcode_several_arg() {
        let (name, args) = parse_shortcode(r#"{{ youtube(id="w7Ft2ymGmfc", autoplay=true) }}"#);
        assert_eq!(name, "youtube");
        assert_eq!(args["id"], "w7Ft2ymGmfc");
        assert_eq!(args["autoplay"], "true");
    }

    #[test]
    fn can_parse_block_shortcode_several_arg() {
        let (name, args) = parse_shortcode(r#"{% youtube(id="w7Ft2ymGmfc", autoplay=true) %}"#);
        assert_eq!(name, "youtube");
        assert_eq!(args["id"], "w7Ft2ymGmfc");
        assert_eq!(args["autoplay"], "true");
    }
}
