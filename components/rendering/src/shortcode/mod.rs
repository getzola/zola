#[macro_use]
mod util;

mod arg_value;
mod inner_tag;
mod parse;
mod string_literal;

use arg_value::ToJsonConvertError;
pub use parse::{fetch_shortcodes, ShortcodeContext};
use crate::transform::Transform;

use std::collections::HashMap;

#[derive(Clone, PartialEq)]
pub enum ShortcodeFileType {
    Markdown,
    HTML,
}

#[derive(Clone)]
pub struct ShortcodeDefinition {
    file_type: ShortcodeFileType,
    content: String,
}

const MAX_CALLSTACK_DEPTH: usize = 128;

#[derive(Debug, PartialEq)]
pub enum RenderError {
    VariableNotFound { complete_var: Vec<String>, specific_part: String },
    RecursiveReuseOfShortcode(String),
    MaxRecusionDepthReached,
    FloatParseError,
    TeraError,
}

impl From<ToJsonConvertError> for RenderError {
    fn from(err: ToJsonConvertError) -> Self {
        match err {
            ToJsonConvertError::VariableNotFound { complete_var, specific_part } => {
                RenderError::VariableNotFound { complete_var, specific_part }
            }
            ToJsonConvertError::FloatParseError => RenderError::FloatParseError,
        }
    }
}

impl Into<errors::Error> for RenderError {
    fn into(self) -> errors::Error {
        // TODO: Improve this conversion
        errors::Error::msg("Something went wrong whilst rendering shortcodes")
    }
}


/// Looks through a source string and will replace all shortcodes taking into account the call
/// stack, invocation_counts, and the preexisting tera_context.
fn replace_all_shortcodes(
    source: &str,
    shortcode_definitions: &HashMap<String, ShortcodeDefinition>,
    call_stack: &mut Vec<String>,
    invocation_counts: &mut HashMap<String, usize>,
    context: &tera::Context,
    filter_file_type: &ShortcodeFileType,
) -> Result<(String, Vec<Transform>), RenderError> {
    let mut content = source.to_string();

    let mut transforms = Vec::new();

    for mut ctx in fetch_shortcodes(&content).into_iter() {
        for Transform { span_start, initial_end, after_end } in transforms.iter() {
            ctx.update_on_source_insert(*span_start, *initial_end, *after_end)
                .expect("Errors here should never happen");
        }

        let ctx_span = ctx.span();
        let res = render_shortcode(
            &content,
            ctx,
            shortcode_definitions,
            call_stack,
            invocation_counts,
            context,
            filter_file_type,
        )?;

        transforms.push(Transform::new(&ctx_span, res.len()));
        content.replace_range(ctx_span, &res);
    }

    Ok((content, transforms))
}

/// Take one specific shortcode and attempt to turn it into its resulting replacement string
fn render_shortcode(
    source: &str,
    context: ShortcodeContext,
    shortcode_definitions: &HashMap<String, ShortcodeDefinition>,
    call_stack: &mut Vec<String>,
    invocation_counts: &mut HashMap<String, usize>,
    tera_context: &tera::Context,
    filter_file_type: &ShortcodeFileType,
    //tera: &tera::Tera,
) -> Result<String, RenderError> {
    // Throw an error if the call stack already contains the current shortcode
    if call_stack.contains(context.name()) {
        return Err(RenderError::RecursiveReuseOfShortcode(context.name().to_owned()));
    }

    // Throw an error if the call stack goes over the max limit
    if call_stack.len() > MAX_CALLSTACK_DEPTH {
        return Err(RenderError::MaxRecusionDepthReached);
    }

    let body_content = context.body_content(source);

    let mut new_context = tera::Context::new();

    for (key, value) in context.args().iter() {
        new_context.insert(key, &value.to_tera(&new_context)?);
    }
    if let Some(ref body_content) = body_content {
        // Trimming right to avoid most shortcodes with bodies ending up with a HTML new line
        new_context.insert("body", body_content.trim_end());
    }

    // We don't have to take into account the call stack, since we know for sure that it will not
    // contain this shortcode again.
    let invocation_count = invocation_counts.get(context.name()).unwrap_or(&1).clone();
    new_context.insert("nth", &invocation_count);
    invocation_counts.insert(context.name().to_string(), invocation_count + 1);

    new_context.extend(tera_context.clone());

    let shortcode_def = shortcode_definitions.get(context.name());
    Ok(match shortcode_def {
        // Filter out where the shortcode definition is unknown and where the shortcode definition
        // is HTML.
        None => source[context.span()].to_string(),
        Some(ShortcodeDefinition { content, ref file_type, .. }) => {
            // Make sure we are actually matching either a MD or HTML shortcode and not both at the
            // same time
            if file_type != filter_file_type {
                return Ok(source[context.span()].to_string());
            }

            // Add the current shortcode to the call stack.
            call_stack.push(context.name().to_string());

            // NOTE: We can ignore transforms here since the overarching transform will capture the
            // idea anyway.
            let (content, _) = replace_all_shortcodes(
                &content,
                shortcode_definitions,
                call_stack,
                invocation_counts,
                &tera_context,
                filter_file_type,
            )?;

            // Remove the current function again from the call start.
            call_stack.pop();

            // TODO: Properly add tera errors here
            // TODO: Change this to use builtins
            tera::Tera::one_off(&content, &new_context, false)
                .map_err(|_| RenderError::TeraError)?
        }
    })
}

/// Inserts shortcodes of file type `filter_file_type` (recursively) into a source string
pub fn insert_shortcodes(
    source: &str,
    shortcode_definitions: &HashMap<String, ShortcodeDefinition>,
    filter_file_type: ShortcodeFileType,
    tera_context: &tera::Context,
) -> Result<(String, Vec<Transform>), RenderError> {
    let mut invocation_counts = HashMap::new();
    let mut call_stack = Vec::new();

    replace_all_shortcodes(
        source,
        shortcode_definitions,
        &mut call_stack,
        &mut invocation_counts,
        tera_context,
        &filter_file_type,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ShortcodeFileType::*;

    impl ShortcodeDefinition {
        fn new(file_type: ShortcodeFileType, content: &str) -> ShortcodeDefinition {
            let content = content.to_string();

            ShortcodeDefinition { file_type, content }
        }
    }

    macro_rules! assert_render_md_shortcode {
        ($source:expr, $context:expr, $defs:expr$(, [$($call_stack:expr),*])?$(,)? => $res:expr) => {
            let context = tera::Context::new();
            let mut invocation_counts = HashMap::new();
            let mut call_stack = vec![ $($($call_stack),*)? ];
            assert_eq!(
                render_shortcode(
                    $source,
                    $context,
                    &$defs,
                    &mut call_stack,
                    &mut invocation_counts,
                    &context,
                    &ShortcodeFileType::Markdown,
                ),
                $res
            );
        }
    }

    macro_rules! shortcode_defs {
        ($($name:expr => $ty:expr, $content:expr),*$(,)?) => {{
            let mut map = HashMap::new();
            $(
                map.insert($name.to_string(), ShortcodeDefinition::new($ty, $content));
            )*
            map
        }}
    }

    #[test]
    fn render_md_shortcode() {
        let shortcodes = shortcode_defs![
            "a" => Markdown, "wow",
            "calls_b" => Markdown, "Prefix {{ b() }}",
            "b" => Markdown, "Internal of b",
            "one_two" => Markdown, "{{ one() }}",
            "one" => Markdown, "{{ one_two() }}",
        ];

        assert_render_md_shortcode!(
            "abc {{ a() }}", ShortcodeContext::new("a", vec![], 4..13, None), shortcodes =>
                Ok("wow".to_string())
        );
        assert_render_md_shortcode!(
             "abc {{ calls_b() }}", ShortcodeContext::new("calls_b", vec![], 4..19, None), shortcodes =>
                Ok("Prefix Internal of b".to_string())
        );
        assert_render_md_shortcode!(
            "abc {{ n() }}", ShortcodeContext::new("a", vec![], 4..13, None), shortcodes =>
                Ok("wow".to_string())
        );
        assert_render_md_shortcode!(
            "{{ one_two() }}", ShortcodeContext::new("one_two", vec![], 0..15, None), shortcodes =>
                Err(RenderError::RecursiveReuseOfShortcode("one_two".to_string()))
        );
    }

    #[test]
    fn insert_md_shortcodes() {
        let shortcodes = shortcode_defs![
            "a" => Markdown, "wow",
            "calls_b" => Markdown, "Prefix {{ b() }}",
            "b" => Markdown, "Internal of b",
            "one_two" => Markdown, "{{ one() }}",
            "one" => Markdown, "{{ one_two() }}",
            "inv" => Markdown, "{{ nth }}",
            "html" => HTML, "{{ wow }}",
            "bodied" => Markdown, "much {{ body }}",
        ];
        
        let tera_context = tera::Context::new();

        assert_eq!(
            insert_shortcodes(
                "{{ inv() }}{{ inv() }}",
                &shortcodes,
                ShortcodeFileType::Markdown,
                &tera_context
            ).unwrap().0,
            "12".to_string()
        );
        assert_eq!(
            insert_shortcodes(
                "{% bodied() %}{{ a() }}{% end %}",
                &shortcodes,
                ShortcodeFileType::Markdown,
                &tera_context
            ).unwrap().0,
            "much wow".to_string()
        );
        assert_eq!(
            insert_shortcodes(
                "Hello {{ html() }}!",
                &shortcodes,
                ShortcodeFileType::Markdown,
                &tera_context
            ).unwrap().0,
            "Hello {{ html() }}!".to_string()
        );
        assert_eq!(
            insert_shortcodes(
                "{% bodied() %}{{ a() }} {{ html() }}{% end %}",
                &shortcodes,
                ShortcodeFileType::Markdown,
                &tera_context
            ).unwrap().0,
            "much wow {{ html() }}".to_string()
        );
    }

    #[test]
    fn insert_html_shortcodes() {
        let shortcodes = shortcode_defs![
            "a" => HTML, "wow",
            "calls_b" => HTML, "Prefix {{ b() }}",
            "b" => HTML, "Internal of b",
            "one_two" => HTML, "{{ one() }}",
            "one" => HTML, "{{ one_two() }}",
            "inv" => HTML, "{{ nth }}",
            "html" => Markdown, "{{ wow }}",
            "bodied" => HTML, "much {{ body }}",
        ];

        let tera_context = tera::Context::new();

        assert_eq!(
            insert_shortcodes(
                "{{ inv() }}{{ inv() }}",
                &shortcodes,
                ShortcodeFileType::HTML,
                &tera_context
            ).unwrap().0,
            "12".to_string()
        );
        assert_eq!(
            insert_shortcodes(
                "{% bodied() %}{{ a() }}{% end %}",
                &shortcodes,
                ShortcodeFileType::HTML,
                &tera_context
            ).unwrap().0,
            "much wow".to_string()
        );
        assert_eq!(
            insert_shortcodes(
                "Hello {{ html() }}!",
                &shortcodes,
                ShortcodeFileType::HTML,
                &tera_context
            ).unwrap().0,
            "Hello {{ html() }}!".to_string()
        );
        assert_eq!(
            insert_shortcodes(
                "{% bodied() %}{{ a() }} {{ html() }}{% end %}",
                &shortcodes,
                ShortcodeFileType::HTML,
                &tera_context
            ).unwrap().0,
            "much wow {{ html() }}".to_string()
        );
    }
}

//
//
// pub struct ShortCodeContext {
//     shortcode_type: ShortCodeType,
//     block: Span,
//     end_block: Option<Span>,
//     name: String,
//     arguments: HashMap<String, String>,
// }
//
// fn replace_string_markers(input: &str) -> String {
//     match input.chars().next().unwrap() {
//         '"' => input.replace('"', ""),
//         '\'' => input.replace('\'', ""),
//         '`' => input.replace('`', ""),
//         _ => unreachable!("How did you even get there"),
//     }
// }
//
// fn parse_literal(pair: Pair<Rule>) -> Value {
//     let mut val = None;
//     for p in pair.into_inner() {
//         match p.as_rule() {
//             Rule::boolean => match p.as_str() {
//                 "true" => val = Some(Value::Bool(true)),
//                 "false" => val = Some(Value::Bool(false)),
//                 _ => unreachable!(),
//             },
//             Rule::string => val = Some(Value::String(replace_string_markers(p.as_str()))),
//             Rule::float => {
//                 val = Some(to_value(p.as_str().parse::<f64>().unwrap()).unwrap());
//             }
//             Rule::int => {
//                 val = Some(to_value(p.as_str().parse::<i64>().unwrap()).unwrap());
//             }
//             _ => unreachable!("Unknown literal: {:?}", p),
//         };
//     }
//
//     val.unwrap()
// }
//
// /// Returns (shortcode_name, kwargs)
// fn parse_shortcode_call(pair: Pair<Rule>) -> (String, Map<String, Value>) {
//     let mut name = None;
//     let mut args = Map::new();
//
//     for p in pair.into_inner() {
//         match p.as_rule() {
//             Rule::ident => {
//                 name = Some(p.as_span().as_str().to_string());
//             }
//             Rule::kwarg => {
//                 let mut arg_name = None;
//                 let mut arg_val = None;
//                 for p2 in p.into_inner() {
//                     match p2.as_rule() {
//                         Rule::ident => {
//                             arg_name = Some(p2.as_span().as_str().to_string());
//                         }
//                         Rule::literal => {
//                             arg_val = Some(parse_literal(p2));
//                         }
//                         Rule::array => {
//                             let mut vals = vec![];
//                             for p3 in p2.into_inner() {
//                                 match p3.as_rule() {
//                                     Rule::literal => vals.push(parse_literal(p3)),
//                                     _ => unreachable!(
//                                         "Got something other than literal in an array: {:?}",
//                                         p3
//                                     ),
//                                 }
//                             }
//                             arg_val = Some(Value::Array(vals));
//                         }
//                         _ => unreachable!("Got something unexpected in a kwarg: {:?}", p2),
//                     }
//                 }
//
//                 args.insert(arg_name.unwrap(), arg_val.unwrap());
//             }
//             _ => unreachable!("Got something unexpected in a shortcode: {:?}", p),
//         }
//     }
//     (name.unwrap(), args)
// }
//
// fn render_shortcode(
//     name: &str,
//     args: &Map<String, Value>,
//     context: &RenderContext,
//     invocation_count: u32,
//     body: Option<&str>,
// ) -> Result<String> {
//     let mut tera_context = Context::new();
//     for (key, value) in args.iter() {
//         tera_context.insert(key, value);
//     }
//     if let Some(ref b) = body {
//         // Trimming right to avoid most shortcodes with bodies ending up with a HTML new line
//         tera_context.insert("body", b.trim_end());
//     }
//     tera_context.insert("nth", &invocation_count);
//     tera_context.extend(context.tera_context.clone());
//
//     let mut template_name = format!("shortcodes/{}.md", name);
//     if !context.tera.templates.contains_key(&template_name) {
//         template_name = format!("shortcodes/{}.html", name);
//     }
//
//     let res = utils::templates::render_template(&template_name, &context.tera, tera_context, &None)
//         .map_err(|e| Error::chain(format!("Failed to render {} shortcode", name), e))?;
//
//     let res = OUTER_NEWLINE_RE.replace_all(&res, "");
//
//     // A blank line will cause the markdown parser to think we're out of HTML and start looking
//     // at indentation, making the output a code block. To avoid this, newlines are replaced with
//     // "<!--\n-->" at this stage, which will be undone after markdown rendering in lib.rs. Since
//     // that is an HTML comment, it shouldn't be rendered anyway. and not cause problems unless
//     // someone wants to include that comment in their content. This behaviour is unwanted in when
//     // rendering markdown shortcodes.
//     if template_name.ends_with(".html") {
//         Ok(format!("<pre data-shortcode>{}</pre>", res))
//     } else {
//         Ok(res.to_string())
//     }
// }
//
// pub fn render_shortcodes(content: &str, context: &RenderContext) -> Result<String> {
//     let mut res = String::with_capacity(content.len());
//     let mut invocation_map: HashMap<String, u32> = HashMap::new();
//     let mut get_invocation_count = |name: &str| {
//         let invocation_number = invocation_map.entry(String::from(name)).or_insert(0);
//         *invocation_number += 1;
//         *invocation_number
//     };
//
//     let mut pairs = match ContentParser::parse(Rule::page, content) {
//         Ok(p) => p,
//         Err(e) => {
//             let fancy_e = e.renamed_rules(|rule| match *rule {
//                 Rule::int => "an integer".to_string(),
//                 Rule::float => "a float".to_string(),
//                 Rule::string => "a string".to_string(),
//                 Rule::literal => "a literal (int, float, string, bool)".to_string(),
//                 Rule::array => "an array".to_string(),
//                 Rule::kwarg => "a keyword argument".to_string(),
//                 Rule::ident => "an identifier".to_string(),
//                 Rule::inline_shortcode => "an inline shortcode".to_string(),
//                 Rule::ignored_inline_shortcode => "an ignored inline shortcode".to_string(),
//                 Rule::sc_body_start => "the start of a shortcode".to_string(),
//                 Rule::ignored_sc_body_start => "the start of an ignored shortcode".to_string(),
//                 Rule::text => "some text".to_string(),
//                 Rule::EOI => "end of input".to_string(),
//                 Rule::double_quoted_string => "double quoted string".to_string(),
//                 Rule::single_quoted_string => "single quoted string".to_string(),
//                 Rule::backquoted_quoted_string => "backquoted quoted string".to_string(),
//                 Rule::boolean => "a boolean (true, false)".to_string(),
//                 Rule::all_chars => "a alphanumerical character".to_string(),
//                 Rule::kwargs => "a list of keyword arguments".to_string(),
//                 Rule::sc_def => "a shortcode definition".to_string(),
//                 Rule::shortcode_with_body => "a shortcode with body".to_string(),
//                 Rule::ignored_shortcode_with_body => "an ignored shortcode with body".to_string(),
//                 Rule::sc_body_end => "{% end %}".to_string(),
//                 Rule::ignored_sc_body_end => "{%/* end */%}".to_string(),
//                 Rule::text_in_body_sc => "text in a shortcode body".to_string(),
//                 Rule::text_in_ignored_body_sc => "text in an ignored shortcode body".to_string(),
//                 Rule::content => "some content".to_string(),
//                 Rule::page => "a page".to_string(),
//                 Rule::WHITESPACE => "whitespace".to_string(),
//             });
//             bail!("{}", fancy_e);
//         }
//     };
//
//     // We have at least a `page` pair
//     for p in pairs.next().unwrap().into_inner() {
//         match p.as_rule() {
//             Rule::text => res.push_str(p.as_span().as_str()),
//             Rule::inline_shortcode => {
//                 let (name, args) = parse_shortcode_call(p);
//                 res.push_str(&render_shortcode(
//                     &name,
//                     &args,
//                     context,
//                     get_invocation_count(&name),
//                     None,
//                 )?);
//             }
//             Rule::shortcode_with_body => {
//                 let mut inner = p.into_inner();
//                 // 3 items in inner: call, body, end
//                 // we don't care about the closing tag
//                 let (name, args) = parse_shortcode_call(inner.next().unwrap());
//                 let body = inner.next().unwrap().as_span().as_str();
//                 res.push_str(&render_shortcode(
//                     &name,
//                     &args,
//                     context,
//                     get_invocation_count(&name),
//                     Some(body),
//                 )?);
//             }
//             Rule::ignored_inline_shortcode => {
//                 res.push_str(
//                     &p.as_span().as_str().replacen("{{/*", "{{", 1).replacen("*/}}", "}}", 1),
//                 );
//             }
//             Rule::ignored_shortcode_with_body => {
//                 for p2 in p.into_inner() {
//                     match p2.as_rule() {
//                         Rule::ignored_sc_body_start | Rule::ignored_sc_body_end => {
//                             res.push_str(
//                                 &p2.as_span()
//                                     .as_str()
//                                     .replacen("{%/*", "{%", 1)
//                                     .replacen("*/%}", "%}", 1),
//                             );
//                         }
//                         Rule::text_in_ignored_body_sc => res.push_str(p2.as_span().as_str()),
//                         _ => unreachable!("Got something weird in an ignored shortcode: {:?}", p2),
//                     }
//                 }
//             }
//             Rule::EOI => (),
//             _ => unreachable!("unexpected page rule: {:?}", p.as_rule()),
//         }
//     }
//
//     Ok(res)
// }
//
// #[cfg(test)]
// mod tests {
//     use std::collections::HashMap;
//
//     use super::*;
//     use config::Config;
//     use front_matter::InsertAnchor;
//     use tera::Tera;
//
//     macro_rules! assert_lex_rule {
//         ($rule: expr, $input: expr) => {
//             let res = ContentParser::parse($rule, $input);
//             println!("{:?}", $input);
//             println!("{:#?}", res);
//             if res.is_err() {
//                 println!("{}", res.unwrap_err());
//                 panic!();
//             }
//             assert!(res.is_ok());
//             assert_eq!(res.unwrap().last().unwrap().as_span().end(), $input.len());
//         };
//     }
//
//     fn render_shortcodes(code: &str, tera: &Tera) -> String {
//         let config = Config::default_for_test();
//         let permalinks = HashMap::new();
//         let context = RenderContext::new(
//             &tera,
//             &config,
//             &config.default_language,
//             "",
//             &permalinks,
//             InsertAnchor::None,
//         );
//         super::render_shortcodes(code, &context).unwrap()
//     }
//
//     #[test]
//     fn lex_text() {
//         let inputs = vec!["Hello world", "HEllo \n world", "Hello 1 2 true false 'hey'"];
//         for i in inputs {
//             assert_lex_rule!(Rule::text, i);
//         }
//     }
//
//     #[test]
//     fn lex_inline_shortcode() {
//         let inputs = vec![
//             "{{ youtube() }}",
//             "{{ youtube(id=1, autoplay=true, url='hey') }}",
//             "{{ youtube(id=1, \nautoplay=true, url='hey') }}",
//         ];
//         for i in inputs {
//             assert_lex_rule!(Rule::inline_shortcode, i);
//         }
//     }
//
//     #[test]
//     fn lex_inline_ignored_shortcode() {
//         let inputs = vec![
//             "{{/* youtube() */}}",
//             "{{/* youtube(id=1, autoplay=true, url='hey') */}}",
//             "{{/* youtube(id=1, \nautoplay=true, \nurl='hey') */}}",
//         ];
//         for i in inputs {
//             assert_lex_rule!(Rule::ignored_inline_shortcode, i);
//         }
//     }
//
//     #[test]
//     fn lex_shortcode_with_body() {
//         let inputs = vec![
//             r#"{% youtube() %}
//             Some text
//             {% end %}"#,
//             r#"{% youtube(id=1,
//             autoplay=true, url='hey') %}
//             Some text
//             {% end %}"#,
//         ];
//         for i in inputs {
//             assert_lex_rule!(Rule::shortcode_with_body, i);
//         }
//     }
//
//     #[test]
//     fn lex_ignored_shortcode_with_body() {
//         let inputs = vec![
//             r#"{%/* youtube() */%}
//             Some text
//             {%/* end */%}"#,
//             r#"{%/* youtube(id=1,
//             autoplay=true, url='hey') */%}
//             Some text
//             {%/* end */%}"#,
//         ];
//         for i in inputs {
//             assert_lex_rule!(Rule::ignored_shortcode_with_body, i);
//         }
//     }
//
//     #[test]
//     fn lex_page() {
//         let inputs = vec![
//             "Some text and a shortcode `{{/* youtube() */}}`",
//             "{{ youtube(id=1, autoplay=true, url='hey') }}",
//             "{{ youtube(id=1, \nautoplay=true, url='hey') }} that's it",
//             r#"
//             This is a test
//             {% hello() %}
//             Body {{ var }}
//             {% end %}
//             "#,
//         ];
//         for i in inputs {
//             assert_lex_rule!(Rule::page, i);
//         }
//     }
//
//     #[test]
//     fn does_nothing_with_no_shortcodes() {
//         let res = render_shortcodes("Hello World", &Tera::default());
//         assert_eq!(res, "Hello World");
//     }
//
//     #[test]
//     fn can_unignore_inline_shortcode() {
//         let res = render_shortcodes("Hello World {{/* youtube() */}}", &Tera::default());
//         assert_eq!(res, "Hello World {{ youtube() }}");
//     }
//
//     #[test]
//     fn can_unignore_shortcode_with_body() {
//         let res = render_shortcodes(
//             r#"
// Hello World
// {%/* youtube() */%}Some body {{ hello() }}{%/* end */%}"#,
//             &Tera::default(),
//         );
//         assert_eq!(res, "\nHello World\n{% youtube() %}Some body {{ hello() }}{% end %}");
//     }
//
//     // https://github.com/Keats/gutenberg/issues/383
//     #[test]
//     fn unignore_shortcode_with_body_does_not_swallow_initial_whitespace() {
//         let res = render_shortcodes(
//             r#"
// Hello World
// {%/* youtube() */%}
// Some body {{ hello() }}{%/* end */%}"#,
//             &Tera::default(),
//         );
//         assert_eq!(res, "\nHello World\n{% youtube() %}\nSome body {{ hello() }}{% end %}");
//     }
//
//     #[test]
//     fn can_parse_shortcode_arguments() {
//         let inputs = vec![
//             ("{{ youtube() }}", "youtube", Map::new()),
//             ("{{ youtube(id=1, autoplay=true, hello='salut', float=1.2) }}", "youtube", {
//                 let mut m = Map::new();
//                 m.insert("id".to_string(), to_value(1).unwrap());
//                 m.insert("autoplay".to_string(), to_value(true).unwrap());
//                 m.insert("hello".to_string(), to_value("salut").unwrap());
//                 m.insert("float".to_string(), to_value(1.2).unwrap());
//                 m
//             }),
//             ("{{ gallery(photos=['something', 'else'], fullscreen=true) }}", "gallery", {
//                 let mut m = Map::new();
//                 m.insert("photos".to_string(), to_value(["something", "else"]).unwrap());
//                 m.insert("fullscreen".to_string(), to_value(true).unwrap());
//                 m
//             }),
//         ];
//
//         for (i, n, a) in inputs {
//             let mut res = ContentParser::parse(Rule::inline_shortcode, i).unwrap();
//             let (name, args) = parse_shortcode_call(res.next().unwrap());
//             assert_eq!(name, n);
//             assert_eq!(args, a);
//         }
//     }
//
//     #[test]
//     fn can_render_inline_shortcodes() {
//         let mut tera = Tera::default();
//         tera.add_raw_template("shortcodes/youtube.html", "Hello {{id}}").unwrap();
//         let res = render_shortcodes("Inline {{ youtube(id=1) }}.", &tera);
//         assert_eq!(res, "Inline <pre data-shortcode>Hello 1</pre>.");
//     }
//
//     #[test]
//     fn can_render_shortcodes_with_body() {
//         let mut tera = Tera::default();
//         tera.add_raw_template("shortcodes/youtube.html", "{{body}}").unwrap();
//         let res = render_shortcodes("Body\n {% youtube() %}Hey!{% end %}", &tera);
//         assert_eq!(res, "Body\n <pre data-shortcode>Hey!</pre>");
//     }
//
//     // https://github.com/Keats/gutenberg/issues/462
//     #[test]
//     fn shortcodes_with_body_do_not_eat_newlines() {
//         let mut tera = Tera::default();
//         tera.add_raw_template("shortcodes/youtube.html", "{{body | safe}}").unwrap();
//         let res = render_shortcodes("Body\n {% youtube() %}\nHello \n \n\n World{% end %}", &tera);
//         assert_eq!(res, "Body\n <pre data-shortcode>Hello \n \n\n World</pre>");
//     }
//
//     #[test]
//     fn outer_newlines_removed_from_shortcodes_with_body() {
//         let mut tera = Tera::default();
//         tera.add_raw_template("shortcodes/youtube.html", "  \n  {{body}}  \n  ").unwrap();
//         let res = render_shortcodes("\n{% youtube() %}  \n  content  \n  {% end %}\n", &tera);
//         assert_eq!(res, "\n<pre data-shortcode>  content  </pre>\n");
//     }
//
//     #[test]
//     fn outer_newlines_removed_from_inline_shortcodes() {
//         let mut tera = Tera::default();
//         tera.add_raw_template("shortcodes/youtube.html", "  \n  Hello, Zola.  \n  ").unwrap();
//         let res = render_shortcodes("\n{{ youtube() }}\n", &tera);
//         assert_eq!(res, "\n<pre data-shortcode>  Hello, Zola.  </pre>\n");
//     }
//
//     #[test]
//     fn shortcodes_that_emit_markdown() {
//         let mut tera = Tera::default();
//         tera.add_raw_template(
//             "shortcodes/youtube.md",
//             "{% for i in [1,2,3] %}\n* {{ i }}\n{%- endfor %}",
//         )
//         .unwrap();
//         let res = render_shortcodes("{{ youtube() }}", &tera);
//         assert_eq!(res, "* 1\n* 2\n* 3");
//     }
// }
