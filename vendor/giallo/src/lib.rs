//! A syntax highlighting library using TextMate grammars and themes and producing the same
//! output as VSCode.
//!
//! # Example
//!
//! ```ignore
//! use giallo::{HighlightOptions, HtmlRenderer, Options, Registry, ThemeVariant};
//!
//! // Using the `dump` feature and loading the prebuilt assets
//! let registry = Registry::load_from_file("builtin.zst")?;
//! let code = "let x = 42;";
//!
//! let options = HighlightOptions::new("javascript", ThemeVariant::Single("catppuccin-frappe"));
//! let highlighted = registry.highlight(code, options)?;
//!
//! let render_options = Options {
//!     show_line_numbers: true,
//!     ..Default::default()
//! };
//! let html = HtmlRenderer::default().render(&highlighted, &render_options);
//! ```

#![deny(missing_docs)]

mod error;
mod grammars;
mod registry;
mod scope;
mod themes;

mod highlight;
mod markdown_fence;
mod renderers;
mod tokenizer;

pub use error::Error;
pub use highlight::HighlightedText;
pub use markdown_fence::{ParsedFence, parse_markdown_fence};
pub use registry::{HighlightOptions, HighlightedCode, PLAIN_GRAMMAR_NAME, Registry};
pub use renderers::{
    RenderOptions, html::DataAttrPosition, html::ExtraHtmlContent, html::HtmlRenderer,
    terminal::TerminalRenderer,
};
pub use themes::{Color, CompiledTheme, FontStyle, Style, ThemeVariant};

/// The CSS needed for the line number gutter to display properly
pub const GIALLO_CSS: &str = r#".giallo-l {
  display: inline-block;
  min-height: 1lh;
  width: 100%;
}
.giallo-ln {
  display: inline-block;
  user-select: none;
  margin-right: 0.4em;
  padding: 0.4em;
  opacity: 0.8;
}
"#;

#[cfg(test)]
pub(crate) mod test_utils {
    use crate::Registry;
    use std::fs;

    pub fn get_registry() -> Registry {
        let mut registry = Registry::default();
        for entry in fs::read_dir("grammars-themes/packages/tm-grammars/grammars").unwrap() {
            let path = entry.unwrap().path();
            registry.add_grammar_from_path(path).unwrap();
        }
        registry.link_grammars();
        registry
            .add_theme_from_path("grammars-themes/packages/tm-themes/themes/vitesse-black.json")
            .unwrap();
        registry
    }
}
