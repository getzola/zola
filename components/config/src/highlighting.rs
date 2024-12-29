use libs::once_cell::sync::Lazy;
use libs::syntect::dumps::from_binary;
use libs::syntect::highlighting::{Theme, ThemeSet};
use libs::syntect::html::ClassStyle;
use libs::syntect::parsing::{SyntaxReference, SyntaxSet};

use crate::config::Config;

pub const CLASS_STYLE: ClassStyle = ClassStyle::SpacedPrefixed { prefix: "z-" };

pub static SYNTAX_SET: Lazy<SyntaxSet> =
    Lazy::new(|| from_binary(include_bytes!("../sublime/syntaxes/newlines.packdump")));

pub static THEME_SET: Lazy<ThemeSet> =
    Lazy::new(|| from_binary(include_bytes!("../sublime/themes/all.themedump")));

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HighlightSource {
    /// One of the built-in Zola syntaxes
    BuiltIn,
    /// Found in the extra syntaxes
    Extra,
    /// No language specified
    Plain,
    /// We didn't find the language in built-in and extra syntaxes
    NotFound,
}

pub struct SyntaxAndTheme<'config> {
    pub syntax: &'config SyntaxReference,
    pub syntax_set: &'config SyntaxSet,
    /// None if highlighting via CSS
    pub theme: Option<&'config Theme>,
    pub source: HighlightSource,
}

pub fn resolve_syntax_and_theme<'config>(
    language: Option<&'_ str>,
    config: &'config Config,
) -> SyntaxAndTheme<'config> {
    let theme = config.markdown.get_highlight_theme();

    if let Some(ref lang) = language {
        if let Some(ref extra_syntaxes) = config.markdown.extra_syntax_set {
            if let Some(syntax) = extra_syntaxes.find_syntax_by_token(lang) {
                return SyntaxAndTheme {
                    syntax,
                    syntax_set: extra_syntaxes,
                    theme,
                    source: HighlightSource::Extra,
                };
            }
        }
        // The JS syntax hangs a lot... the TS syntax is probably better anyway.
        // https://github.com/getzola/zola/issues/1241
        // https://github.com/getzola/zola/issues/1211
        // https://github.com/getzola/zola/issues/1174
        let hacked_lang = if *lang == "js" || *lang == "javascript" { "ts" } else { lang };
        if let Some(syntax) = SYNTAX_SET.find_syntax_by_token(hacked_lang) {
            SyntaxAndTheme {
                syntax,
                syntax_set: &SYNTAX_SET as &SyntaxSet,
                theme,
                source: HighlightSource::BuiltIn,
            }
        } else {
            SyntaxAndTheme {
                syntax: SYNTAX_SET.find_syntax_plain_text(),
                syntax_set: &SYNTAX_SET as &SyntaxSet,
                theme,
                source: HighlightSource::NotFound,
            }
        }
    } else {
        SyntaxAndTheme {
            syntax: SYNTAX_SET.find_syntax_plain_text(),
            syntax_set: &SYNTAX_SET as &SyntaxSet,
            theme,
            source: HighlightSource::Plain,
        }
    }
}
