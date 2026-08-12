mod color;
pub(crate) mod compiled;
pub(crate) mod css;
pub(crate) mod font_style;
pub(crate) mod raw;
pub(crate) mod selector;

use serde::{Deserialize, Serialize};

pub use color::Color;
pub use compiled::{CompiledTheme, Style};
pub use font_style::FontStyle;
pub use raw::RawTheme;

/// Generic enum for single or dual (light/dark) theme values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeVariant<T> {
    /// A single theme was requested
    Single(T),
    /// Light/dark themes were requested
    Dual {
        /// The light theme
        light: T,
        /// The dark theme
        dark: T,
    },
}

impl ThemeVariant<Style> {
    pub(crate) fn has_decoration(&self) -> bool {
        match self {
            Self::Single(style) => style.has_decorations(),
            Self::Dual { light, dark } => light.has_decorations() || dark.has_decorations(),
        }
    }
}
