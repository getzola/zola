use logos::Logos;

#[derive(Debug, PartialEq)]
/// The style used to create a string literal
pub enum QuoteType {
    /// `"`
    Double,
    /// `'`
    Single,
    /// `\``
    Backtick,
}

#[derive(Debug, PartialEq)]
pub enum QuoteParseError {
    ClosingQuoteEncountedEarly(usize),
    UnknownError,
}

/// Given a quoted source string and a quotation style used will turn an escaped string into an
/// unescaped string.
///
/// Will insert `\\n`, `\\t` and `\\r`. Will unescape escaped quotes (using the quotation
/// style) and escaped backslashes.
pub fn unescape_quoted_string(
    source: &str,
    quote_type: QuoteType,
) -> Result<String, QuoteParseError> {
    use QuoteParseError::*;
    use QuoteToken::*;
    use QuoteType::*;

    // In debug we can make sure that we are actually removing the proper quotes
    debug_assert!(source.len() > 2);
    debug_assert_eq!((source.chars().nth(0), source.chars().last()), {
        let quote_char = match quote_type {
            Double => '"',
            Single => '\'',
            Backtick => '`',
        };

        (Some(quote_char), Some(quote_char))
    });

    // Remove surrounding quotes
    let source = &source[1..source.len() - 1];
    let mut lex = QuoteToken::lexer(source);

    // Used to keep track of where the last token ended.
    let mut last = 0;

    // We can allocate a string which is as long as the source string.
    //
    // This way it will never grow since the only manipulations we do to the source string are
    // potential reductions in size.
    let mut output = String::with_capacity(source.len());

    while let Some(token) = lex.next() {
        // Push the string from the end of the last token till now
        output.push_str(&source[last..lex.span().start]);

        output.push_str(match (token, &quote_type) {
            // Normal escaped characters
            (Backslash, _) => "\\",
            (NewLine, _) => "\n",
            (HorizontalTab, _) => "\t",
            (CarriageReturn, _) => "\r",

            // If we encounter a unescaped quote in the quotation style, something went wrong.
            (SingleQuote, Single) | (DoubleQuote, Double) | (BacktickQuote, Backtick) => {
                return Err(ClosingQuoteEncountedEarly(lex.span().start))
            }

            // Escaped quotes for the quotation style and quotations not matching the style can
            // just be insertes plainly.
            (EscapedSingleQuote, Single) | (SingleQuote, _) => "'",
            (EscapedDoubleQuote, Double) | (DoubleQuote, _) => "\"",
            (EscapedBacktick, Backtick) | (BacktickQuote, _) => "`",

            // Escaped quotes not matching the quotation style will still have a `\` before them.
            (EscapedSingleQuote, _) => "\\'",
            (EscapedDoubleQuote, _) => "\\\"",
            (EscapedBacktick, _) => "\\`",

            _ => return Err(UnknownError),
        });

        last = lex.span().end;
    }

    // Add string from last till the end
    output.push_str(&source[last..]);

    Ok(output)
}

#[derive(Debug, PartialEq, Logos)]
/// The tokens that are of interest during a quoted string
enum QuoteToken {
    #[token(r"\\")]
    Backslash,

    #[token(r"\n")]
    NewLine,
    #[token(r"\t")]
    HorizontalTab,
    #[token(r"\r")]
    CarriageReturn,

    #[token(r"\'")]
    EscapedSingleQuote,
    #[token(r#"\""#)]
    EscapedDoubleQuote,
    #[token(r"\`")]
    EscapedBacktick,

    #[token(r"'")]
    SingleQuote,
    #[token(r#"""#)]
    DoubleQuote,
    #[token(r"`")]
    BacktickQuote,

    #[error]
    #[regex(r#"[^'`"\\]+"#, logos::skip)]
    Error,
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! unescaped_test_ok {
        ($quote:ident, [$($input:expr => $output:expr),+$(,)?]$(,)?) => {
            $(
                let unescaped_str = unescape_quoted_string($input, <QuoteType>::$quote)
                    .expect("Expected unescaped string to be Ok");
                assert_eq!(unescaped_str, String::from($output));
            )+
        }
    }

    #[test]
    fn new_from_quoted() {
        unescaped_test_ok!(
            Single,
            [
                r#"'abc'"# => r#"abc"#,
                r#"'a\'bc'"# => r#"a'bc"#,
                r#"'a\'bc def\''"# => r#"a'bc def'"#,
                r#"' abc '"# => r#" abc "#,
            ],
        );

        unescaped_test_ok!(
            Double,
            [
                r#""abc""# => r#"abc"#,
                r#""a\"bc""# => r#"a"bc"#,
                r#""a\"bc def\"""# => r#"a"bc def""#,
                r#"" abc ""# => r#" abc "#,
            ],
        );

        unescaped_test_ok!(
            Backtick,
            [
                r#"`abc`"# => r#"abc"#,
                r#"`a\`bc`"# => r#"a`bc"#,
                r#"`a\`bc def\``"# => r#"a`bc def`"#,
                r#"` abc `"# => r#" abc "#,
            ],
        );

        unescaped_test_ok!(
            Single,
            [
                r#"'a"b`c'"# => r#"a"b`c"#,
            ],
        );

        unescaped_test_ok!(
            Double,
            [
                r#""a'b`c""# => r#"a'b`c"#,
            ],
        );

        unescaped_test_ok!(
            Backtick,
            [
                r#"`a'b"c`"# => r#"a'b"c"#,
            ],
        );

        unescaped_test_ok!(
            Single,
            [
                r#"'a\\\n\t\rbc'"# => "a\\\n\t\rbc",
            ],
        );
    }
}
