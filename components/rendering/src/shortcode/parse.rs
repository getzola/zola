//! This module contains the logic to locate shortcodes in a source string and parses them into a
//! [ShortcodeContext], which contains a lot of information about that shortcode which is going to
//! be used later on whilst inserted them.

use logos::Logos;

use std::collections::HashMap;
use std::ops::Range;

use super::arg_value::ArgValue;
use super::inner_tag::InnerTag;
use super::range_relation::RangeRelation;

// Ranges have some limitations on adding and subtracting so we use usize's copy behaviour
// to circumvent that with this macro. Plus we are dealing with usizes so we cannot do easy
// subtracting by adding negative numbers.
macro_rules! range_shift {
    ($range:expr, $translation:expr, $do_shift_right:expr) => {{
        if !$do_shift_right {
            // These debugs are in place to check whether we aren't overshifting the range
            debug_assert!($range.start >= $translation);

            ($range.start - $translation)..($range.end - $translation)
        } else {
            ($range.start + $translation)..($range.end + $translation)
        }
    }};
}

#[derive(Debug, PartialEq)]
pub struct BodyInfo {
    content_span: Range<usize>,
    endblock_span: Range<usize>,
}

#[derive(Debug, PartialEq)]
/// The possible valid relationships two spans of shortcodes can have
enum RangeToShortcodeRelation {
    /// A shortcode is before another shortcode
    Before,
    /// A shortcode is within another shortcode
    InBody,
    /// A shortcode is after another shortcode
    After,
}

#[derive(Debug, PartialEq)]
/// An invalid state relating to the the relationship between the span of one shortcode and another
pub struct RangeRelationInvalidState {
    /// The relation between the `span` of one shortcode and the openblock span of another
    relation_openblock: RangeRelation,
    /// The relation between the `span` of one shortcode and the end_block span of another
    relation_endblock: Option<RangeRelation>,
}

#[derive(PartialEq, Debug)]
/// Used to represent all the information present in a shortcode
pub struct ShortcodeContext {
    name: String,
    args: HashMap<String, ArgValue>,
    openblock_span: Range<usize>,
    body: Option<BodyInfo>,
}

impl ShortcodeContext {
    #[cfg(test)]
    pub fn new(
        name: &str,
        args_vec: Vec<(&str, ArgValue)>,
        openblock_span: Range<usize>,
        body: Option<BodyInfo>,
    ) -> ShortcodeContext {
        let InnerTag { name, args } = InnerTag::new(name, args_vec);
        ShortcodeContext { name, args, openblock_span, body }
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
    pub fn body_content<'a>(&self, source_str: &'a str) -> Option<&'a str> {
        self.body
            .as_ref()
            .map(|BodyInfo { content_span, .. }| &source_str[content_span.clone()])
    }

    /// Returns the span of the shortcode within source string
    pub fn span(&self) -> Range<usize> {
        self.openblock_span.start..match &self.body {
            None => self.openblock_span.end,
            Some(BodyInfo { endblock_span, .. }) => endblock_span.end,
        }
    }

    /// Translates/Moves the span by `translation` either to the left or the right depending on
    /// `do_shift_right`.
    fn shift_span(&mut self, translation: usize, do_shift_right: bool) {
        self.openblock_span = range_shift!(self.openblock_span, translation, do_shift_right);

        if let Some(ref mut body) = self.body {
            body.content_span = range_shift!(body.content_span, translation, do_shift_right);
            body.endblock_span = range_shift!(body.endblock_span, translation, do_shift_right);

            // Make sure everything is still properly aligned
            debug_assert_eq!(self.openblock_span.end, body.content_span.start);
            debug_assert_eq!(body.content_span.end, body.endblock_span.start);
        }
    }

    /// Gets the range relation between a `span` of another shortcode and the span of the current
    /// shortcode.
    fn get_range_relation(
        &self,
        span: Range<usize>,
    ) -> Result<RangeToShortcodeRelation, RangeRelationInvalidState> {
        let relation_openblock = RangeRelation::new(&self.openblock_span, &span);

        if let Some(ref body) = self.body {
            let relation_endblock = RangeRelation::new(&body.endblock_span, &span);

            match (&relation_openblock, &relation_endblock) {
                (RangeRelation::Before, _) => Ok(RangeToShortcodeRelation::Before),
                (_, RangeRelation::After) => Ok(RangeToShortcodeRelation::After),

                // If the `span` falls between the openblock and the end_block, it is in the body
                // of the shortcode.
                (RangeRelation::After, RangeRelation::Before) => {
                    Ok(RangeToShortcodeRelation::InBody)
                }

                _ => Err(RangeRelationInvalidState {
                    relation_openblock,
                    relation_endblock: Some(relation_endblock),
                }),
            }
        } else {
            match relation_openblock {
                RangeRelation::Before => Ok(RangeToShortcodeRelation::Before),
                RangeRelation::After => Ok(RangeToShortcodeRelation::After),
                _ => Err(RangeRelationInvalidState { relation_openblock, relation_endblock: None }),
            }
        }
    }

    /// Update all the spans when the source string is being altered.
    pub fn update_on_source_insert(
        &mut self,
        start_point: usize,
        original_end: usize,
        new_end: usize,
    ) -> Result<(), RangeRelationInvalidState> {
        use std::cmp::Ordering;

        // We have to take great care of the translation direction because we using usizes and
        // those cannot become negative.
        let (translation, do_shift_right) = match new_end.cmp(&original_end) {
            // If two spans are of equal length we don't have to do anything.
            Ordering::Equal => return Ok(()),

            Ordering::Less => (original_end - new_end, false),
            Ordering::Greater => (new_end - original_end, true),
        };

        match self.get_range_relation(start_point..original_end)? {
            // If the insertion is after the current shortcode, we don't have to do
            // anything.
            RangeToShortcodeRelation::After => {}

            // If the insertion takes place before the shortcode, we shift the entire shortcode
            // span.
            RangeToShortcodeRelation::Before => {
                // Move the spans by the different between the lengths of `original_span` and
                // `new_span`.
                self.shift_span(translation, do_shift_right);
            }

            // If the insertion takes place within the body of the shortcode, we resize the body
            // content and tranlste the end block span.
            RangeToShortcodeRelation::InBody => {
                let body = self.body.as_mut().expect(
                    "If we get the `RangeToShortcodeRelation::InBody`, there should be a body",
                );

                // The content span should be able to contain the shortcode we are talking
                // about.
                debug_assert!(body.content_span.len() >= translation);

                body.content_span = body.content_span.start..if do_shift_right {
                    body.content_span.end + translation
                } else {
                    body.content_span.end - translation
                };
                body.endblock_span = range_shift!(body.endblock_span, translation, do_shift_right);
            }
        }

        Ok(())
    }
}

/// Used to keep track of body items when parsing Shortcode. Since multiple can be embedded into
/// eachother. This needs to be kept track off.
struct BodiedStackItem {
    name: String,
    args: HashMap<String, ArgValue>,
    openblock_span: Range<usize>,
    body_start: usize,
}

/// Fetch a [Vec] of all Shortcodes which are present in source string
///
/// Will put the shortcodes which are contained within the body of another shortcode before the
/// shortcode they are contained in. This is very important.
pub fn fetch_shortcodes(source: &str) -> Vec<ShortcodeContext> {
    let mut lex = Openers::lexer(source);
    let mut shortcodes = Vec::new();

    let mut body_stack: Vec<BodiedStackItem> = Vec::new();

    // Loop until we run out of potential shortcodes
    while let Some(open_tag) = lex.next() {
        // Check if the open tag is an endblock
        if matches!(open_tag, Openers::EndBlock) {
            // Check whether a bodied shortcode has already been located
            if let Some(BodiedStackItem { name, args, openblock_span, body_start }) =
                body_stack.pop()
            {
                let body = Some(BodyInfo {
                    content_span: body_start..lex.span().start,
                    endblock_span: lex.span().start..lex.span().end,
                });

                shortcodes.push(ShortcodeContext { name, args, openblock_span, body });
            }

            continue;
        }

        let openblock_start = lex.span().start;

        // Parse the inside of the shortcode tag
        // TODO: Remove this clone()
        if let Ok((inner_tag_lex, InnerTag { name, args })) =
            InnerTag::lex_parse(lex.clone().morph())
        {
            let mut closing = inner_tag_lex.morph();

            if let Some(close_tag) = closing.next() {
                let openblock_span = openblock_start..closing.span().end;

                // Make sure that we have `{{` and `}}` or `{%` and `%}`.
                match (open_tag, close_tag) {
                    (Openers::Normal, Closers::Normal) => {
                        shortcodes.push(ShortcodeContext { name, args, openblock_span, body: None })
                    }

                    (Openers::Body, Closers::Body) => body_stack.push(BodiedStackItem {
                        name,
                        args,
                        openblock_span,
                        body_start: closing.span().end,
                    }),

                    _ => {
                        // Tags don't match
                        continue;
                    }
                }
            }

            lex = closing.morph();
        }
    }

    shortcodes
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

    #[test]
    fn update_spans() {
        let mut ctx = ShortcodeContext::new("a", Vec::new(), 10..20, None);
        ctx.update_on_source_insert(2, 8, 10).unwrap(); 
        assert_eq!(ctx.span(), 12..22);
        ctx.update_on_source_insert(24, 30, 30).unwrap(); 
        assert_eq!(ctx.span(), 12..22);
        ctx.update_on_source_insert(5, 11, 6).unwrap(); 
        assert_eq!(ctx.span(), 7..17);
    }

    #[test]
    fn no_shortcodes() {
        assert_eq!(fetch_shortcodes(""), vec![]);
        assert_eq!(fetch_shortcodes("abc"), vec![]);
        assert_eq!(fetch_shortcodes("{{ abc }}"), vec![]);
        assert_eq!(fetch_shortcodes("{{ abc() %}"), vec![]);
    }

    #[test]
    fn basic() {
        let test_str = r#"
# Hello World!

{{ abc(wow=true) }}

{% bodied(def="Hello!") %}The inside of this body{% end %}"#;

        let fst_start = "\n# Hello World!\n\n".len();
        let fst_end = fst_start + "{{ abc(wow=true) }}".len();
        let snd_start = fst_end + 2;
        let snd_end = snd_start + r#"{% bodied(def="Hello!") %}"#.len();
        let body_end = snd_end + r#"The inside of this body"#.len();
        let endblock_end = test_str.len();

        assert_eq!(
            fetch_shortcodes(test_str),
            vec![
                ShortcodeContext::new(
                    "abc",
                    vec![("wow", ArgValue::Boolean(true))],
                    fst_start..fst_end,
                    None
                ),
                ShortcodeContext::new(
                    "bodied",
                    vec![("def", ArgValue::Text("Hello!".to_string()))],
                    snd_start..snd_end,
                    Some(BodyInfo {
                        content_span: snd_end..body_end,
                        endblock_span: body_end..endblock_end
                    })
                )
            ]
        );
    }

    #[test]
    fn shortcode_in_body_requirement() {
        let test_str = "{% a() %}{{ b() }}{% end %}";
        let end_open_a = "{% a() %}".len();
        let end_open_b = end_open_a + "{{ b() }}".len();

        assert_eq!(
            fetch_shortcodes(test_str),
            vec![
                ShortcodeContext::new("b", vec![], end_open_a..end_open_b, None),
                ShortcodeContext::new(
                    "a",
                    vec![],
                    0..end_open_a,
                    Some(BodyInfo {
                        content_span: end_open_a..end_open_b,
                        endblock_span: end_open_b..test_str.len()
                    })
                )
            ]
        );

        let test_str = "{% a() %}{% b() %}{{ c() }}{% end %}{% end %}";
        let end_open_a = "{% a() %}".len();
        let end_open_b = end_open_a + "{{ b() }}".len();
        let end_open_c = end_open_b + "{{ c() }}".len();
        let end_endblock_b = end_open_c + "{% end %}".len();

        assert_eq!(
            fetch_shortcodes(test_str),
            vec![
                ShortcodeContext::new("c", vec![], end_open_b..end_open_c, None),
                ShortcodeContext::new(
                    "b",
                    vec![],
                    end_open_a..end_open_b,
                    Some(BodyInfo {
                        content_span: end_open_b..end_open_c,
                        endblock_span: end_open_c..end_endblock_b
                    })
                ),
                ShortcodeContext::new(
                    "a",
                    vec![],
                    0..end_open_a,
                    Some(BodyInfo {
                        content_span: end_open_a..end_endblock_b,
                        endblock_span: end_endblock_b..test_str.len()
                    })
                )
            ]
        );
    }
}
