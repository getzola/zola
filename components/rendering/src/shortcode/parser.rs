use std::ops::Range;

use errors::{bail, Result};
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;
use tera::{to_value, Context, Map, Tera, Value};
use utils::templates::ShortcodeFileType;

pub const SHORTCODE_PLACEHOLDER: &str = "||ZOLA_SC_PLACEHOLDER||";

#[derive(PartialEq, Debug)]
pub struct Shortcode {
    pub(crate) name: String,
    pub(crate) args: Value,
    pub(crate) span: Range<usize>,
    pub(crate) body: Option<String>,
    pub(crate) nth: usize,
    // set later down the line, for quick access without needing the definitions
    pub(crate) tera_name: String,
}

impl Shortcode {
    pub fn file_type(&self) -> ShortcodeFileType {
        if self.tera_name.ends_with("md") {
            ShortcodeFileType::Markdown
        } else {
            ShortcodeFileType::Html
        }
    }

    pub fn render(self, tera: &Tera, context: &Context) -> Result<String> {
        let name = self.name;
        let tpl_name = self.tera_name;
        let mut new_context = Context::from_value(self.args)?;

        if let Some(body_content) = self.body {
            // Trimming right to avoid most shortcodes with bodies ending up with a HTML new line
            new_context.insert("body", body_content.trim_end());
        }
        new_context.insert("nth", &self.nth);
        new_context.extend(context.clone());

        let res = utils::templates::render_template(&tpl_name, tera, new_context, &None)
            .map_err(|e| errors::Error::chain(format!("Failed to render {} shortcode", name), e))?
            .replace("\r\n", "\n");

        Ok(res)
    }

    pub fn update_range(&mut self, sc_span: &Range<usize>, rendered_length: usize) {
        if self.span.start < sc_span.start {
            return;
        }

        let rendered_end = sc_span.start + rendered_length;
        let delta = if sc_span.end < rendered_end {
            rendered_end - sc_span.end
        } else {
            sc_span.end - rendered_end
        };

        if sc_span.end < rendered_end {
            self.span = (self.span.start + delta)..(self.span.end + delta);
        } else {
            self.span = (self.span.start - delta)..(self.span.end - delta);
        }
    }
}

// This include forces recompiling this source file if the grammar file changes.
// Uncomment it when doing changes to the .pest file
const _GRAMMAR: &str = include_str!("../content.pest");

#[derive(Parser)]
#[grammar = "content.pest"]
pub struct ContentParser;

fn replace_string_markers(input: &str) -> String {
    match input.chars().next().unwrap() {
        '"' => input.replace('"', ""),
        '\'' => input.replace('\'', ""),
        '`' => input.replace('`', ""),
        _ => unreachable!("How did you even get there"),
    }
}

fn parse_kwarg_value(pair: Pair<Rule>) -> Value {
    let mut val = None;
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::boolean => match p.as_str() {
                "true" => val = Some(Value::Bool(true)),
                "false" => val = Some(Value::Bool(false)),
                _ => unreachable!(),
            },
            Rule::string => val = Some(Value::String(replace_string_markers(p.as_str()))),
            Rule::float => {
                val = Some(to_value(p.as_str().parse::<f64>().unwrap()).unwrap());
            }
            Rule::int => {
                val = Some(to_value(p.as_str().parse::<i64>().unwrap()).unwrap());
            }
            Rule::array => {
                let mut vals = vec![];
                for p2 in p.into_inner() {
                    match p2.as_rule() {
                        Rule::literal => vals.push(parse_kwarg_value(p2)),
                        _ => unreachable!("Got something other than literal in an array: {:?}", p2),
                    }
                }
                val = Some(Value::Array(vals));
            }
            _ => unreachable!("Unknown literal: {:?}", p),
        };
    }

    val.unwrap()
}

/// Returns (shortcode_name, kwargs)
fn parse_shortcode_call(pair: Pair<Rule>) -> (String, Value) {
    let mut name = None;
    let mut args = Map::new();

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::ident => {
                name = Some(p.as_span().as_str().to_string());
            }
            Rule::kwarg => {
                let mut arg_name = None;
                let mut arg_val = None;
                for p2 in p.into_inner() {
                    match p2.as_rule() {
                        Rule::ident => {
                            arg_name = Some(p2.as_span().as_str().to_string());
                        }
                        Rule::literal => {
                            arg_val = Some(parse_kwarg_value(p2));
                        }
                        _ => unreachable!("Got something unexpected in a kwarg: {:?}", p2),
                    }
                }

                args.insert(arg_name.unwrap(), arg_val.unwrap());
            }
            _ => unreachable!("Got something unexpected in a shortcode: {:?}", p),
        }
    }
    (name.unwrap(), Value::Object(args))
}

pub fn parse_for_shortcodes(content: &str) -> Result<(String, Vec<Shortcode>)> {
    let mut shortcodes = Vec::new();
    let mut nths = HashMap::new();
    let mut get_invocation_count = |name: &str| {
        let nth = nths.entry(String::from(name)).or_insert(0);
        *nth += 1;
        *nth
    };
    let mut output = String::with_capacity(content.len());

    let mut pairs = match ContentParser::parse(Rule::page, content) {
        Ok(p) => p,
        Err(e) => {
            let fancy_e = e.renamed_rules(|rule| match *rule {
                Rule::int => "an integer".to_string(),
                Rule::float => "a float".to_string(),
                Rule::string => "a string".to_string(),
                Rule::literal => "a literal (int, float, string, bool)".to_string(),
                Rule::array => "an array".to_string(),
                Rule::kwarg => "a keyword argument".to_string(),
                Rule::ident => "an identifier".to_string(),
                Rule::inline_shortcode => "an inline shortcode".to_string(),
                Rule::ignored_inline_shortcode => "an ignored inline shortcode".to_string(),
                Rule::sc_body_start => "the start of a shortcode".to_string(),
                Rule::ignored_sc_body_start => "the start of an ignored shortcode".to_string(),
                Rule::text => "some text".to_string(),
                Rule::EOI => "end of input".to_string(),
                Rule::double_quoted_string => "double quoted string".to_string(),
                Rule::single_quoted_string => "single quoted string".to_string(),
                Rule::backquoted_quoted_string => "backquoted quoted string".to_string(),
                Rule::boolean => "a boolean (true, false)".to_string(),
                Rule::all_chars => "a alphanumerical character".to_string(),
                Rule::kwargs => "a list of keyword arguments".to_string(),
                Rule::sc_def => "a shortcode definition".to_string(),
                Rule::shortcode_with_body => "a shortcode with body".to_string(),
                Rule::ignored_shortcode_with_body => "an ignored shortcode with body".to_string(),
                Rule::sc_body_end => "{% end %}".to_string(),
                Rule::ignored_sc_body_end => "{%/* end */%}".to_string(),
                Rule::text_in_body_sc => "text in a shortcode body".to_string(),
                Rule::text_in_ignored_body_sc => "text in an ignored shortcode body".to_string(),
                Rule::content => "some content".to_string(),
                Rule::page => "a page".to_string(),
                Rule::WHITESPACE => "whitespace".to_string(),
            });
            bail!("{}", fancy_e);
        }
    };

    // We have at least a `page` pair
    for p in pairs.next().unwrap().into_inner() {
        match p.as_rule() {
            Rule::text => output.push_str(p.as_span().as_str()),
            Rule::inline_shortcode => {
                let start = output.len();
                let (name, args) = parse_shortcode_call(p);
                let nth = get_invocation_count(&name);
                shortcodes.push(Shortcode {
                    name,
                    args,
                    span: start..(start + SHORTCODE_PLACEHOLDER.len()),
                    body: None,
                    nth,
                    tera_name: String::new(),
                });
                output.push_str(SHORTCODE_PLACEHOLDER);
            }
            Rule::shortcode_with_body => {
                let start = output.len();
                let mut inner = p.into_inner();
                // 3 items in inner: call, body, end
                // we don't care about the closing tag
                let (name, args) = parse_shortcode_call(inner.next().unwrap());
                let body = inner.next().unwrap().as_span().as_str().trim();
                let nth = get_invocation_count(&name);
                shortcodes.push(Shortcode {
                    name,
                    args,
                    span: start..(start + SHORTCODE_PLACEHOLDER.len()),
                    body: Some(body.to_string()),
                    nth,
                    tera_name: String::new(),
                });
                output.push_str(SHORTCODE_PLACEHOLDER)
            }
            Rule::ignored_inline_shortcode => {
                output.push_str(
                    &p.as_span().as_str().replacen("{{/*", "{{", 1).replacen("*/}}", "}}", 1),
                );
            }
            Rule::ignored_shortcode_with_body => {
                for p2 in p.into_inner() {
                    match p2.as_rule() {
                        Rule::ignored_sc_body_start | Rule::ignored_sc_body_end => {
                            output.push_str(
                                &p2.as_span()
                                    .as_str()
                                    .replacen("{%/*", "{%", 1)
                                    .replacen("*/%}", "%}", 1),
                            );
                        }
                        Rule::text_in_ignored_body_sc => output.push_str(p2.as_span().as_str()),
                        _ => unreachable!("Got something weird in an ignored shortcode: {:?}", p2),
                    }
                }
            }
            Rule::EOI => (),
            _ => unreachable!("unexpected page rule: {:?}", p.as_rule()),
        }
    }

    Ok((output, shortcodes))
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_lex_rule {
        ($rule: expr, $input: expr) => {
            let res = ContentParser::parse($rule, $input);
            println!("{:?}", $input);
            println!("{:#?}", res);
            if res.is_err() {
                println!("{}", res.unwrap_err());
                panic!();
            }
            assert!(res.is_ok());
            assert_eq!(res.unwrap().last().unwrap().as_span().end(), $input.len());
        };
    }

    #[test]
    fn lex_text() {
        let inputs = vec!["Hello world", "HEllo \n world", "Hello 1 2 true false 'hey'"];
        for i in inputs {
            assert_lex_rule!(Rule::text, i);
        }
    }

    #[test]
    fn lex_inline_shortcode() {
        let inputs = vec![
            "{{ youtube() }}",
            "{{ youtube(id=1, autoplay=true, url='hey') }}",
            "{{ youtube(id=1, \nautoplay=true, url='hey', array=[]) }}",
            "{{ youtube(id=1, \nautoplay=true, url='hey', multi_aray=[[]]) }}",
        ];
        for i in inputs {
            assert_lex_rule!(Rule::inline_shortcode, i);
        }
    }

    #[test]
    fn lex_inline_ignored_shortcode() {
        let inputs = vec![
            "{{/* youtube() */}}",
            "{{/* youtube(id=1, autoplay=true, url='hey') */}}",
            "{{/* youtube(id=1, \nautoplay=true, \nurl='hey') */}}",
        ];
        for i in inputs {
            assert_lex_rule!(Rule::ignored_inline_shortcode, i);
        }
    }

    #[test]
    fn lex_shortcode_with_body() {
        let inputs = vec![
            r#"{% youtube() %}
            Some text
            {% end %}"#,
            r#"{% youtube(id=1,
            autoplay=true, url='hey') %}
            Some text
            {% end %}"#,
        ];
        for i in inputs {
            assert_lex_rule!(Rule::shortcode_with_body, i);
        }
    }

    #[test]
    fn lex_ignored_shortcode_with_body() {
        let inputs = vec![
            r#"{%/* youtube() */%}
            Some text
            {%/* end */%}"#,
            r#"{%/* youtube(id=1,
            autoplay=true, url='hey') */%}
            Some text
            {%/* end */%}"#,
        ];
        for i in inputs {
            assert_lex_rule!(Rule::ignored_shortcode_with_body, i);
        }
    }

    #[test]
    fn lex_page() {
        let inputs = vec![
            "Some text and a shortcode `{{/* youtube() */}}`",
            "{{ youtube(id=1, autoplay=true, url='hey') }}",
            "{{ youtube(id=1, \nautoplay=true, url='hey') }} that's it",
            r#"
            This is a test
            {% hello() %}
            Body {{ var }}
            {% end %}
            "#,
        ];
        for i in inputs {
            assert_lex_rule!(Rule::page, i);
        }
    }

    #[test]
    fn can_update_ranges() {
        let mut sc = Shortcode {
            name: "a".to_string(),
            args: Value::Null,
            span: 10..20,
            body: None,
            nth: 0,
            tera_name: String::new(),
        };
        // 6 -> 10 in length so +4 on both sides of the range
        sc.update_range(&(2..8), 10);
        assert_eq!(sc.span, 14..24);
        // After the shortcode so no impact
        sc.update_range(&(25..30), 30);
        assert_eq!(sc.span, 14..24);
        // +4 again
        sc.update_range(&(5..11), 10);
        assert_eq!(sc.span, 18..28);

        // buggy case from https://zola.discourse.group/t/zola-0-15-md-shortcode-stopped-working/1099/3
        let mut sc = Shortcode {
            name: "a".to_string(),
            args: Value::Null,
            span: 42..65,
            body: None,
            nth: 0,
            tera_name: String::new(),
        };
        sc.update_range(&(9..32), 3);
        assert_eq!(sc.span, 22..45);
    }

    #[test]
    fn can_extract_basic_inline_shortcode_with_args() {
        let (out, shortcodes) = parse_for_shortcodes(
            "Inline shortcode: {{ hello(string='hey', int=1, float=2.1, bool=true, array=[true, false]) }} hey",
        )
        .unwrap();
        assert_eq!(out, format!("Inline shortcode: {} hey", SHORTCODE_PLACEHOLDER));
        assert_eq!(shortcodes.len(), 1);
        assert_eq!(shortcodes[0].name, "hello");
        assert_eq!(shortcodes[0].args.as_object().unwrap().len(), 5);
        assert_eq!(shortcodes[0].args["string"], Value::String("hey".to_string()));
        assert_eq!(shortcodes[0].args["bool"], Value::Bool(true));
        assert_eq!(shortcodes[0].args["int"], to_value(1).unwrap());
        assert_eq!(shortcodes[0].args["float"], to_value(2.1).unwrap());
        assert_eq!(
            shortcodes[0].args["array"],
            Value::Array(vec![Value::Bool(true), Value::Bool(false)])
        );
        assert_eq!(shortcodes[0].span, 18..(18 + SHORTCODE_PLACEHOLDER.len()));
        assert_eq!(shortcodes[0].nth, 1);
    }

    #[test]
    fn can_unignore_ignored_inline_shortcode() {
        let (out, shortcodes) =
            parse_for_shortcodes("Hello World {{/* youtube() */}} hey").unwrap();
        assert_eq!(out, "Hello World {{ youtube() }} hey");
        assert_eq!(shortcodes.len(), 0);
    }

    #[test]
    fn can_extract_shortcode_with_body() {
        let (out, shortcodes) = parse_for_shortcodes(
            "Body shortcode\n {% quote(author='Bobby', array=[[true]]) %}DROP TABLES;{% end %} \n hey",
        )
        .unwrap();
        assert_eq!(out, format!("Body shortcode\n {} \n hey", SHORTCODE_PLACEHOLDER));
        assert_eq!(shortcodes.len(), 1);
        assert_eq!(shortcodes[0].name, "quote");
        assert_eq!(shortcodes[0].args.as_object().unwrap().len(), 2);
        assert_eq!(shortcodes[0].args["author"], Value::String("Bobby".to_string()));
        assert_eq!(
            shortcodes[0].args["array"],
            Value::Array(vec![Value::Array(vec![Value::Bool(true)])])
        );
        assert_eq!(shortcodes[0].body, Some("DROP TABLES;".to_owned()));
        assert_eq!(shortcodes[0].span, 16..(16 + SHORTCODE_PLACEHOLDER.len()));
        assert_eq!(shortcodes[0].nth, 1);
    }

    #[test]
    fn can_unignore_ignored_shortcode_with_body() {
        let (out, shortcodes) =
            parse_for_shortcodes("Hello World {%/* youtube() */%} Somebody {%/* end */%} hey")
                .unwrap();
        assert_eq!(out, "Hello World {% youtube() %} Somebody {% end %} hey");
        assert_eq!(shortcodes.len(), 0);
    }

    #[test]
    fn can_extract_multiple_shortcodes_and_increment_nth() {
        let (out, shortcodes) = parse_for_shortcodes(
            "Hello World {% youtube() %} Somebody {% end %} {{ hello() }}\n {{hello()}}",
        )
        .unwrap();
        assert_eq!(
            out,
            format!(
                "Hello World {} {}\n {}",
                SHORTCODE_PLACEHOLDER, SHORTCODE_PLACEHOLDER, SHORTCODE_PLACEHOLDER
            )
        );
        assert_eq!(shortcodes.len(), 3);
        assert_eq!(shortcodes[0].nth, 1);
        assert_eq!(shortcodes[1].nth, 1);
        assert_eq!(shortcodes[2].nth, 2);
    }

    #[test]
    fn can_handle_multiple_shortcodes() {
        let (_, shortcodes) = parse_for_shortcodes(
            r#"
        {{ youtube(id="ub36ffWAqgQ") }}
        {{ youtube(id="ub36ffWAqgQ", autoplay=true) }}
        {{ vimeo(id="210073083") }}
        {{ streamable(id="c0ic") }}
        {{ gist(url="https://gist.github.com/Keats/32d26f699dcc13ebd41b") }}"#,
        )
        .unwrap();
        assert_eq!(shortcodes.len(), 5);
    }
}
