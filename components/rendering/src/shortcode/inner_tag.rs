use super::argvalue::{ArgValue, ArgValueParseError};
use logos::{Lexer, Logos};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
/// The parsed inside of a shortcode tag (inside of a shortcode tag = what is between `{{` and `}}`
/// or `{%` and `%}`.
pub struct InnerTag {
    /// The name of the shortcode
    pub name: String,
    /// The arguments used when calling a shortcode
    pub args: HashMap<String, ArgValue>,
}

#[derive(Debug, PartialEq)]
/// Error Type which gets triggered with Parsing an [InnerTag]
pub enum InnerTagParseError {
    /// An [InnerTagToken] which was not expected was encountered
    UnexpectedToken(InnerTagToken),

    /// The parsing expected another Token but none were encountered
    UnexpectedEnd,

    /// The inside of tag doesn't contain parentheses after the shortcode name. Probably indicating
    /// that this is not a shortcode but instead a context dependent expression.
    NoParentheses,

    /// An argument is declared twice.
    ReuseOfArgKey(String),

    // TODO: Remove this error type
    /// Something went wrong when parsing an argument value
    ArgValueError(ArgValueParseError),
}

impl InnerTag {
    #[cfg(test)]
    /// Create a new instance of [InnerTag]. Will turn all tuples of args into entries for the
    /// HashMap.
    pub fn new(name: &str, args_vec: Vec<(&str, ArgValue)>) -> Self {
        let name = name.to_string();

        let mut args = HashMap::new();
        args_vec.into_iter().for_each(|(key, value)| {
            args.insert(key.to_string(), value);
        });

        InnerTag { name, args }
    }

    /// Input a [logos::Lexer] and it will start attempted to parse one [InnerTag]
    pub fn lex_parse<'a>(
        mut lex: Lexer<'a, InnerTagToken>,
    ) -> Result<(Lexer<'a, InnerTagToken>, InnerTag), InnerTagParseError> {
        use InnerTagParseError::*;
        use InnerTagToken::*;

        let inner_tag = InnerTag {
            // Expect shortcode name
            name: expect_token!(
                lex.next(),
                [Identifier => lex.slice().to_string()],
                UnexpectedEnd,
                |t| UnexpectedToken(t),
            )?,
            args: {
                let mut args = HashMap::new();

                // Expect open parenthese (`(`)
                expect_token!(lex.next(), [ArgsOpen => {}], NoParentheses, |t| UnexpectedToken(t))?;

                loop {
                    // Handle the `()` situation
                    if let Some(ArgsClose) = lex.clone().next() {
                        lex.next();
                        break;
                    }

                    // Expect argument name
                    let arg_key = expect_token!(
                        lex.next(),
                        [Identifier => lex.slice().to_string()],
                        UnexpectedEnd,
                        |t| UnexpectedToken(t),
                    )?;

                    if args.contains_key(&arg_key) {
                        return Err(ReuseOfArgKey(arg_key));
                    }

                    // Expect equals sign (`=`)
                    expect_token!(
                        lex.next(),
                        [Equals => {}],
                        UnexpectedEnd,
                        |t| UnexpectedToken(t)
                    )?;

                    // This tmp_lex has to be done because otherwise the lifetime of lex is too
                    // short.
                    // TODO: Make this error cast better
                    let (tmp_lex, arg_value) = ArgValue::lex_parse(lex.morph())
                        .map_err(|arg_value_err| ArgValueError(arg_value_err))?;
                    lex = tmp_lex.morph();

                    args.insert(arg_key, arg_value);

                    // This has to be an if statement since macro hygiene forbids us from
                    // putting a break in our expr.
                    if expect_token!(
                        lex.next(),
                        [Comma => false, ArgsClose => true],
                        UnexpectedEnd,
                        |t| UnexpectedToken(t),
                    )? {
                        break;
                    }
                }

                args
            },
        };

        Ok((lex, inner_tag))
    }
}

#[derive(Debug, PartialEq, Clone, Logos)]
/// Tokens used to lex [InnerTag]
pub enum InnerTagToken {
    #[regex("[a-zA-Z][a-zA-Z0-9_]*")]
    /// An identifier used for the shortcode name and argument keys
    Identifier,

    #[token("(")]
    /// The start of arguments (`(`)
    ArgsOpen,
    #[token(")")]
    /// The end of arguments (`)`)
    ArgsClose,

    #[token(",")]
    /// Token used for delimiting arguments
    Comma,
    #[token("=")]
    /// Token used for defining arguments
    Equals,

    #[error]
    #[regex(r"[ \n\t\f]+", logos::skip)]
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    use InnerTagParseError::*;

    macro_rules! lex_assert_ok {
        ($content:expr, $outcome:expr$(, $rem:expr)?$(,)?) => {
            lex_assert_eq_ok!(InnerTagToken, InnerTag, $content, $outcome$(, $rem)?)
        }
    }

    macro_rules! lex_assert_err {
        ($content:expr, $outcome:expr) => {
            lex_assert_eq_err!(InnerTagToken, InnerTag, $content, $outcome)
        };
    }

    #[test]
    fn missing_token() {
        lex_assert_err!("", UnexpectedEnd);
        lex_assert_err!("\t", UnexpectedEnd);
        lex_assert_err!("\n", UnexpectedEnd);
        lex_assert_err!(" ", UnexpectedEnd);
    }

    #[test]
    fn unexpected_token() {
        lex_assert_err!("*", UnexpectedToken(InnerTagToken::Error));
        lex_assert_err!("_", UnexpectedToken(InnerTagToken::Error));
        lex_assert_err!("-", UnexpectedToken(InnerTagToken::Error));
        lex_assert_err!("=", UnexpectedToken(InnerTagToken::Equals));
    }

    #[test]
    fn no_args() {
        lex_assert_ok!("abc()", InnerTag::new("abc", vec![]), "");
        lex_assert_ok!("abc()def", InnerTag::new("abc", vec![]), "def");
        lex_assert_ok!("wow_much_cool()", InnerTag::new("wow_much_cool", vec![]), "");
    }

    #[test]
    fn not_shortcode() {
        lex_assert_err!("abc", NoParentheses);
    }

    #[test]
    fn single_args() {
        lex_assert_ok!(
            "abc(def=123)",
            InnerTag::new("abc", vec![("def", ArgValue::Integer(123))]),
            ""
        );
        lex_assert_ok!(
            "abc(wow_much_cool123='abc\\'def')",
            InnerTag::new("abc", vec![("wow_much_cool123", ArgValue::Text("abc'def".to_string()))]),
            ""
        );
    }

    #[test]
    fn multiple_args() {
        lex_assert_ok!(
            "abc(def=123, wow_much_cool123='abc\\'def')",
            InnerTag::new(
                "abc",
                vec![
                    ("def", ArgValue::Integer(123)),
                    ("wow_much_cool123", ArgValue::Text("abc'def".to_string()))
                ]
            ),
            ""
        );
    }
}
