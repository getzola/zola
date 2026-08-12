use std::fmt;
use std::fs::File;
use std::path::Path;

use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer, de};

use crate::error::GialloResult;
use crate::themes::compiled::CompiledTheme;

/// Token color settings from VSCode theme JSON
#[derive(Debug, Clone, Deserialize, Default)]
pub struct TokenColorSettings {
    pub foreground: Option<String>,
    pub background: Option<String>,
    #[serde(rename = "fontStyle")]
    pub font_style: Option<String>,
}

impl TokenColorSettings {
    pub fn foreground(&self) -> Option<&str> {
        self.foreground.as_deref().filter(|s| *s != "inherit")
    }

    pub fn background(&self) -> Option<&str> {
        self.background.as_deref().filter(|s| *s != "inherit")
    }
}

/// Custom deserializer for scope field that can be a string or an array of string
fn deserialize_string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct ScopeVisitor;

    impl<'de> Visitor<'de> for ScopeVisitor {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or array of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.split(',').map(|s| s.trim().to_string()).collect())
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(item) = seq.next_element::<String>()? {
                vec.push(item);
            }
            Ok(vec)
        }
    }

    deserializer.deserialize_any(ScopeVisitor)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Colors {
    pub foreground: String,
    pub background: String,
    pub highlight_background: Option<String>,
    pub line_number_foreground: Option<String>,
}

// Some themes have it as editor.foreground/background some don't have the `editor.` prefix
impl<'de> Deserialize<'de> for Colors {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ColorsVisitor;

        impl<'de> Visitor<'de> for ColorsVisitor {
            type Value = Colors;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Colors")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Colors, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut foreground = None;
                let mut background = None;
                let mut highlight_background = None;
                let mut line_number_foreground = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "foreground" | "editor.foreground" => {
                            if foreground.is_none() {
                                foreground = Some(map.next_value()?);
                            } else {
                                // Skip the value if we already have one
                                let _: de::IgnoredAny = map.next_value()?;
                            }
                        }
                        "background" | "editor.background" => {
                            if background.is_none() {
                                background = Some(map.next_value()?);
                            } else {
                                // Skip the value if we already have one
                                let _: de::IgnoredAny = map.next_value()?;
                            }
                        }
                        "editor.lineHighlightBackground" => {
                            highlight_background = Some(map.next_value()?);
                        }
                        "editorLineNumber.foreground" => {
                            line_number_foreground = Some(map.next_value()?);
                        }
                        _ => {
                            // Skip unknown fields
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let foreground = foreground
                    .ok_or_else(|| de::Error::missing_field("foreground or editor.foreground"))?;
                let background = background
                    .ok_or_else(|| de::Error::missing_field("background or editor.background"))?;

                Ok(Colors {
                    foreground,
                    background,
                    highlight_background,
                    line_number_foreground,
                })
            }
        }

        deserializer.deserialize_map(ColorsVisitor)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenColorRule {
    #[serde(deserialize_with = "deserialize_string_or_vec", default)]
    pub scope: Vec<String>,
    #[serde(default)]
    pub settings: TokenColorSettings,
}

/// Raw theme loaded from a JSON theme file
#[derive(Debug, Clone, Deserialize)]
pub struct RawTheme {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub colors: Colors,
    /// Token color rules for syntax highlighting
    #[serde(rename = "tokenColors")]
    pub token_colors: Vec<TokenColorRule>,
}

impl RawTheme {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> GialloResult<Self> {
        let file = File::open(path)?;
        let theme = serde_json::from_reader(file)?;
        Ok(theme)
    }

    /// Compile this raw grammar into an optimized compiled grammar
    pub fn compile(self) -> GialloResult<CompiledTheme> {
        CompiledTheme::from_raw_theme(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_all_kinds_of_scope() {
        let theme = RawTheme::load_from_file("src/fixtures/themes/all_scope_styles.json").unwrap();

        assert_eq!(theme.name, "test");
        assert_eq!(theme.token_colors.len(), 5);

        // Expected scope parsing results for different formats
        let expected_scopes = [
            // Rule 0: No scope (default/fallback rule)
            vec![],
            // Rule 1: Array format with 2 scopes
            vec!["comment", "markup.quote.markdown"],
            // Rule 2: Comma-separated string format with 3 scopes
            vec![
                "variable.language.this",
                "variable.language.self",
                "variable.language.super",
            ],
            // Rule 3: Array format with >
            vec!["string > source", "string embedded"],
            // Rule 4: String format with comma-separated scopes with >
            vec!["string > source", "string embedded"],
        ];

        // Expected foreground colors
        let expected_foregrounds = [
            Some("#D5CED9"),   // Rule 0: default/fallback rule with foreground
            Some("#A0A1A7cc"), // Rule 1: comment scopes
            Some("#d699b6"),   // Rule 2: language variables
            Some("#383A42"),   // Rule 3: string sources (array format)
            Some("#383A42"),   // Rule 4: string sources (string format)
        ];

        // Test scope parsing and color settings for each rule
        for (i, (expected_scope, expected_fg)) in expected_scopes
            .iter()
            .zip(expected_foregrounds.iter())
            .enumerate()
        {
            let rule = &theme.token_colors[i];

            // Check scope parsing
            assert_eq!(
                rule.scope.len(),
                expected_scope.len(),
                "Rule {} scope count mismatch",
                i
            );
            assert_eq!(
                rule.scope, *expected_scope,
                "Rule {} scope content mismatch",
                i
            );

            // Check foreground color
            assert_eq!(
                rule.settings.foreground(),
                *expected_fg,
                "Rule {} foreground color mismatch",
                i
            );
        }

        // Test that the theme can be compiled successfully
        let compiled_theme = theme.compile().expect("Failed to compile test theme");

        // Verify the compiled theme has the expected name
        assert_eq!(compiled_theme.name, "test");
    }
}
