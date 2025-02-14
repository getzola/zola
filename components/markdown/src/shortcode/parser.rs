use std::{collections::HashMap, ops::Range};

use errors::{bail, Context as ErrorContext, Result};
use libs::tera::{to_value, Context, Map, Tera, Value};
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use utils::templates::{ShortcodeDefinition, ShortcodeFileType, ShortcodeInvocationCounter};

pub const SHORTCODE_PLACEHOLDER: &str = "@@ZOLA_SC_PLACEHOLDER@@";

#[derive(PartialEq, Debug, Eq)]
pub struct Shortcode {
    pub(crate) name: String,
    pub(crate) args: Value,
    // In practice, span.len() is always equal to SHORTCODE_PLACEHOLDER.len()
    pub(crate) span: Range<usize>,
    pub(crate) body: Option<String>,
    pub(crate) indent: String,
    pub(crate) nth: usize,
    pub(crate) inner: Vec<Shortcode>,
    // set later down the line, for quick access without needing the definitions
    pub(crate) tera_name: String,
}

impl Shortcode {
    /// Attempts to fill the `tera_name` field from the provided definitions for self and all of self.inner.
    ///
    /// This returns an error if the definitions do not have this shortcode.
    pub fn fill_tera_name(
        &mut self,
        definitions: &HashMap<String, ShortcodeDefinition>,
    ) -> Result<()> {
        if let Some(def) = definitions.get(&self.name) {
            self.tera_name = def.tera_name.clone();
        } else {
            return Err(errors::anyhow!("Found usage of a shortcode named `{}` but we do not know about. Make sure it's not a typo and that a field name `{}.{{html,md}}` exists in the `templates/shortcodes` directory.", self.name, self.name));
        }
        for inner_sc in self.inner.iter_mut() {
            inner_sc.fill_tera_name(definitions)?;
        }
        Ok(())
    }

    pub fn file_type(&self) -> ShortcodeFileType {
        if self.tera_name.ends_with("md") {
            ShortcodeFileType::Markdown
        } else {
            ShortcodeFileType::Html
        }
    }

    /// Expands all inner-shortcodes and leaves self.inner empty.
    ///
    /// This function has no effect with shortcodes without bodies.
    pub fn render_inner_shortcodes(&mut self, tera: &Tera, context: &Context) -> Result<()> {
        let Some(body) = &mut self.body else {
            return Ok(());
        };
        for inner_sc in std::mem::take(&mut self.inner).into_iter().rev() {
            // We're not considering the file_type of the inner shortcodes.
            // - HTML SC invokes HTML SC: works as expected.
            // - MD SC invokes HTML SC: MD can do inline-html, it is assumed that this is intentional.
            // - MD SC invokes MD SC: works as expected.
            // - HTML SC invokes MD SC: HTML SC's with MD bodies usually use the "markdown" filter.
            let inner_sc_span = inner_sc.span.clone();
            let inner_sc_result = inner_sc.render(tera, context)?;
            body.replace_range(inner_sc_span, &inner_sc_result);
        }
        Ok(())
    }

    pub fn render(mut self, tera: &Tera, context: &Context) -> Result<String> {
        // This function gets called under the following circumstances
        // 1. as an .md shortcode, the resulting body is inserted into the document _before_ MD -> HTML conversion
        // 2. as an .html shortcode, the result is inserted into the document _during_ MD -> HTML conversion. (The HTML
        //    is injected into cmark's AST)
        // 3. As an inner-part of a shortcode which is being flattened. The file_type is not considered.
        self.render_inner_shortcodes(tera, context)?;

        let name = self.name;
        let tpl_name = self.tera_name;
        let mut new_context = Context::from_value(self.args)?;

        if let Some(body_content) = self.body {
            // Trimming right to avoid most shortcodes with bodies ending up with a HTML new line
            new_context.insert("body", body_content.trim_end());
        }
        new_context.insert("nth", &self.nth);
        new_context.extend(context.clone());

        let rendered = utils::templates::render_template(&tpl_name, tera, new_context, &None)
            .with_context(|| format!("Failed to render {} shortcode", name))?;
        // Append the rendered text, but indenting each line after the first one as much as the line in which the shortcode was called.
        let mut res = String::with_capacity(rendered.len());
        let mut lines = rendered.split_terminator('\n');
        if let Some(first_line) = lines.next() {
            res.push_str(first_line.trim_end_matches('\r'));
            res.push('\n');
            for line in lines {
                res.push_str(&self.indent);
                res.push_str(line.trim_end_matches('\r'));
                res.push('\n');
            }
        }

        Ok(res)
    }

    /// Shifts `self.span` by `(rendered_length - sc_span.len())`
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

pub fn parse_for_shortcodes(
    content: &str,
    invocation_counter: &mut ShortcodeInvocationCounter,
) -> Result<(String, Vec<Shortcode>)> {
    let mut shortcodes = Vec::new();
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
        fn current_indent(text: &str) -> &str {
            let current_line = match text.rsplit_once('\n') {
                Some((_, line)) => line,
                None => text,
            };
            // Stop at the first character that is not considered indentation by the CommonMark spec.
            match current_line.split_once(|ch| ch != ' ' && ch != '\t') {
                Some((whitespace, _)) => whitespace,
                None => current_line,
            }
        }

        match p.as_rule() {
            Rule::text => output.push_str(p.as_span().as_str()),
            Rule::inline_shortcode => {
                let start = output.len();
                let indent = current_indent(&output).into();
                let (name, args) = parse_shortcode_call(p);
                let nth = invocation_counter.get(&name);
                shortcodes.push(Shortcode {
                    name,
                    args,
                    span: start..(start + SHORTCODE_PLACEHOLDER.len()),
                    body: None,
                    indent,
                    nth,
                    inner: Vec::new(),
                    tera_name: String::new(),
                });
                output.push_str(SHORTCODE_PLACEHOLDER);
            }
            Rule::shortcode_with_body => {
                let start = output.len();
                let indent = current_indent(&output).into();
                let mut inner = p.into_inner();
                // 3 items in inner: call, body, end
                // we don't care about the closing tag
                let (name, args) = parse_shortcode_call(inner.next().unwrap());
                let nth = invocation_counter.get(&name);
                let (body, inner) = parse_for_shortcodes(
                    inner.next().unwrap().as_span().as_str().trim(),
                    invocation_counter,
                )?;
                shortcodes.push(Shortcode {
                    name,
                    args,
                    span: start..(start + SHORTCODE_PLACEHOLDER.len()),
                    body: Some(body),
                    indent,
                    nth,
                    inner,
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
            indent: String::new(),
            nth: 0,
            inner: Vec::new(),
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
            indent: String::new(),
            nth: 0,
            inner: Vec::new(),
            tera_name: String::new(),
        };
        sc.update_range(&(9..32), 3);
        assert_eq!(sc.span, 22..45);
    }

    #[test]
    fn can_extract_basic_inline_shortcode_with_args() {
        let (out, shortcodes) = parse_for_shortcodes(
            "Inline shortcode: {{ hello(string='hey', int=1, float=2.1, bool=true, array=[true, false]) }} hey",
            &mut ShortcodeInvocationCounter::new(),
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
        let (out, shortcodes) = parse_for_shortcodes(
            "Hello World {{/* youtube() */}} hey",
            &mut ShortcodeInvocationCounter::new(),
        )
        .unwrap();
        assert_eq!(out, "Hello World {{ youtube() }} hey");
        assert_eq!(shortcodes.len(), 0);
    }

    #[test]
    fn can_extract_shortcode_with_body() {
        let (out, shortcodes) = parse_for_shortcodes(
            "Body shortcode\n {% quote(author='Bobby', array=[[true]]) %}DROP TABLES;{% end %} \n hey",
            &mut ShortcodeInvocationCounter::new()
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
        let (out, shortcodes) = parse_for_shortcodes(
            "Hello World {%/* youtube() */%} Somebody {%/* end */%} hey",
            &mut ShortcodeInvocationCounter::new(),
        )
        .unwrap();
        assert_eq!(out, "Hello World {% youtube() %} Somebody {% end %} hey");
        assert_eq!(shortcodes.len(), 0);
    }

    #[test]
    fn can_extract_multiple_shortcodes_and_increment_nth() {
        let (out, shortcodes) = parse_for_shortcodes(
            "Hello World {% youtube() %} Somebody {% end %} {{ hello() }}\n {{hello()}}",
            &mut ShortcodeInvocationCounter::new(),
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
    fn can_extract_nested_shortcode_bodies_and_increment_nth() {
        let (out, shortcodes) = parse_for_shortcodes(
            "Hello World {% i_am_gonna_nest() %} Somebody {% i_am_gonna_nest() %} Somebody {% end %} {% end %}!!",
            &mut ShortcodeInvocationCounter::new(),
        )
        .unwrap();
        assert_eq!(out, format!("Hello World {}!!", SHORTCODE_PLACEHOLDER,));
        assert_eq!(shortcodes.len(), 1);
        assert_eq!(shortcodes[0].inner.len(), 1);
        assert_eq!(shortcodes[0].nth, 1);
        assert_eq!(shortcodes[0].inner[0].nth, 2);
        assert_eq!(shortcodes[0].body, Some(format!("Somebody {SHORTCODE_PLACEHOLDER}")));
    }

    #[test]
    fn can_handle_multiple_shortcodes() {
        let (_, shortcodes) = parse_for_shortcodes(
            r#"
        {{ youtube(id="ub36ffWAqgQ_hey_") }}
        {{ youtube(id="ub36ffWAqgQ", autoplay=true) }}
        {{ vimeo(id="210073083#hello", n_a_me="hello") }}
        {{ streamable(id="c0ic", n1=true) }}
        {{ gist(url="https://gist.github.com/Keats/32d26f699dcc13ebd41b") }}"#,
            &mut ShortcodeInvocationCounter::new(),
        )
        .unwrap();
        assert_eq!(shortcodes.len(), 5);
    }
}
