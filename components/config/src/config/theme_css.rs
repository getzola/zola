use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeCss {
    /// Theme used for generating CSS
    pub theme: String,
    /// Filename for CSS
    pub filename: String,
}

impl Default for ThemeCss {
    fn default() -> ThemeCss {
        ThemeCss { theme: String::new(), filename: String::new() }
    }
}
