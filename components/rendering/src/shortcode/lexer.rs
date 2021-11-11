use logos::Logos;
use std::fmt::Formatter;

fn replace_string_markers(input: &str, marker: char) -> String {
    input.replace(marker, "")
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Content {
    #[token("{{")]
    InlineShortcodeStart,
    #[token("{{/*")]
    IgnoredInlineShortcodeStart,
    #[token("{%")]
    ShortcodeWithBodyStart,
    #[token("{%/*")]
    IgnoredShortcodeWithBodyStart,

    #[token("}}")]
    InlineShortcodeEnd,
    #[token("*/}}")]
    IgnoredInlineShortcodeEnd,
    #[token("%}")]
    ShortcodeWithBodyEnd,
    #[token("*/%}")]
    IgnoredShortcodeWithBodyEnd,
    #[token("{% end %}")]
    ShortcodeWithBodyClosing,
    #[token("{%/* end */%}")]
    IgnoredShortcodeWithBodyClosing,

    #[regex(r"[^{\*%\}]+", logos::skip)]
    #[error]
    Error,
}

impl std::fmt::Display for Content {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Content::InlineShortcodeStart => "`{{`",
            Content::IgnoredInlineShortcodeStart => "`{{/*`",
            Content::ShortcodeWithBodyStart => "`{%`",
            Content::IgnoredShortcodeWithBodyStart => "`{%/*",
            Content::InlineShortcodeEnd => "`}}`",
            Content::IgnoredInlineShortcodeEnd => "`*/}}`",
            Content::ShortcodeWithBodyEnd => "`%}`",
            Content::IgnoredShortcodeWithBodyEnd => "`*/%}`",
            Content::ShortcodeWithBodyClosing => "`{% end %}`",
            Content::IgnoredShortcodeWithBodyClosing => "`{%/* end */%}`",
            Content::Error => "`error`",
        };

        write!(f, "{}", val)
    }
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum InnerShortcode {
    #[token("(")]
    OpenParenthesis,
    #[token(")")]
    CloseParenthesis,
    #[token("[")]
    OpenBracket,
    #[token("]")]
    CloseBracket,
    #[token(",")]
    Comma,
    #[token("=")]
    Equals,
    #[regex("[a-zA-Z][a-zA-Z0-9_]*")]
    Ident,
    #[regex("-?[0-9]+", |lex| lex.slice().parse())]
    Integer(i64),
    #[regex("-?[0-9]+\\.[0-9]+", |lex| lex.slice().parse())]
    Float(f64),
    #[token("true", |_| true)]
    #[token("True", |_| true)]
    #[token("false", |_| false)]
    #[token("False", |_| false)]
    Bool(bool),
    #[regex(r#"'([^'\\]*(\\.[^'\\]*)*)'"#, |lex| replace_string_markers(lex.slice(), '\''))]
    #[regex(r#""([^"\\]*(\\.[^"\\]*)*)""#, |lex| replace_string_markers(lex.slice(), '"'))]
    #[regex(r#"`([^`\\]*(\\.[^`\\]*)*)`"#, |lex| replace_string_markers(lex.slice(), '`'))]
    Str(String),

    #[regex(r"[ \t\n\f]+", logos::skip)]
    #[error]
    Error,
}

impl std::fmt::Display for InnerShortcode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            InnerShortcode::OpenParenthesis => "`(`",
            InnerShortcode::CloseParenthesis => "`)`",
            InnerShortcode::OpenBracket => "`[`",
            InnerShortcode::CloseBracket => "`]`",
            InnerShortcode::Comma => "`,`",
            InnerShortcode::Equals => "`=`",
            InnerShortcode::Ident => "`identifier`",
            InnerShortcode::Integer(_) => "`integer`",
            InnerShortcode::Float(_) => "`float`",
            InnerShortcode::Bool(_) => "`boolean`",
            InnerShortcode::Str(_) => "`string`",
            InnerShortcode::Error => "`error`",
        };

        write!(f, "{}", val)
    }
}
