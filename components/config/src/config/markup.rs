use serde_derive::{Deserialize, Serialize};

pub const DEFAULT_HIGHLIGHT_THEME: &str = "base16-ocean-dark";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Markdown {
    /// Whether to highlight all code blocks found in markdown files. Defaults to false
    pub highlight_code: bool,
    /// Which themes to use for code highlighting. See Readme for supported themes
    /// Defaults to "base16-ocean-dark"
    pub highlight_theme: String,
    /// Whether to render emoji aliases (e.g.: :smile: => 😄) in the markdown files
    pub render_emoji: bool,
}

impl Default for Markdown {
    fn default() -> Markdown {
        Markdown {
            highlight_code: false,
            highlight_theme: DEFAULT_HIGHLIGHT_THEME.to_owned(),
            render_emoji: false,
        }
    }
}
