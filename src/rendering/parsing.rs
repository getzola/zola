use syntect::parsing::SyntaxSet;
use syntect::dumps::from_binary;

thread_local! {
    pub static SYNTAX_SET: SyntaxSet = {
        let mut ss: SyntaxSet = from_binary(include_bytes!("../../sublime_syntaxes/newlines.packdump"));
        ss.link_syntaxes();
        ss
    };
}
