use syntect::dumps::from_binary;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

use Config;

lazy_static! {
    pub static ref SYNTAX_SET: SyntaxSet = {
        let ss: SyntaxSet =
            from_binary(include_bytes!("../../../sublime_syntaxes/newlines.packdump"));
        ss
    };
    pub static ref THEME_SET: ThemeSet =
        from_binary(include_bytes!("../../../sublime_themes/all.themedump"));
}

/// Returns the highlighter and whether it was found in the extra or not
pub fn get_highlighter<'a>(info: &str, config: &Config) -> (HighlightLines<'a>, bool) {
    let theme = &THEME_SET.themes[&config.highlight_theme];
    let mut in_extra = false;

    if let Some(ref lang) = info.split(' ').next() {
        let syntax = SYNTAX_SET
            .find_syntax_by_token(lang)
            .or_else(|| {
                if let Some(ref extra) = config.extra_syntax_set {
                    let s = extra.find_syntax_by_token(lang);
                    if s.is_some() {
                        in_extra = true;
                    }
                    s
                } else {
                    None
                }
            })
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());
        (HighlightLines::new(syntax, theme), in_extra)
    } else {
        (HighlightLines::new(SYNTAX_SET.find_syntax_plain_text(), theme), false)
    }
}
