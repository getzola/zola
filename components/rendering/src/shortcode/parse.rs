//! This module contains the logic to locate shortcodes in a source string and parses them into a
//! [ShortcodeContext], which contains a lot of information about that shortcode which is going to
//! be used later on whilst inserted them.

use logos::Logos;

use std::collections::HashMap;
use std::ops::Range;

use super::arg_value::ArgValue;
use super::inner_tag::InnerTag;

use super::{ShortcodeDefinition, ShortcodeFileType};

/// Ranges have some limitations on adding and subtracting so we use usize's copy behaviour
/// to circumvent that with this function. Plus we are dealing with usizes so we cannot do easy
/// subtracting by adding negative numbers.
fn range_shift(
    range: &Range<usize>,
    translation: usize,
    do_shift_right: bool,
) -> Option<Range<usize>> {
    Some(if !do_shift_right {
        // If the subtraction is going to be bigger than the range start.
        if range.start < translation {
            return None;
        }

        (range.start - translation)..(range.end - translation)
    } else {
        (range.start + translation)..(range.end + translation)
    })
}

#[derive(Debug, PartialEq)]
/// The possible valid relationships two spans of shortcodes can have
enum RangeToPositionRelation {
    /// A position is before another shortcode
    Before,
    /// A position is within another shortcode
    Within,
    /// A position is after another shortcode
    After,
}

#[derive(PartialEq, Debug)]
/// Used to represent all the information present in a shortcode
pub struct ShortcodeContext {
    name: String,
    args: HashMap<String, ArgValue>,
    span: Range<usize>,
    body: Option<String>,
    file_type: ShortcodeFileType,
    tera_name: String,
}

impl ShortcodeContext {
    #[cfg(test)]
    pub fn new(
        name: &str,
        args_vec: Vec<(&str, ArgValue)>,
        span: Range<usize>,
        body: Option<&str>,
        file_type: ShortcodeFileType,
        tera_name: &str,
    ) -> ShortcodeContext {
        let InnerTag { name, args } = InnerTag::new(name, args_vec);
        let body = body.map(|b| b.to_string());
        let tera_name = tera_name.to_string();

        ShortcodeContext { name, args, span, body, file_type, tera_name }
    }

    /// Get the name of the shortcode
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Get the args of the shortcode
    pub fn args(&self) -> &HashMap<String, ArgValue> {
        &self.args
    }

    /// Get the body content of the shortcode using a source string
    pub fn body(&self) -> Option<&String> {
        self.body.as_ref()
    }

    /// Returns the span of the shortcode within source string
    pub fn span(&self) -> &Range<usize> {
        &self.span
    }

    /// Returns the file type of the shortcode context
    pub fn file_type(&self) -> &ShortcodeFileType {
        &self.file_type
    }

    /// Returns the content of the definition of the shortcode
    pub fn tera_name(&self) -> &String {
        &self.tera_name
    }

    /// Translates/Moves the span by `translation` either to the left or the right depending on
    /// `do_shift_right`.
    fn shift_span(&mut self, translation: usize, do_shift_right: bool) {
        // TODO: Look at removing this unwrap
        self.span = range_shift(&self.span, translation, do_shift_right).unwrap();
    }

    /// Gets the range relation between a `position` and the span of the current shortcode.
    fn get_range_relation(&self, position: usize) -> RangeToPositionRelation {
        match (position < self.span.start, position >= self.span.end) {
            (false, false) => RangeToPositionRelation::Within,
            // (true, true) should be impossible since a start <= end
            (true, _) => RangeToPositionRelation::Before,
            (_, true) => RangeToPositionRelation::After,
        }
    }

    /// Update all the spans when the source string is being altered. If the position is within the
    /// span the translation is ignored.
    pub fn update_on_source_insert(
        &mut self,
        position: usize,
        original_length: usize,
        new_length: usize,
    ) {
        let delta = if original_length < new_length {
            new_length - original_length
        } else {
            original_length - new_length
        };

        match self.get_range_relation(position) {
            RangeToPositionRelation::Before => {
                self.span = range_shift(&self.span, delta, original_length < new_length).unwrap()
            }
            RangeToPositionRelation::After | RangeToPositionRelation::Within => {}
        }
    }
}

const SHORTCODE_PLACEHOLDER: &str = "{{SC()}}";

/// Fetch a [Vec] of all Shortcodes which are present in source string
///
/// Will put the shortcodes which are contained within the body of another shortcode before the
/// shortcode they are contained in. This is very important.
pub fn fetch_shortcodes(
    source: &str,
    definitions: &HashMap<String, ShortcodeDefinition>,
) -> (String, Vec<ShortcodeContext>) {
    /// Used to keep track of body items when parsing Shortcode. Since multiple can be embedded into
    /// eachother. This needs to be kept track off.
    struct BodiedStackItem {
        name: String,
        args: HashMap<String, ArgValue>,
        openblock_span: Range<usize>,
        body_start: usize,
        file_type: ShortcodeFileType,
        tera_name: String,
    }

    let mut lex = Openers::lexer(source);
    let mut shortcodes = Vec::new();

    let mut current_body = None;

    let mut output_str = String::with_capacity(source.len());
    let mut last_lex_end = 0;

    // Loop until we run out of potential shortcodes
    while let Some(open_tag) = lex.next() {
        // Check if the open tag is an endblock
        if matches!(open_tag, Openers::EndBlock) {
            // Check whether a bodied shortcode has already been located
            if let Some(BodiedStackItem {
                name,
                args,
                openblock_span,
                body_start,
                file_type,
                tera_name,
            }) = current_body.take()
            {
                let body = Some(String::from(source[body_start..lex.span().start].trim()));

                shortcodes.push(ShortcodeContext {
                    name,
                    args,
                    span: output_str.len() - openblock_span.len()..output_str.len(),
                    body,
                    file_type,
                    tera_name,
                });

                last_lex_end = lex.span().end;
            }

            continue;
        }

        // Skip over all shortcodes contained within bodies
        if current_body.is_some() {
            continue;
        }

        output_str.push_str(&source[last_lex_end..lex.span().start]);
        last_lex_end = lex.span().start;

        // Parse the inside of the shortcode tag
        // TODO: Remove this clone()
        if let Ok((inner_tag_lex, InnerTag { name, args })) =
            InnerTag::lex_parse(lex.clone().morph())
        {
            let mut closing = inner_tag_lex.morph();

            if let Some(ShortcodeDefinition { file_type, tera_name }) =
                definitions.get(&name)
            {
                if let Some(close_tag) = closing.next() {
                    let openblock_span =
                        output_str.len()..(output_str.len() + SHORTCODE_PLACEHOLDER.len());

                    // Make sure that we have `{{` and `}}` or `{%` and `%}`.
                    match (open_tag, close_tag) {
                        (Openers::Normal, Closers::Normal) => {
                            output_str.push_str(SHORTCODE_PLACEHOLDER);
                            last_lex_end = closing.span().end;

                            shortcodes.push(ShortcodeContext {
                                name,
                                args,
                                span: openblock_span,
                                body: None,
                                file_type: file_type.clone(),
                                tera_name: tera_name.clone(),
                            });
                        }

                        (Openers::Body, Closers::Body) => {
                            output_str.push_str(SHORTCODE_PLACEHOLDER);
                            last_lex_end = closing.span().end;

                            current_body = Some(BodiedStackItem {
                                name,
                                args,
                                openblock_span,
                                body_start: closing.span().end,
                                file_type: file_type.clone(),
                                tera_name: tera_name.clone(),
                            });
                        }

                        _ => {
                            // Tags don't match
                            continue;
                        }
                    }
                }
            }

            lex = closing.morph();
        }
    }

    // Push last chunk
    output_str.push_str(&source[last_lex_end..]);

    (output_str, shortcodes)
}

#[derive(Debug, PartialEq, Clone, Logos)]
/// Tokens used initial parsing of source strings
enum Openers {
    #[regex(r"([{]%)([ \t\n\f]*)[eE][nN][dD]([ \t\n\f]*)(%[}])")]
    /// The token used to end a bodied shortcode (`{% end %}` with arbitrary whitespace and
    /// capitalization)
    EndBlock,

    #[regex(r"[{]%[ \t\n\f]*")]
    /// The token used to open a bodied shortcode (`{%`)
    Body,

    #[regex(r"[{][{][ \t\n\f]*")]
    /// The token used to open a normal shortcode `{{`)
    Normal,

    #[error]
    #[regex(r"[^{]+", logos::skip)]
    Error,
}

#[derive(Debug, PartialEq, Logos)]
/// Tokens used for parsing of source strings after the [InnerTag] has been established
enum Closers {
    #[regex(r"[ \t\n\f]*%[}]")]
    /// The token used to close a bodied shortcode (`%}`)
    Body,

    #[regex(r"[ \t\n\f]*[}][}]")]
    /// The token used to close a normal shortcode (`}}`)
    Normal,

    #[error]
    #[regex(r"[^%}]+", logos::skip)]
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn update_spans() {
        let mut ctx =
            ShortcodeContext::new("a", Vec::new(), 10..20, None, ShortcodeFileType::Markdown, "");
        ctx.update_on_source_insert(2, 8, 10);
        assert_eq!(ctx.span().clone(), 12..22);
        ctx.update_on_source_insert(24, 30, 30);
        assert_eq!(ctx.span().clone(), 12..22);
        ctx.update_on_source_insert(5, 11, 6);
        assert_eq!(ctx.span().clone(), 7..17);
    }

    #[test]
    fn no_shortcodes() {
        let shortcode_definitions = shortcode_defs!(
            "abc" => ShortcodeFileType::Markdown, "xyz"
        );

        assert_eq!(fetch_shortcodes("", &shortcode_definitions), (String::from(""), vec![]));
        assert_eq!(fetch_shortcodes("abc", &shortcode_definitions), (String::from("abc"), vec![]));
        assert_eq!(
            fetch_shortcodes("{{ abc }}", &shortcode_definitions),
            (String::from("{{ abc }}"), vec![])
        );
        assert_eq!(
            fetch_shortcodes("{{ abc() %}", &shortcode_definitions),
            (String::from("{{ abc() %}"), vec![])
        );
    }

    #[test]
    fn basic() {
        let shortcode_definitions = shortcode_defs!(
            "abc" => ShortcodeFileType::Markdown, "xyz",
            "bodied" => ShortcodeFileType::Markdown, "{{ body }}"
        );

        let test_str = r#"
# Hello World!

{{ abc(wow=true) }}

{% bodied(def="Hello!") %}The inside of this body{% end %}"#;

        let fst_start = "\n# Hello World!\n\n".len();
        let fst_end = fst_start + SHORTCODE_PLACEHOLDER.len();
        let snd_start = fst_end + 2;
        let snd_end = snd_start + SHORTCODE_PLACEHOLDER.len();

        assert_eq!(
            fetch_shortcodes(test_str, &shortcode_definitions),
            (
                format!(
                    r#"
# Hello World!

{0}

{0}"#,
                    SHORTCODE_PLACEHOLDER
                ),
                vec![
                    ShortcodeContext::new(
                        "abc",
                        vec![("wow", ArgValue::Boolean(true))],
                        fst_start..fst_end,
                        None,
                        ShortcodeFileType::Markdown,
                        "xyz"
                    ),
                    ShortcodeContext::new(
                        "bodied",
                        vec![("def", ArgValue::Text("Hello!".to_string()))],
                        snd_start..snd_end,
                        Some("The inside of this body"),
                        ShortcodeFileType::Markdown,
                        "{{ body }}"
                    )
                ]
            )
        );
    }

    #[test]
    fn sequential_bodies() {
        let shortcode_definitions = shortcode_defs!(
            "a" => ShortcodeFileType::Markdown, "xyz"
        );

        let test_str = "{% a() %}First body!{% end %}{% a() %}Second body!{% end %}";
        let end_open_a = "{% a() %}".len();

        assert_eq!(
            fetch_shortcodes(test_str, &shortcode_definitions),
            (
                format!("{0}{0}", SHORTCODE_PLACEHOLDER),
                vec![
                    ShortcodeContext::new(
                        "a",
                        vec![],
                        0..SHORTCODE_PLACEHOLDER.len(),
                        Some("First body!"),
                        ShortcodeFileType::Markdown,
                        "xyz"
                    ),
                    ShortcodeContext::new(
                        "a",
                        vec![],
                        SHORTCODE_PLACEHOLDER.len()..(2 * SHORTCODE_PLACEHOLDER.len()),
                        Some("Second body!"),
                        ShortcodeFileType::Markdown,
                        "xyz"
                    )
                ]
            )
        );
    }
}
