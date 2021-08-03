use logos::{Lexer, Logos};
use std::num::{ParseFloatError, ParseIntError};
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum ArgValue {
    /// A string originally surrounded with `'`, `"` or `\``
    Text(String),

    /// Boolean 'true' or 'false' (first letter may be captitalized)
    Boolean(bool),

    /// A floating point number
    FloatingPoint(f32),

    /// An signed integer
    Integer(i32),

    /// An array of [ArgValue]'s
    Array(Vec<ArgValue>),

    /// A variable from the context
    // TODO: Add square bracket notation
    // TODO: Add filter notation
    Var(Vec<String>),
}

/// The character used to device into an variable property
const VARIABLE_SPLITTER: char = '.';

#[derive(Debug, PartialEq)]
/// Error Type which gets triggered with Parsing an [ArgValue]
pub enum ArgValueParseError {
    /// An [ArgValueToken] which was not expected was encountered
    UnexpectedToken(ArgValueToken),

    /// The parsing expected another Token but none were encountered
    UnexpectedEnd,

    /// Something went wrong when parsing a floating point number
    FloatParseError(ParseFloatError),

    /// Something went wrong when parsing an integer
    IntegerParseError(ParseIntError),
}

/// Takes a quoted source String and a Quotation Type and will remove the surrounding quotation
/// marks and unescape the escaped quotations.
fn string_from_quoted(source: &str, quote_type: QuoteType) -> String {
    let (quote_char, quote_str, escaped_str) = quote_type.consts();

    // In debug we can make sure that we are actually removing the proper quotes
    debug_assert!(source.len() > 2);
    debug_assert_eq!(source.chars().nth(0), Some(quote_char));
    debug_assert_eq!(source.chars().nth(source.len() - 1), Some(quote_char));

    // TODO: Fix bug where '\\"' Will still be turned into '\"'
    return source[1..source.len() - 1].replace(escaped_str, quote_str).to_string();
}

impl ArgValue {
    /// Input a [logos::Lexer] and it will start attempted to parse one [ArgValue]
    pub fn lex_parse<'a>(
        mut lex: Lexer<'a, ArgValueToken>,
    ) -> Result<(Lexer<'a, ArgValueToken>, ArgValue), ArgValueParseError> {
        use ArgValue::*;
        use ArgValueParseError::*;
        use ArgValueToken::*;

        let arg_value = expect_token!(
            lex.next(),
            [
                BoolLiteral(val) => Boolean(val),
                StrLiteral(quote_type) => Text(string_from_quoted(lex.slice(), quote_type)),
                IntLiteral => {
                    Integer(i32::from_str(lex.slice()).map_err(|err| IntegerParseError(err))?)
                },
                FPLiteral => {
                    FloatingPoint(f32::from_str(lex.slice()).map_err(|err| FloatParseError(err))?)
                },
                Variable => {
                    Var(lex.slice().split(VARIABLE_SPLITTER).map(|s| s.to_string()).collect())
                },
                OpenArray => {
                    let mut items = Vec::new();

                    // Basically loop collecting ArgValue's delimited by Comma until a CloseArray
                    // token is found.
                    loop {
                        // Handle the `[]` situation
                        if let Some(CloseArray) = lex.clone().next() {
                            lex.next();
                            break;
                        }
                            
                        // We have to do this `tmp_lex`, since otherwise the lex has the incorrect
                        // lifetime.
                        let (tmp_lex, arg_value) = ArgValue::lex_parse(lex)?;
                        lex = tmp_lex;

                        items.push(arg_value);

                        // This has to be an if statement since macro hygiene forbids us from
                        // putting a break in our expr.
                        if expect_token!(
                            lex.next(),
                            [Comma => false, CloseArray => true],
                            UnexpectedEnd,
                            |t| UnexpectedToken(t),
                        )? {
                            break;
                        }
                    }

                    Array(items)
                }
            ],
            UnexpectedEnd,
            |t| UnexpectedToken(t)
        )?;

        Ok((lex, arg_value))
    }
}

#[derive(Debug, PartialEq, Clone)]
/// The style used to create a string literal
/// TODO: Move this to be a Logos definition
pub enum QuoteType {
    /// `"`
    Double,
    /// `'`
    Single,
    /// `\``
    Backtick,
}

impl QuoteType {
    /// This will return all the constants belonging to a Quotation Type. This is the Quote Char,
    /// Quote Str, Escaped Quote Str, respectively.
    fn consts(&self) -> (char, &'static str, &'static str) {
        match self {
            &QuoteType::Double => ('"', "\"", "\\\""),
            &QuoteType::Single => ('\'', "'", "\\'"),
            &QuoteType::Backtick => ('`', "`", "\\`"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Logos)]
pub enum ArgValueToken {
    // Syntax for `'`, `"` and `\`` enclosed strings, respectively.
    #[regex(r#"'([^'\\]*(\\.[^'\\]*)*)'"#, |_| QuoteType::Single)]
    #[regex(r#""([^"\\]*(\\.[^"\\]*)*)""#, |_| QuoteType::Double)]
    #[regex(r#"`([^`\\]*(\\.[^`\\]*)*)`"#, |_| QuoteType::Backtick)]
    /// A string literal enclosed by `'`, `"` or `\``
    StrLiteral(QuoteType),

    #[token("true", |_| true)]
    #[token("True", |_| true)]
    #[token("false", |_| false)]
    #[token("False", |_| false)]
    /// A boolean literal `true` or `True` for true, and `false` or `False` for false.
    BoolLiteral(bool),

    // We need to take into account a lot of different ways floating point numbers can be written.
    // The first is needed because otherwise IntLiteral will take `123e12`.
    #[regex("[+-]?(0|[1-9][0-9]*)[eE][+-]?(0|[1-9][0-9]*)")]
    #[regex("[+-]?(0|[1-9][0-9]*)[.]([0-9]+)?([eE][+-]?(0|[1-9][0-9]*))?")]
    /// A floating point number literal
    FPLiteral,

    #[regex("[+-]?(0|[1-9][0-9]*)")]
    /// A signed integer literal
    IntLiteral,

    #[token("[")]
    /// The token used to open arrays, which is opening square bracket (`[`)
    OpenArray,
    #[token("]")]
    /// The token used to close arrays, which is closing square bracket (`]`)
    CloseArray,

    #[token(",")]
    /// A comma is used to delimit array elements
    Comma,

    #[regex("([a-zA-Z][a-zA-Z0-9_-]*)([.][a-zA-Z][a-zA-Z0-9_-]*)*")]
    /// A context dependent variable
    Variable,

    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_from_quoted() {
        assert_eq!(&string_from_quoted("'abc'", QuoteType::Single), r#"abc"#);
        assert_eq!(&string_from_quoted(r#"'a\'bc'"#, QuoteType::Single), r#"a'bc"#);
        assert_eq!(&string_from_quoted(r#"'a\'bc def\''"#, QuoteType::Single), r#"a'bc def'"#);
        assert_eq!(&string_from_quoted("' abc '", QuoteType::Single), r#" abc "#);

        assert_eq!(&string_from_quoted(r#""abc""#, QuoteType::Double), r#"abc"#);
        assert_eq!(&string_from_quoted(r#""a\"bc""#, QuoteType::Double), r#"a"bc"#);
        assert_eq!(&string_from_quoted(r#""a\"bc def\"""#, QuoteType::Double), r#"a"bc def""#);
        assert_eq!(&string_from_quoted(r#"" abc ""#, QuoteType::Double), r#" abc "#);

        assert_eq!(&string_from_quoted(r#"`abc`"#, QuoteType::Backtick), r#"abc"#);
        assert_eq!(&string_from_quoted(r#"`a\`bc`"#, QuoteType::Backtick), r#"a`bc"#);
        assert_eq!(&string_from_quoted(r#"`a\`bc def\``"#, QuoteType::Backtick), r#"a`bc def`"#);
        assert_eq!(&string_from_quoted(r#"` abc `"#, QuoteType::Backtick), r#" abc "#);

        assert_eq!(&string_from_quoted(r#"'a"b`c'"#, QuoteType::Single), r#"a"b`c"#);
        assert_eq!(&string_from_quoted(r#""a`b'c""#, QuoteType::Double), r#"a`b'c"#);
        assert_eq!(&string_from_quoted(r#"`a"b'c`"#, QuoteType::Backtick), r#"a"b'c"#);
    }

    use ArgValue::*;
    use ArgValueParseError::*;

    macro_rules! lex_assert_ok {
        ($content:expr, $outcome:expr$(, $rem:expr)?$(,)?) => {
            lex_assert_eq_ok!(ArgValueToken, ArgValue, $content, $outcome$(, $rem)?)
        }
    }

    macro_rules! lex_assert_err {
        ($content:expr, $outcome:expr) => {
            lex_assert_eq_err!(ArgValueToken, ArgValue, $content, $outcome)
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
        lex_assert_err!("*", UnexpectedToken(ArgValueToken::Error));
        lex_assert_err!("+", UnexpectedToken(ArgValueToken::Error));
        lex_assert_err!("-", UnexpectedToken(ArgValueToken::Error));
        lex_assert_err!("]", UnexpectedToken(ArgValueToken::CloseArray));
    }

    #[test]
    fn boolean() {
        lex_assert_ok!("true", Boolean(true), "");
        lex_assert_ok!("True", Boolean(true), "");
        lex_assert_ok!("True*", Boolean(true), "*");

        lex_assert_ok!("false", Boolean(false), "");
        lex_assert_ok!("False", Boolean(false), "");
        lex_assert_ok!("False*", Boolean(false), "*");
    }

    #[test]
    fn integer() {
        lex_assert_ok!("123", Integer(123), "");
        lex_assert_ok!("-123", Integer(-123), "");
        lex_assert_ok!("-123abc", Integer(-123), "abc");

        let pos_overflow = "9999999999999999";
        lex_assert_err!(pos_overflow, IntegerParseError(i32::from_str(pos_overflow).unwrap_err()));
    }

    #[test]
    fn float() {
        lex_assert_ok!("123.", FloatingPoint(123.), "");
        lex_assert_ok!("-123.", FloatingPoint(-123.), "");
        lex_assert_ok!("1.1", FloatingPoint(1.1), "");
        lex_assert_ok!("-0.1", FloatingPoint(-0.1), "");
        lex_assert_ok!("0.1abc", FloatingPoint(0.1), "abc");
        lex_assert_ok!("1e10abc", FloatingPoint(1e10), "abc");
        lex_assert_ok!("-1e10abc", FloatingPoint(-1e10), "abc");
    }

    #[test]
    fn text() {
        lex_assert_ok!("'abc'", Text("abc".to_string()), "");
        lex_assert_ok!("`abc`", Text("abc".to_string()), "");
        lex_assert_ok!("\"abc\"", Text("abc".to_string()), "");
        lex_assert_ok!("'abc'123", Text("abc".to_string()), "123");
    }

    macro_rules! vec_owned {
        ($($str:expr),*$(,)?) => {{
            vec![$($str.to_owned()),*]
        }}
    }

    #[test]
    fn variable() {
        lex_assert_ok!("abc.def.ghi", Var(vec_owned!["abc", "def", "ghi"]), "");
        lex_assert_ok!("abc", Var(vec_owned!["abc"]), "");
        lex_assert_ok!("abc12", Var(vec_owned!["abc12"]), "");
        lex_assert_ok!("abc12.abc", Var(vec_owned!["abc12", "abc"]), "");
        lex_assert_ok!("abc12.abc**", Var(vec_owned!["abc12", "abc"]), "**");
    }

    #[test]
    fn array() {
        lex_assert_ok!("[]", Array(vec![]), "");
        lex_assert_ok!(
            "[abc,def,ghi]",
            Array(vec![Var(vec_owned!["abc"]), Var(vec_owned!["def"]), Var(vec_owned!["ghi"])]),
            ""
        );
        lex_assert_ok!(
            "[123,def,true]",
            Array(vec![Integer(123), Var(vec_owned!["def"]), Boolean(true)]),
            ""
        );
        lex_assert_ok!(
            "[123,def,true]*",
            Array(vec![Integer(123), Var(vec_owned!["def"]), Boolean(true)]),
            "*"
        );
    }
}
