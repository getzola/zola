#[macro_use]
extern crate lazy_static;
extern crate syntect;

use syntect::dumps::from_binary;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;

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
