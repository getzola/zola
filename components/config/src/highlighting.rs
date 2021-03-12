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

pub enum HighlightSource {
    Theme,
    Extra,
    Plain,
    NotFound,
}

/// Returns the highlighter and whether it was found in the extra or not
pub fn get_highlighter(
    language: Option<&str>,
    config: &Config,
) -> (HighlightLines<'static>, HighlightSource) {
    let theme = &THEME_SET.themes[&config.markdown.highlight_theme];

    if let Some(ref lang) = language {
        if let Some(ref extra_syntaxes) = config.markdown.extra_syntax_set {
            if let Some(syntax) = extra_syntaxes.find_syntax_by_token(lang) {
                return (HighlightLines::new(syntax, theme), HighlightSource::Extra);
            }
        }
        // The JS syntax hangs a lot... the TS syntax is probably better anyway.
        // https://github.com/getzola/zola/issues/1241
        // https://github.com/getzola/zola/issues/1211
        // https://github.com/getzola/zola/issues/1174
        let hacked_lang = if *lang == "js" || *lang == "javascript" { "ts" } else { lang };
        if let Some(syntax) = SYNTAX_SET.find_syntax_by_token(hacked_lang) {
            (HighlightLines::new(syntax, theme), HighlightSource::Theme)
        } else {
            (
                HighlightLines::new(SYNTAX_SET.find_syntax_plain_text(), theme),
                HighlightSource::NotFound,
            )
        }
    } else {
        (HighlightLines::new(SYNTAX_SET.find_syntax_plain_text(), theme), HighlightSource::Plain)
    }
}
