use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{Error, GialloResult};
use crate::themes::Color;
use crate::themes::font_style::FontStyle;
use crate::themes::raw::{RawTheme, TokenColorSettings};
use crate::themes::selector::{ThemeSelector, parse_selector};

/// Maps unique colors from a theme to sequential numeric IDs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct StyleMap {
    pub(crate) fg: HashMap<Color, u32>,
    pub(crate) bg: HashMap<Color, u32>,
}

impl StyleMap {
    pub(crate) fn new(rules: &[CompiledThemeRule], default_style: Style) -> Self {
        let mut fg_set: Vec<Color> = Vec::new();
        let mut bg_set: Vec<Color> = Vec::new();
        for rule in rules {
            if let Some(fg) = rule.style_modifier.foreground
                && fg != default_style.foreground
                && !fg_set.contains(&fg)
            {
                fg_set.push(fg);
            }
            if let Some(bg) = rule.style_modifier.background
                && bg != default_style.background
                && !bg_set.contains(&bg)
            {
                bg_set.push(bg);
            }
        }

        fg_set.sort_by_key(|a| a.as_hex());
        bg_set.sort_by_key(|a| a.as_hex());

        let mut fg = HashMap::new();
        for (idx, color) in fg_set.into_iter().enumerate() {
            fg.insert(color, (idx as u32) + 1);
        }

        let mut bg = HashMap::new();
        for (idx, color) in bg_set.into_iter().enumerate() {
            bg.insert(color, (idx as u32) + 1);
        }

        StyleMap { fg, bg }
    }

    pub(crate) fn fg_id(&self, color: Color) -> Option<u32> {
        self.fg.get(&color).copied()
    }

    pub(crate) fn bg_id(&self, color: Color) -> Option<u32> {
        self.bg.get(&color).copied()
    }
}

/// Simplified specificity calculation for theme rule sorting.
/// We use it to sort from less specific to more specific in a theme since
/// styling is meant to be inherited from parents
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Specificity {
    /// Number of atoms in target scope (most important)
    scope_depth: u32,
    /// Number of parent scopes (tie-breaker)
    parent_count: u32,
}

impl Specificity {
    fn calculate(selector: &ThemeSelector) -> Self {
        // Count atoms in target scope (dots + 1)
        let target_scope_string = selector.target_scope.build_string();
        let scope_depth = if target_scope_string.is_empty() {
            0
        } else {
            (target_scope_string.matches('.').count() + 1) as u32
        };

        let parent_count = selector.parent_scopes.len() as u32;

        Specificity {
            scope_depth,
            parent_count,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
/// A complete, concrete, style with foreground, background colors and font styling
pub struct Style {
    /// The foreground color
    pub foreground: Color,
    /// The background color
    pub background: Color,
    /// Any associated font style
    pub font_style: FontStyle,
}

impl Default for Style {
    fn default() -> Style {
        Style {
            foreground: Color::BLACK,
            background: Color::WHITE,
            font_style: FontStyle::default(),
        }
    }
}

impl Style {
    pub(crate) fn has_decorations(&self) -> bool {
        self.font_style.contains(FontStyle::UNDERLINE)
            || self.font_style.contains(FontStyle::STRIKETHROUGH)
    }
}

/// A style modifier with optional values for theme parsing
///
/// This represents theme entries where colors and font styles are optional.
/// Used during theme loading and then resolved to concrete Style values.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct StyleModifier {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub font_style: Option<FontStyle>,
}

impl TryFrom<TokenColorSettings> for StyleModifier {
    type Error = Error;

    fn try_from(settings: TokenColorSettings) -> GialloResult<Self> {
        let foreground = if let Some(s) = settings.foreground() {
            Some(Color::from_hex(s)?)
        } else {
            None
        };
        let background = if let Some(s) = settings.background() {
            Some(Color::from_hex(s)?)
        } else {
            None
        };

        let font_style = settings.font_style.map(|s| FontStyle::from_theme_str(&s));

        Ok(Self {
            foreground,
            background,
            font_style,
        })
    }
}

impl StyleModifier {
    /// Apply this style modifier to a base style, creating a new style
    pub fn apply_to(&self, base: &Style) -> Style {
        Style {
            foreground: self.foreground.unwrap_or(base.foreground),
            background: self.background.unwrap_or(base.background),
            font_style: self.font_style.unwrap_or(base.font_style),
        }
    }
}

/// Theme type for determining fallback colors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemeType {
    Light,
    #[default]
    Dark,
}

impl ThemeType {
    pub fn from_theme_str(s: &str) -> ThemeType {
        if s.eq_ignore_ascii_case("light") {
            ThemeType::Light
        } else {
            ThemeType::Dark
        }
    }
}

/// Compiled theme rule for efficient matching
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledThemeRule {
    pub selector: ThemeSelector,
    pub style_modifier: StyleModifier,
}

/// Compiled theme optimized for fast lookups
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledTheme {
    /// The name of the theme
    pub name: String,
    /// Theme type ("light" or "dark")
    pub(crate) theme_type: ThemeType,
    /// Default style for tokens with no specific rules
    pub default_style: Style,
    /// Value of `editor.lineHighlightBackground`
    pub highlight_background_color: Option<Color>,
    /// Value of `editorLineNumber.foreground`
    pub line_number_foreground: Option<Color>,
    /// Theme rules sorted by specificity (most specific first)
    pub(crate) rules: Vec<CompiledThemeRule>,
    /// Color -> class name for CSS output
    pub(crate) style_map: StyleMap,
}

impl CompiledTheme {
    pub(crate) fn from_raw_theme(raw_theme: RawTheme) -> GialloResult<Self> {
        let theme_type = raw_theme
            .kind
            .map(|s| ThemeType::from_theme_str(&s))
            .unwrap_or_default();

        let foreground = Color::from_hex(&raw_theme.colors.foreground)?;
        let background = Color::from_hex(&raw_theme.colors.background)?;
        let highlight_background_color = if let Some(bg) = raw_theme.colors.highlight_background {
            Some(Color::from_hex(&bg)?)
        } else {
            None
        };
        let line_number_foreground = if let Some(fg) = raw_theme.colors.line_number_foreground {
            Some(Color::from_hex(&fg)?)
        } else {
            None
        };

        let mut default_style = Style {
            foreground,
            background,
            font_style: FontStyle::default(),
        };

        let mut rules_with_specificity = Vec::new();

        for token_rule in raw_theme.token_colors {
            // Special case: some themes have defaults in token colors:
            // eg andromeeda
            //     {
            //       "settings": {
            //         "background": "#23262E",
            //         "foreground": "#D5CED9"
            //       }
            //     },
            if token_rule.scope.is_empty() {
                if let Some(fg) = token_rule.settings.foreground() {
                    default_style.foreground = Color::from_hex(fg)?;
                }
                if let Some(bg) = token_rule.settings.background() {
                    default_style.background = Color::from_hex(bg)?;
                }
                continue;
            }

            let mut selectors = Vec::new();

            for scope_pattern in &token_rule.scope {
                if let Some(selector) = parse_selector(scope_pattern) {
                    selectors.push(selector);
                } else {
                    #[cfg(feature = "debug")]
                    log::warn!(
                        "Failed to parse theme selector: '{scope_pattern}' in theme {}",
                        raw_theme.name
                    );
                }
            }

            if !selectors.is_empty() {
                let style_modifier = StyleModifier::try_from(token_rule.settings.clone())?;

                for selector in selectors {
                    let specificity = Specificity::calculate(&selector);
                    rules_with_specificity.push((
                        CompiledThemeRule {
                            selector,
                            style_modifier,
                        },
                        specificity,
                    ));
                }
            }
        }

        rules_with_specificity.sort_by(|a, b| a.1.cmp(&b.1));
        // and then we discard specificity, we don't need it anymore
        let rules: Vec<CompiledThemeRule> = rules_with_specificity
            .into_iter()
            .map(|(rule, _)| rule)
            .collect();

        let style_map = StyleMap::new(&rules, default_style);

        Ok(CompiledTheme {
            name: raw_theme.name,
            theme_type,
            default_style,
            highlight_background_color,
            line_number_foreground,
            rules,
            style_map,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn can_load_and_compile_all_shiki_themes() {
        let entries = fs::read_dir("grammars-themes/packages/tm-themes/themes")
            .expect("Failed to read grammars directory");

        for entry in entries {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();
            println!("{:?}", path);
            RawTheme::load_from_file(&path)
                .unwrap()
                .compile()
                .unwrap_or_else(|_| panic!("Failed to compile theme: {path:?}"));
        }
    }

    #[test]
    fn can_load_default_from_token_colors() {
        let theme = RawTheme::load_from_file("src/fixtures/themes/all_scope_styles.json").unwrap();
        let compiled = CompiledTheme::from_raw_theme(theme).unwrap();
        assert_eq!(compiled.default_style.background.as_hex(), "#23262E");
        assert_eq!(compiled.default_style.foreground.as_hex(), "#D5CED9");
    }
}
