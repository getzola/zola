#[macro_use]
extern crate lazy_static;
extern crate syntect;

use syntect::dumps::from_binary;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Theme};
use syntect::easy::HighlightLines;

thread_local!{
    pub static SYNTAX_SET: SyntaxSet = {
        let mut ss: SyntaxSet = from_binary(include_bytes!("../../../sublime_syntaxes/newlines.packdump"));
        ss.link_syntaxes();
        ss
    };
}

lazy_static!{
    pub static ref THEME_SET: ThemeSet = from_binary(include_bytes!("../../../sublime_themes/all.themedump"));
}


pub fn get_highlighter<'a>(theme: &'a Theme, info: &str) -> HighlightLines<'a> {
    SYNTAX_SET.with(|ss| {
        let syntax = info
            .split(' ')
            .next()
            .and_then(|lang| ss.find_syntax_by_token(lang))
            .unwrap_or_else(|| ss.find_syntax_plain_text());
        HighlightLines::new(syntax, theme)
    })
}
