use std::collections::HashMap;

use regex::Regex;
use tera::{Tera, Context, Value, to_value};

use errors::{Result, ResultExt};

lazy_static!{
    // Does this look like a shortcode?
    pub static ref SHORTCODE_RE: Regex = Regex::new(
        r#"\{(?:%|\{)\s+(\w+?)\((\w+?="?(?:.|\n)+?"?)?\)\s+(?:%|\})\}"#
    ).unwrap();

    // Parse the shortcode args with capture groups named after their type
    pub static ref SHORTCODE_ARGS_RE: Regex = Regex::new(
        r#"(?P<name>\w+)=\s*((?P<str>".*?")|(?P<float>[-+]?[0-9]+\.[0-9]+)|(?P<int>[-+]?[0-9]+)|(?P<bool>true|false))"#
    ).unwrap();
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
        for arg_cap in SHORTCODE_ARGS_RE.captures_iter(arg_list.as_str()) {
            let arg_name = arg_cap["name"].trim().to_string();

            if let Some(arg_val) = arg_cap.name("str") {
                args.insert(arg_name, to_value(arg_val.as_str().replace("\"", "")).unwrap());
                continue;
            }

            if let Some(arg_val) = arg_cap.name("int") {
                args.insert(arg_name, to_value(arg_val.as_str().parse::<i64>().unwrap()).unwrap());
                continue;
            }

            if let Some(arg_val) = arg_cap.name("float") {
                args.insert(arg_name, to_value(arg_val.as_str().parse::<f64>().unwrap()).unwrap());
                continue;
            }

            if let Some(arg_val) = arg_cap.name("bool") {
                args.insert(arg_name, to_value(arg_val.as_str() == "true").unwrap());
                continue;
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
            // https://github.com/Keats/gutenberg/issues/229
            r#"{{ youtube(id="dQw4w9WgXcQ",

           autoplay=true) }}"#,
        ];

        for i in inputs {
            println!("{}", i);
            assert!(SHORTCODE_RE.is_match(i));
        }
    }

    // https://github.com/Keats/gutenberg/issues/228
    #[test]
    fn doesnt_panic_on_invalid_shortcode() {
        let (name, args) = parse_shortcode(r#"{{ youtube(id="dQw4w9WgXcQ", autoplay) }}"#);
        assert_eq!(name, "youtube");
        assert_eq!(args["id"], "dQw4w9WgXcQ");
        assert!(args.get("autoplay").is_none());
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
        let (name, args) = parse_shortcode(r#"{% test(int=42, float=42.0, autoplay=false) %}"#);
        assert_eq!(name, "test");
        assert_eq!(args["int"], 42);
        assert_eq!(args["float"], 42.0);
        assert_eq!(args["autoplay"], false);
    }

    // https://github.com/Keats/gutenberg/issues/249
    #[test]
    fn can_parse_shortcode_with_comma_in_it() {
        let (name, args) = parse_shortcode(
            r#"{% quote(author="C++ Standard Core Language Defect Reports and Accepted Issues, Revision 82, delete and user-written deallocation function", href="http://www.open-std.org/jtc1/sc22/wg21/docs/cwg_defects.html#348") %}"#
        );
        assert_eq!(name, "quote");
        assert_eq!(args["author"], "C++ Standard Core Language Defect Reports and Accepted Issues, Revision 82, delete and user-written deallocation function");
        assert_eq!(args["href"], "http://www.open-std.org/jtc1/sc22/wg21/docs/cwg_defects.html#348");
    }
}
