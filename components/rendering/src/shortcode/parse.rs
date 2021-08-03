use super::argvalue::ArgValue;
use super::inner_tag::{InnerTag, InnerTagParseError};
use logos::Logos;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum ShortcodeFetchError {
    UnusedEndBlock,
    UnmatchingTags,
    InnerTagError(InnerTagParseError),
}

#[derive(PartialEq, Debug)]
/// Used to represent all the information present in a shortcode
pub struct Shortcode {
    name: String,
    args: HashMap<String, ArgValue>,
    body: Option<String>,
}

impl Shortcode {
    #[cfg(test)]
    fn new(name: &str, args_vec: Vec<(&str, ArgValue)>, body: Option<&str>) -> Shortcode {
        let InnerTag { name, args } = InnerTag::new(name, args_vec);
        let body = body.map(|b| b.to_string());

        Shortcode { name, args, body }
    }
}

/// Used to keep track of body items when parsing Shortcode. Since multiple can be embedded into
/// eachother. This needs to be kept track off.
struct BodiedStackItem {
    name: String,
    args: HashMap<String, ArgValue>,
    body_start: usize,
}

/// Fetch a [Vec] of all Shortcodes which are present in source string
pub fn fetch_shortcodes(source: &str) -> Result<Vec<Shortcode>, ShortcodeFetchError> {
    use ShortcodeFetchError::*;

    let mut lex = Openers::lexer(source);
    let mut shortcodes = Vec::new();

    let mut body_stack: Vec<BodiedStackItem> = Vec::new();

    // Loop until we run out of potential shortcodes
    while let Some(open_tag) = lex.next() {

        // Check if the open tag is an endblock
        if matches!(open_tag, Openers::EndBlock) {
            if let Some(BodiedStackItem { name, args, body_start }) = body_stack.pop() {
                let body = Some(source[body_start..lex.span().start].to_string());

                shortcodes.push(Shortcode { name, args, body });

                continue;
            } else {
                // No bodied tag was initiated, "{% end %}" is out of place
                return Err(UnusedEndBlock);
            }
        }

        // Parse the inside of the shortcode tag
        // TODO: Allow for context dependent variables
        let (inner_tag_lex, InnerTag { name, args }) = InnerTag::lex_parse(lex.morph())
            .map_err(|inner_tag_error| InnerTagError(inner_tag_error))?;
        let mut closing = inner_tag_lex.morph();

        if let Some(close_tag) = closing.next() {
            // Make sure that we have `{{` and `}}` or `{%` and `%}`.
            match (open_tag, close_tag) {
                (Openers::Normal, Closers::Normal) => {
                    shortcodes.push(Shortcode { name, args, body: None })
                }

                (Openers::Body, Closers::Body) => {
                    body_stack.push(BodiedStackItem { name, args, body_start: closing.span().end })
                }

                _ => {
                    // Tags don't match
                    return Err(UnmatchingTags);
                }
            }
        }

        lex = closing.morph();
    }

    Ok(shortcodes)
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
    use logos::Logos;

    use ShortcodeFetchError::*;

    #[test]
    fn no_shortcodes() {
        assert_eq!(fetch_shortcodes(""), Ok(vec![]));
        assert_eq!(fetch_shortcodes("abc"), Ok(vec![]));
    }

    #[test]
    fn unused_end_blocks() {
        assert_eq!(fetch_shortcodes("{% end %}"), Err(UnusedEndBlock));
        assert!(fetch_shortcodes("{% a() %}{% end %}").is_ok());
        assert_eq!(fetch_shortcodes("{% a() %}{% end %}{% end %}"), Err(UnusedEndBlock));
    }

    #[test]
    fn basic() {
        let test_str = r#"
# Hello World!

{{ abc(wow=true) }}

{% bodied(def="Hello!") %}The inside of this body{% end %}"#;

        assert_eq!(
            fetch_shortcodes(test_str),
            Ok(vec![
                Shortcode::new("abc", vec![("wow", ArgValue::Boolean(true))], None),
                Shortcode::new(
                    "bodied",
                    vec![("def", ArgValue::Text("Hello!".to_string()))],
                    Some("The inside of this body")
                )
            ])
        );
    }
}
