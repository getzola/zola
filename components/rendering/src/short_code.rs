use std::collections::HashMap;

use regex::Regex;
use tera::{Tera, Context, Value, to_value};

use errors::{Result, ResultExt};

lazy_static!{
    pub static ref SHORTCODE_RE: Regex = Regex::new(r#"\{(?:%|\{)\s+([[:word:]]+?)\(([[:word:]]+?="?.+?"?)?\)\s+(?:%|\})\}"#).unwrap();
}

/// A shortcode that has a body
/// Called by having some content like {% ... %} body {% end %}
/// We need the struct to hold the data while we're processing the markdown
#[derive(Debug)]
pub struct ShortCode {
    name: String,
    args: HashMap<String, Value>,
    body: String,
}

impl ShortCode {
    pub fn new(name: &str, args: HashMap<String, Value>) -> ShortCode {
        ShortCode {
            name: name.to_string(),
            args,
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
pub fn parse_shortcode(input: &str) -> (String, HashMap<String, Value>) {
    let mut args = HashMap::new();
    let caps = SHORTCODE_RE.captures(input).unwrap();
    // caps[0] is the full match
    let name = &caps[1];

    if let Some(arg_list) = caps.get(2) {
        for arg in arg_list.as_str().split(',') {
            let bits = arg.split('=').collect::<Vec<_>>();
            let arg_name = bits[0].trim().to_string();
            let arg_val = bits[1].replace("\"", "");

            // Regex captures will be str so we need to figure out if they are
            // actually str or bool/number
            if input.contains(&format!("{}=\"{}\"", arg_name, arg_val)) {
                // that's a str, just add it
                args.insert(arg_name, to_value(arg_val).unwrap());
                continue;
            }

            if input.contains(&format!("{}=true", arg_name)) {
                args.insert(arg_name, to_value(true).unwrap());
                continue;
            }

            if input.contains(&format!("{}=false", arg_name)) {
                args.insert(arg_name, to_value(false).unwrap());
                continue;
            }

            // Not a string or a bool, a number then?
            if arg_val.contains('.') {
                if let Ok(float) = arg_val.parse::<f64>() {
                    args.insert(arg_name, to_value(float).unwrap());
                }
                continue;
            }

            // must be an integer
            if let Ok(int) = arg_val.parse::<i64>() {
                args.insert(arg_name, to_value(int).unwrap());
            }
        }
    }

    (name.to_string(), args)
}

/// Renders a shortcode or return an error
pub fn render_simple_shortcode(tera: &Tera, name: &str, args: &HashMap<String, Value>) -> Result<String> {
    let mut context = Context::new();
    for (key, value) in args.iter() {
        context.add(key, value);
    }
    let tpl_name = format!("shortcodes/{}.html", name);

    tera.render(&tpl_name, &context).chain_err(|| format!("Failed to render {} shortcode", name))
}


#[cfg(test)]
mod tests {
    use super::{parse_shortcode, SHORTCODE_RE};

    #[test]
    fn can_match_all_kinds_of_shortcode() {
        let inputs = vec![
            "{{ basic() }}",
            "{{ basic(ho=1) }}",
            "{{ basic(ho=\"hey\") }}",
            "{{ basic(ho=\"hey_underscore\") }}",
            "{{ basic(ho=\"hey-dash\") }}",
            "{% basic(ho=\"hey-dash\") %}",
            "{% basic(ho=\"hey_underscore\") %}",
            "{% basic() %}",
            "{% quo_te(author=\"Bob\") %}",
            "{{ quo_te(author=\"Bob\") }}",
        ];

        for i in inputs {
            println!("{}", i);
            assert!(SHORTCODE_RE.is_match(i));
        }
    }

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
        assert_eq!(args["autoplay"], true);
    }

    #[test]
    fn can_parse_block_shortcode_several_arg() {
        let (name, args) = parse_shortcode(r#"{% youtube(id="w7Ft2ymGmfc", autoplay=true) %}"#);
        assert_eq!(name, "youtube");
        assert_eq!(args["id"], "w7Ft2ymGmfc");
        assert_eq!(args["autoplay"], true);
    }

    #[test]
    fn can_parse_shortcode_number() {
        let (name, args) = parse_shortcode(r#"{% test(int=42, float=42.0, autoplay=true) %}"#);
        assert_eq!(name, "test");
        assert_eq!(args["int"], 42);
        assert_eq!(args["float"], 42.0);
        assert_eq!(args["autoplay"], true);
    }
}
