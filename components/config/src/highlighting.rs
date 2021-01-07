use lazy_static::lazy_static;
use syntect::dumps::from_binary;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

use crate::config::Config;

lazy_static! {
    pub static ref SYNTAX_SET: SyntaxSet = {
        let ss: SyntaxSet =
            from_binary(include_bytes!("../../../sublime/syntaxes/newlines.packdump"));
        ss
    };
    pub static ref THEME_SET: ThemeSet =
        from_binary(include_bytes!("../../../sublime/themes/all.themedump"));
}

/// Returns the highlighter and whether it was found in the extra or not
pub fn get_highlighter(language: Option<&str>, config: &Config) -> (HighlightLines<'static>, bool) {
    let theme = &THEME_SET.themes[config.highlight_theme()];
    let mut in_extra = false;

    if let Some(ref lang) = language {
        let syntax = if let Some(ref extra) = config.markdown.extra_syntax_set {
            let s = extra.find_syntax_by_token(lang);
            if s.is_some() {
                in_extra = true;
            }
            s
        } else {
            // The JS syntax hangs a lot... the TS syntax is probably better anyway.
            // https://github.com/getzola/zola/issues/1241
            // https://github.com/getzola/zola/issues/1211
            // https://github.com/getzola/zola/issues/1174
            let hacked_lang = if *lang == "js" || *lang == "javascript" { "ts" } else { lang };
            SYNTAX_SET.find_syntax_by_token(hacked_lang)
        }
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());
        (HighlightLines::new(syntax, theme), in_extra)
    } else {
        (HighlightLines::new(SYNTAX_SET.find_syntax_plain_text(), theme), false)
    }
}
