use logos::Logos;

pub enum ArgValue {
    Text(String),
    Boolean(bool),
    FloatingPoint(f32),
    Integer(f32),
    Array(Vec<ArgValue>),
}

pub struct Shortcode {
    name: String,
    args: HashMap<String, ArgValue>,
    body: Option<String>,
}

#[derive(Debug, PartialEq)]
enum SyntaxType {
    Normal,
    Body,
}

#[derive(Debug, PartialEq)]
enum FPRepresentationType {
    Plain,
    MiddlePoint,
    StartingPoint,
}

#[derive(Debug, PartialEq)]
enum StringType {
    DoubleQuoted,
    SingleQuoted,
    BackQuoted,
}

#[derive(Debug, PartialEq, Logos)]
enum Token {
    #[token("end")]
    EndKeyword,

    #[token("{%", |_| SyntaxType::Body)]
    #[token("{{", |_| SyntaxType::Normal)]
    OpenShortcode(SyntaxType),

    #[token("%}", |_| SyntaxType::Body)]
    #[token("}}", |_| SyntaxType::Normal)]
    CloseShortcode(SyntaxType),

    #[token("(")]
    OpenParenthesis,
    #[token(")")]
    CloseParenthesis,

    #[regex(r#""([^"\\]*(\\.[^"\\]*)*)""#, |_| StringType::DoubleQuoted)]
    #[regex(r#"'([^'\\]*(\\.[^'\\]*)*)'"#, |_| StringType::SingleQuoted)]
    #[regex(r#"`([^`\\]*(\\.[^`\\]*)*)`"#, |_| StringType::BackQuoted)]
    QuotedString(StringType),

    #[regex("(?:true)", |_| true)]
    #[regex("(?i:false)", |_| false)]
    Boolean(bool),

    #[regex("[+-]?(0|[1-9][0-9]*)[.]([0-9]+)", |_| FPRepresentationType::MiddlePoint)]
    #[regex("[+-]?(0|[1-9][0-9]*)[.]", |_| FPRepresentationType::Plain)]
    #[regex("[+-]?[.]([0-9]+)", |_| FPRepresentationType::StartingPoint)]
    FloatingPointBase(FPRepresentationType),

    #[regex("(?i:e[0-9]+)?")]
    FloatingPointExt,

    #[regex("[-]?(0|[1-9][0-9]*)")]
    Integer,

    #[token("[")]
    OpenArray,
    #[token("]")]
    CloseArray,

    #[token("=")]
    Equals,
    #[token(",")]
    Comma,
    #[token(".")]
    Period,

    #[regex("[a-zA-Z][a-zA-Z0-9_-]*")]
    Identifier,

    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error
}

#[cfg(test)]
mod tests {
    use super::{FPRepresentationType, Token, SyntaxType, StringType};
    use logos::Logos;

    #[test]
    fn it_lexes() {
        let test_str = r"{{ abc(wow=true) }}";

        let mut lex = Token::lexer(test_str);

        assert_eq!(lex.next(), Some(Token::OpenShortcode(SyntaxType::Normal)));
        assert_eq!(lex.span(), 0..2);
        assert_eq!(lex.slice(), "{{");

        assert_eq!(lex.next(), Some(Token::Identifier));
        assert_eq!(lex.span(), 3..6);
        assert_eq!(lex.slice(), "abc");

        assert_eq!(lex.next(), Some(Token::OpenParenthesis));
        assert_eq!(lex.span(), 6..7);
        assert_eq!(lex.slice(), "(");

        assert_eq!(lex.next(), Some(Token::Identifier));
        assert_eq!(lex.span(), 7..10);
        assert_eq!(lex.slice(), "wow");

        assert_eq!(lex.next(), Some(Token::Equals));
        assert_eq!(lex.span(), 10..11);
        assert_eq!(lex.slice(), "=");

        assert_eq!(lex.next(), Some(Token::Boolean(true)));
        assert_eq!(lex.span(), 11..15);
        assert_eq!(lex.slice(), "true");

        assert_eq!(lex.next(), Some(Token::CloseParenthesis));
        assert_eq!(lex.span(), 15..16);
        assert_eq!(lex.slice(), ")");

        assert_eq!(lex.next(), Some(Token::CloseShortcode(SyntaxType::Normal)));
        assert_eq!(lex.span(), 17..19);
        assert_eq!(lex.slice(), "}}");
    }
}
