use std::collections::HashMap;
use std::ops::Range;

use errors::{Error, Result};
use logos::{Lexer, Logos};
use tera::{to_value, Context, Tera, Value};

use crate::shortcode::lexer::{Content, InnerShortcode};
use utils::templates::ShortcodeFileType;

pub const SHORTCODE_PLACEHOLDER: &str = "||ZOLA_SC_PLACEHOLDER||";

#[derive(PartialEq, Debug)]
pub struct Shortcode {
    pub(crate) name: String,
    pub(crate) args: Value,
    pub(crate) span: Range<usize>,
    pub(crate) body: Option<String>,
    // set later down the line, for quick access without needing the definitions
    pub(crate) tera_name: String,
    pub(crate) nth: usize,
}

impl Shortcode {
    pub fn file_type(&self) -> ShortcodeFileType {
        if self.tera_name.ends_with("md") {
            ShortcodeFileType::Markdown
        } else {
            ShortcodeFileType::Html
        }
    }

    pub fn render(self, tera: &Tera, context: &Context) -> Result<String> {
        let name = self.name;
        let tpl_name = self.tera_name;
        let mut new_context = Context::from_value(self.args)?;

        if let Some(body_content) = self.body {
            // Trimming right to avoid most shortcodes with bodies ending up with a HTML new line
            new_context.insert("body", body_content.trim_end());
        }
        new_context.insert("nth", &self.nth);
        new_context.extend(context.clone());

        let res = utils::templates::render_template(&tpl_name, tera, new_context, &None)
            .map_err(|e| errors::Error::chain(format!("Failed to render {} shortcode", name), e))?;

        Ok(res)
    }

    pub fn update_range(&mut self, sc_span: &Range<usize>, rendered_length: usize) {
        if self.span.start > sc_span.start {
            let delta = if sc_span.end < rendered_length {
                rendered_length - sc_span.end
            } else {
                sc_span.end - rendered_length
            };

            if sc_span.end < rendered_length {
                self.span = (self.span.start + delta)..(self.span.end + delta);
            } else {
                self.span = (self.span.start - delta)..(self.span.end - delta);
            }
        }
    }
}

struct InnerShortcodeParser<'a, 'b> {
    name: String,
    source: &'a str,
    lexer: &'b mut Lexer<'a, InnerShortcode>,
}

impl<'a, 'b> InnerShortcodeParser<'a, 'b> {
    fn next_or_eof(&mut self) -> Result<InnerShortcode> {
        match self.lexer.next() {
            None => Err(Error::msg(format!(
                "Unexpected end of content while parsing shortcode {}",
                self.name
            ))),
            Some(t) => Ok(t),
        }
    }

    fn expect_token(&mut self, expected_token: InnerShortcode) -> Result<()> {
        let token = self.next_or_eof()?;
        if token != expected_token {
            Err(Error::msg(format!(
                "Unexpected token {} while looking for {} of shortcode {}",
                token, expected_token, self.name
            )))
        } else {
            Ok(())
        }
    }

    fn parse_array(&mut self) -> Result<Value> {
        let mut values = Vec::new();
        loop {
            values.push(self.parse_value()?);
            match self.next_or_eof()? {
                InnerShortcode::Comma => {
                    continue;
                }
                InnerShortcode::CloseBracket => break,
                t => {
                    return Err(Error::msg(format!(
                        "Unexpected token {} while looking for `,` or `]` in shortcode {}",
                        t, self.name
                    )));
                }
            };
        }

        Ok(Value::Array(values))
    }

    fn parse_value(&mut self) -> Result<Value> {
        match self.next_or_eof()? {
            InnerShortcode::Bool(b) => Ok(Value::Bool(b)),
            InnerShortcode::Str(s) => Ok(Value::String(s)),
            InnerShortcode::Integer(i) => Ok(to_value(i).expect("valid i64")),
            InnerShortcode::Float(f) => Ok(to_value(f).expect("valid f64")),
            InnerShortcode::OpenBracket => self.parse_array(),
            t => {
                return Err(Error::msg(format!(
                    "Unexpected token {} while parsing arguments of shortcode {}",
                    t, self.name
                )));
            }
        }
    }

    pub fn parse(
        source: &'a str,
        lexer: &'b mut Lexer<'a, InnerShortcode>,
    ) -> Result<(String, Value)> {
        let mut parser = Self { source, lexer, name: String::new() };

        parser.expect_token(InnerShortcode::Ident)?;
        parser.name = parser.source[parser.lexer.span()].to_string();
        parser.expect_token(InnerShortcode::OpenParenthesis)?;
        let mut arguments = HashMap::new();

        loop {
            match parser.next_or_eof()? {
                InnerShortcode::CloseParenthesis => {
                    break;
                }
                InnerShortcode::Comma => {
                    continue;
                }
                InnerShortcode::Ident => {
                    let ident = source[parser.lexer.span()].to_string();
                    parser.expect_token(InnerShortcode::Equals)?;
                    let value = parser.parse_value()?;
                    arguments.insert(ident, value);
                }
                t => {
                    return Err(Error::msg(format!(
                        "Unexpected token {} while parsing arguments of shortcode {}",
                        t, parser.name
                    )));
                }
            }
        }

        Ok((parser.name, to_value(arguments).unwrap()))
    }
}

pub(crate) struct ShortcodeExtractor<'a> {
    source: &'a str,
    output: String,
    last_lex_end: usize,
    lexer: Lexer<'a, Content>,
}

impl<'a> ShortcodeExtractor<'a> {
    /// Only called if there was a `{{` or a `{%` in the source input
    pub fn parse(source: &'a str) -> Result<(String, Vec<Shortcode>)> {
        let sc = Self {
            source,
            output: String::with_capacity(source.len()),
            last_lex_end: 0,
            lexer: Content::lexer(source),
        };

        sc.process()
    }

    fn expect_token(&mut self, expected_token: Content) -> Result<()> {
        let token = self.next_or_eof()?;
        if token != expected_token {
            Err(Error::msg(format!(
                "Unexpected token {} while looking for {}",
                token, expected_token
            )))
        } else {
            Ok(())
        }
    }

    fn next_or_eof(&mut self) -> Result<Content> {
        match self.lexer.next() {
            None => Err(Error::msg("Unexpected end of content while parsing shortcodes")),
            Some(t) => Ok(t),
        }
    }

    fn parse_until(&mut self, token: Content) -> Result<()> {
        loop {
            let tok = self.next_or_eof()?;
            if tok == token {
                return Ok(());
            }
        }
    }

    fn process(mut self) -> Result<(String, Vec<Shortcode>)> {
        let mut shortcodes = Vec::new();
        let mut nths = HashMap::new();

        loop {
            // TODO: some code duplications here but nothing major
            match self.lexer.next() {
                None => {
                    // We're done, pushing whatever's left
                    self.output.push_str(&self.source[self.last_lex_end..]);
                    break;
                }
                Some(Content::InlineShortcodeStart) => {
                    self.output.push_str(&self.source[self.last_lex_end..self.lexer.span().start]);
                    let start = self.output.len();
                    let mut inner_lexer = self.lexer.morph();
                    let (name, args) = InnerShortcodeParser::parse(self.source, &mut inner_lexer)?;
                    self.lexer = inner_lexer.morph();
                    self.expect_token(Content::InlineShortcodeEnd)?;
                    self.output.push_str(SHORTCODE_PLACEHOLDER);
                    self.last_lex_end = self.lexer.span().end;
                    let nth = *nths.entry(name.to_owned()).and_modify(|e| *e += 1).or_insert(1);
                    shortcodes.push(Shortcode {
                        name,
                        args,
                        span: start..(start + SHORTCODE_PLACEHOLDER.len()),
                        body: None,
                        nth,
                        tera_name: String::new(),
                    });
                }
                Some(Content::IgnoredInlineShortcodeStart) => {
                    self.output.push_str(&self.source[self.last_lex_end..self.lexer.span().start]);
                    self.output.push_str("{{");
                    self.last_lex_end = self.lexer.span().end;
                    self.parse_until(Content::IgnoredInlineShortcodeEnd)?;
                    self.output.push_str(&self.source[self.last_lex_end..self.lexer.span().start]);
                    self.output.push_str("}}");
                    self.last_lex_end = self.lexer.span().end;
                }
                Some(Content::ShortcodeWithBodyStart) => {
                    self.output.push_str(&self.source[self.last_lex_end..self.lexer.span().start]);
                    let start = self.output.len();
                    let mut inner_lexer = self.lexer.morph();
                    let (name, args) = InnerShortcodeParser::parse(self.source, &mut inner_lexer)?;
                    self.lexer = inner_lexer.morph();
                    self.expect_token(Content::ShortcodeWithBodyEnd)?;
                    let body_start = self.lexer.span().end;
                    self.parse_until(Content::ShortcodeWithBodyClosing)?;
                    // We trim the start to avoid newlines that users would have put to make the shortcode pretty in md, eg
                    // {% hello() %}
                    // body
                    // {% end %}
                    // it's unlikely that the user wanted/expected a newline before or after "body"
                    let body = self.source[body_start..self.lexer.span().start].trim().to_owned();
                    self.last_lex_end = self.lexer.span().end;
                    self.output.push_str(SHORTCODE_PLACEHOLDER);
                    let nth = *nths.entry(name.to_owned()).and_modify(|e| *e += 1).or_insert(1);
                    shortcodes.push(Shortcode {
                        name,
                        args,
                        span: start..(start + SHORTCODE_PLACEHOLDER.len()),
                        body: Some(body),
                        nth,
                        tera_name: String::new(),
                    });
                }
                Some(Content::IgnoredShortcodeWithBodyStart) => {
                    self.output.push_str(&self.source[self.last_lex_end..self.lexer.span().start]);
                    self.output.push_str("{%");
                    self.last_lex_end = self.lexer.span().end;
                    self.parse_until(Content::IgnoredShortcodeWithBodyEnd)?;
                    self.output.push_str(&self.source[self.last_lex_end..self.lexer.span().start]);
                    self.output.push_str("%}");
                    self.last_lex_end = self.lexer.span().end;
                    self.parse_until(Content::IgnoredShortcodeWithBodyClosing)?;
                    self.output.push_str(&self.source[self.last_lex_end..self.lexer.span().start]);
                    self.output.push_str("{% end %}");
                    self.last_lex_end = self.lexer.span().end;
                }
                Some(Content::Error) => {
                    // Likely just a `*`, `/`, `{`
                    self.output.push_str(&self.source[self.last_lex_end..self.lexer.span().end]);
                    self.last_lex_end = self.lexer.span().end;
                }
                Some(c) => {
                    return Err(Error::msg(format!(
                        "Unexpected token {} while parsing shortcodes",
                        c
                    )));
                }
            }
        }

        Ok((self.output, shortcodes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // From maplit
    macro_rules! hashmap {
    (@single $($x:tt)*) => (());
        (@count $($rest:expr),*) => (<[()]>::len(&[$(hashmap!(@single $rest)),*]));

        ($($key:expr => $value:expr,)+) => { hashmap!($($key => $value),+) };
        ($($key:expr => $value:expr),*) => {
            {
                let _cap = hashmap!(@count $($key),*);
                let mut _map = ::std::collections::HashMap::with_capacity(_cap);
                $(
                    let _ = _map.insert($key, $value);
                )*
                _map
            }
        };
    }

    #[test]
    fn can_update_ranges() {
        let mut sc = Shortcode {
            name: "a".to_string(),
            args: Value::Null,
            span: 10..20,
            body: None,
            nth: 0,
            tera_name: String::new(),
        };
        sc.update_range(&(2..8), 10);
        assert_eq!(sc.span, 12..22);
        sc.update_range(&(24..30), 30);
        assert_eq!(sc.span, 12..22);
        sc.update_range(&(5..11), 6);
        assert_eq!(sc.span, 7..17);
    }

    #[test]
    fn can_parse_inner_shortcode() {
        let input = vec![
            // txt, name, args
            ("hello() }}", "hello", HashMap::new()),
            ("hello_lo1() }}", "hello_lo1", HashMap::new()),
            (
                " shortcode(name='bob', age=45) }}",
                "shortcode",
                hashmap!("name".to_owned() => Value::String("bob".to_owned()), "age".to_owned() => to_value(45).unwrap()),
            ),
            (
                " shortcode(admin=true, age=45.1) }}",
                "shortcode",
                hashmap!("admin".to_owned() => Value::Bool(true), "age".to_owned() => to_value(45.1).unwrap()),
            ),
            (
                "with_array(hello=['true', false]) }}",
                "with_array",
                hashmap!("hello".to_owned() => Value::Array(vec![Value::String("true".to_owned()), Value::Bool(false)])),
            ),
        ];

        for (txt, expected_name, expected_args) in input {
            let mut lexer = InnerShortcode::lexer(txt);
            let (name, args) = InnerShortcodeParser::parse(txt, &mut lexer).unwrap();
            assert_eq!(&name, expected_name);
            assert_eq!(args, to_value(expected_args).unwrap());
        }
    }

    #[test]
    fn can_extract_basic_inline_shortcode_with_args() {
        let (out, shortcodes) = ShortcodeExtractor::parse(
            "Inline shortcode: {{ hello(string='hey', int=1, float=2.1, bool=true) }} hey",
        )
        .unwrap();
        assert_eq!(out, format!("Inline shortcode: {} hey", SHORTCODE_PLACEHOLDER));
        assert_eq!(shortcodes.len(), 1);
        assert_eq!(shortcodes[0].name, "hello");
        assert_eq!(shortcodes[0].args.as_object().unwrap().len(), 4);
        assert_eq!(shortcodes[0].args["string"], Value::String("hey".to_string()));
        assert_eq!(shortcodes[0].args["bool"], Value::Bool(true));
        assert_eq!(shortcodes[0].args["int"], to_value(1).unwrap());
        assert_eq!(shortcodes[0].args["float"], to_value(2.1).unwrap());
        assert_eq!(shortcodes[0].span, 18..(18 + SHORTCODE_PLACEHOLDER.len()));
        assert_eq!(shortcodes[0].nth, 1);
    }

    #[test]
    fn can_unignore_ignored_inline_shortcode() {
        let (out, shortcodes) =
            ShortcodeExtractor::parse("Hello World {{/* youtube() */}} hey").unwrap();
        assert_eq!(out, "Hello World {{ youtube() }} hey");
        assert_eq!(shortcodes.len(), 0);
    }

    #[test]
    fn can_extract_shortcode_with_body() {
        let (out, shortcodes) = ShortcodeExtractor::parse(
            "Body shortcode\n {% quote(author='Bobby') %}DROP TABLES;{% end %} \n hey",
        )
        .unwrap();
        assert_eq!(out, format!("Body shortcode\n {} \n hey", SHORTCODE_PLACEHOLDER));
        assert_eq!(shortcodes.len(), 1);
        assert_eq!(shortcodes[0].name, "quote");
        assert_eq!(shortcodes[0].args.as_object().unwrap().len(), 1);
        assert_eq!(shortcodes[0].args["author"], Value::String("Bobby".to_string()));
        assert_eq!(shortcodes[0].body, Some("DROP TABLES;".to_owned()));
        assert_eq!(shortcodes[0].span, 16..(16 + SHORTCODE_PLACEHOLDER.len()));
        assert_eq!(shortcodes[0].nth, 1);
    }

    #[test]
    fn can_unignore_ignored_shortcode_with_body() {
        let (out, shortcodes) =
            ShortcodeExtractor::parse("Hello World {%/* youtube() */%} Somebody {%/* end */%} hey")
                .unwrap();
        assert_eq!(out, "Hello World {% youtube() %} Somebody {% end %} hey");
        assert_eq!(shortcodes.len(), 0);
    }

    #[test]
    fn can_extract_multiple_shortcodes_and_increment_nth() {
        let (out, shortcodes) = ShortcodeExtractor::parse(
            "Hello World {% youtube() %} Somebody {% end %} {{ hello() }}\n {{hello()}}",
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
    fn can_handle_multiple_shortcodes() {
        let (_, shortcodes) = ShortcodeExtractor::parse(
            r#"
        {{ youtube(id="ub36ffWAqgQ") }}
        {{ youtube(id="ub36ffWAqgQ", autoplay=true) }}
        {{ vimeo(id="210073083") }}
        {{ streamable(id="c0ic") }}
        {{ gist(url="https://gist.github.com/Keats/32d26f699dcc13ebd41b") }}"#,
        )
        .unwrap();
        assert_eq!(shortcodes.len(), 5);
    }

    #[test]
    fn can_provide_good_error_messages() {
        let tests = vec![
            ("{{ hey()", "Unexpected end of content while parsing shortcodes"),
            ("{% hey()", "Unexpected end of content while parsing shortcodes"),
            ("{{ hey(=", "Unexpected token `=` while parsing arguments of shortcode hey"),
            ("{{ hey(ho==", "Unexpected token `=` while parsing arguments of shortcode hey"),
            ("{{ hey(ho=1h", "Unexpected end of content while parsing shortcode hey"),
            ("{{ hey)", "Unexpected token `)` while looking for `(` of shortcode hey"),
            ("{{ hey(ho=(", "Unexpected token `(` while parsing arguments of shortcode hey"),
            ("hello }}", "Unexpected token `}}` while parsing shortcodes"),
            ("hello %}", "Unexpected token `%}` while parsing shortcodes"),
            ("hello {% end %}", "Unexpected token `{% end %}` while parsing shortcodes"),
        ];

        for (t, expected) in tests {
            println!("Testing: {}", t);
            let res = ShortcodeExtractor::parse(t);
            assert!(res.is_err());
            let err = res.unwrap_err();
            assert_eq!(expected, err.to_string());
        }
    }

    #[test]
    fn can_parse_ok_with_problematic_chars() {
        let inputs = vec![
            ("* a\n* b", "* a\n* b"),
            ("a regex {a,b}", "a regex {a,b}"),
            ("a slash //", "a slash //"),
            ("a slash */", "a slash */"),
            ("%a percent%", "%a percent%"),
        ];

        for (input, expected) in inputs {
            let (out, _) = ShortcodeExtractor::parse(input).unwrap();
            assert_eq!(out, expected);
        }
    }
}
