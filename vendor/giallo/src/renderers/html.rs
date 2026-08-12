use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::registry::HighlightedCode;
use crate::renderers::RenderOptions;
use crate::themes::css::{DARK_SUFFIX, LIGHT_SUFFIX};
use crate::themes::{Color, ThemeVariant};

/// Where to put the additional attributes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DataAttrPosition {
    /// On the <pre> only
    Pre,
    #[default]
    /// On the <code> only
    Code,
    /// On both <pre> and <code>
    Both,
    /// Nowhere
    None,
}

/// Extra HTML content to include in the `<pre>` block.
///
/// This may be used to add extra UI elements to highlighted code blocks, e.g. a "Copy" button.
#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct ExtraHtmlContent {
    /// Additional HTML to insert **before** the `<code>` element.
    ///
    /// This string will _not_ be escaped automatically by [`HtmlRenderer`]. The caller is
    /// responsible for ensuring the correctness of the final HTML output.
    pub before: Option<String>,
    /// Additional HTML to insert **after** the `<code>` element.
    ///
    /// This string will _not_ be escaped automatically by [`HtmlRenderer`]. The caller is
    /// responsible for ensuring the correctness of the final HTML output.
    pub after: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
/// A renderer that will output proper HTML code
pub struct HtmlRenderer {
    /// Any metadata we want to add as `<code>` data-* attribute
    pub other_metadata: BTreeMap<String, String>,
    /// Where to put the data attributes on the code blocks
    pub data_attr_position: DataAttrPosition,
    /// If set, output CSS classes instead of inline styles.
    /// The value is the class prefix (e.g., "g-" produces classes like "g-keyword").
    /// Generate corresponding CSS stylesheets using `Registry::generate_css`.
    pub css_class_prefix: Option<String>,
    /// Any extra HTML content to add before or after the `<code>` element
    pub extra_html_content: ExtraHtmlContent,
    /// If css output should set color-scheme when using a dual theme
    ///
    /// The default is `true`, which emits an inline `color-scheme` rule for every code block.
    ///
    /// If set to `false`, no `color-scheme` rule will be emitted.
    /// Use this option if you set `color-scheme` elsewhere and want it to apply to code blocks.
    pub add_color_scheme: bool,
}

impl Default for HtmlRenderer {
    fn default() -> Self {
        Self {
            other_metadata: Default::default(),
            data_attr_position: Default::default(),
            css_class_prefix: Default::default(),
            extra_html_content: Default::default(),
            add_color_scheme: true,
        }
    }
}

impl HtmlRenderer {
    /// Renders the given highlighted code to an HTML string.
    /// This will also handle automatic light/dark theming and escaping characters.
    pub fn render(&self, highlighted: &HighlightedCode, options: &RenderOptions) -> String {
        let lang = highlighted.language;
        let css_prefix = self.css_class_prefix.as_deref();

        // Precompute structural CSS classes (code, hl).
        // For dual themes, we insert both light and dark classes
        let (code_class, hl_class) = match css_prefix {
            Some(p) => match &highlighted.theme {
                ThemeVariant::Single(_) => (Some(format!("{p}code")), format!("{p}hl")),
                ThemeVariant::Dual { .. } => (
                    Some(format!("{p}{LIGHT_SUFFIX}code {p}{DARK_SUFFIX}code")),
                    format!("{p}{LIGHT_SUFFIX}hl {p}{DARK_SUFFIX}hl"),
                ),
            },
            None => (None, String::new()),
        };

        // Pre-compute highlight background CSS/class if available
        let highlight_attr = if !options.highlight_lines.is_empty() {
            if css_prefix.is_some() {
                // CSS class mode: use hl class
                Some(format!(r#" class="{hl_class}""#))
            } else {
                // Inline style mode
                match &highlighted.theme {
                    ThemeVariant::Single(theme) => theme
                        .highlight_background_color
                        .as_ref()
                        .map(|c| format!(r#" style="{}""#, c.as_css_bg_color_property())),
                    ThemeVariant::Dual { light, dark } => {
                        match (
                            &light.highlight_background_color,
                            &dark.highlight_background_color,
                        ) {
                            (Some(l), Some(d)) => Some(format!(
                                r#" style="{}""#,
                                Color::as_css_light_dark_bg_color_property(l, d)
                            )),
                            _ => None,
                        }
                    }
                }
            }
        } else {
            None
        };

        // Pre-compute line number color style if available (inline style mode only)
        let line_number_style = if options.show_line_numbers && css_prefix.is_none() {
            match &highlighted.theme {
                ThemeVariant::Single(theme) => theme
                    .line_number_foreground
                    .as_ref()
                    .map(|c| format!(r#" style="{}""#, c.as_css_color_property())),
                ThemeVariant::Dual { light, dark } => {
                    match (&light.line_number_foreground, &dark.line_number_foreground) {
                        (Some(l), Some(d)) => Some(format!(
                            r#" style="{}""#,
                            Color::as_css_light_dark_color_property(l, d)
                        )),
                        _ => None,
                    }
                }
            }
        } else {
            None
        };

        let line_numbers_size = options.line_number_width(highlighted.tokens.len());
        let mut lines = Vec::with_capacity(highlighted.tokens.len() + 4);
        let mut tokens = highlighted.tokens.iter().enumerate().peekable();
        while let Some((idx, line_tokens)) = tokens.next() {
            let line_num = idx + 1; // 1-indexed

            // Skip trailing empty line
            if tokens.peek().is_none() && line_tokens.is_empty() {
                continue;
            }

            // Skip hidden lines
            if options.hide_lines.iter().any(|r| r.contains(&line_num)) {
                continue;
            }

            // Render tokens
            let mut line_content = Vec::with_capacity(line_tokens.len());
            for tok in line_tokens {
                line_content.push(tok.as_html(&highlighted.theme, css_prefix));
            }
            let line_content = line_content.join("");

            // Line number (uses original source line number, padded with spaces)
            let display_line_num = options.line_number_start + (idx as isize);
            let line_number_html = if options.show_line_numbers {
                let line_num_s = display_line_num.to_string();
                let padded =
                    std::iter::repeat_n(' ', line_numbers_size - line_num_s.chars().count())
                        .chain(line_num_s.chars())
                        .collect::<String>();
                format!(
                    r#"<span aria-hidden="true" class="giallo-ln"{}>{padded}</span>"#,
                    line_number_style.as_deref().unwrap_or_default()
                )
            } else {
                String::new()
            };

            // Build line span, with highlight if applicable
            let is_highlighted = options
                .highlight_lines
                .iter()
                .any(|r| r.contains(&line_num));
            let line_html = match (is_highlighted, &highlight_attr) {
                (true, Some(hl_class_or_style)) => {
                    format!(
                        r#"<span class="giallo-l{hl_class_or_style}"{hl_style}>{line_number_html}{line_content}</span>"#,
                        hl_class_or_style = if css_prefix.is_some() {
                            format!(" {hl_class}")
                        } else {
                            String::new()
                        },
                        hl_style = if css_prefix.is_none() {
                            hl_class_or_style
                        } else {
                            ""
                        }
                    )
                }
                _ => format!(r#"<span class="giallo-l">{line_number_html}{line_content}</span>"#),
            };

            lines.push(line_html);
        }
        let lines = lines.join("\n");

        // Build data attributes from other_metadata
        let mut data_attrs = format!(r#"data-lang="{lang}""#);
        for (key, value) in &self.other_metadata {
            // lowercase and replace non-alphanumeric chars with hyphens
            let slugified_key: String = key
                .to_lowercase()
                .chars()
                .map(|c| {
                    if c.is_alphanumeric() || c == '-' {
                        c
                    } else {
                        '-'
                    }
                })
                .collect();
            data_attrs.push_str(&format!(r#" data-{slugified_key}="{value}""#));
        }
        let pre_data_attrs = match self.data_attr_position {
            DataAttrPosition::Pre | DataAttrPosition::Both => &data_attrs,
            _ => "",
        };
        let code_data_attrs = match self.data_attr_position {
            DataAttrPosition::Code | DataAttrPosition::Both => &data_attrs,
            _ => "",
        };

        // Additional HTML to inject into `<pre>` block
        let before_code_html = self
            .extra_html_content
            .before
            .as_deref()
            .unwrap_or_default();
        let after_code_html = self.extra_html_content.after.as_deref().unwrap_or_default();

        // CSS class mode: output class instead of inline styles on <pre>
        if let Some(code_class) = &code_class {
            return format!(
                r#"<pre class="giallo {code_class}" {pre_data_attrs}>{before_code_html}<code {code_data_attrs}>{lines}</code>{after_code_html}</pre>"#
            );
        }

        // Inline style mode
        match &highlighted.theme {
            ThemeVariant::Single(theme) => {
                let fg = theme.default_style.foreground.as_css_color_property();
                let bg = theme.default_style.background.as_css_bg_color_property();
                format!(
                    r#"<pre class="giallo" style="{fg} {bg}" {pre_data_attrs}>{before_code_html}<code {code_data_attrs}>{lines}</code>{after_code_html}</pre>"#
                )
            }
            ThemeVariant::Dual { light, dark } => {
                let fg = Color::as_css_light_dark_color_property(
                    &light.default_style.foreground,
                    &dark.default_style.foreground,
                );
                let bg = Color::as_css_light_dark_bg_color_property(
                    &light.default_style.background,
                    &dark.default_style.background,
                );
                let color_scheme = if self.add_color_scheme {
                    "color-scheme: light dark; "
                } else {
                    ""
                };
                format!(
                    r#"<pre class="giallo" style="{color_scheme}{fg} {bg}" {pre_data_attrs}>{before_code_html}<code {code_data_attrs}>{lines}</code>{after_code_html}</pre>"#
                )
            }
        }
    }
}

// From syntect
pub(crate) struct HtmlEscaped<'a>(pub &'a str);
impl fmt::Display for HtmlEscaped<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Because the internet is always right, turns out there's not that many
        // characters to escape: http://stackoverflow.com/questions/7381974
        let Self(s) = *self;
        let pile_o_bits = s;
        let mut last = 0;
        for (i, ch) in s.bytes().enumerate() {
            match ch as char {
                '<' | '>' | '&' | '\'' | '"' => {
                    fmt.write_str(&pile_o_bits[last..i])?;
                    let s = match ch as char {
                        '>' => "&gt;",
                        '<' => "&lt;",
                        '&' => "&amp;",
                        '\'' => "&#39;",
                        '"' => "&quot;",
                        _ => unreachable!(),
                    };
                    fmt.write_str(s)?;
                    last = i + 1;
                }
                _ => {}
            }
        }

        if last < s.len() {
            fmt.write_str(&pile_o_bits[last..])?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::HighlightOptions;
    use crate::test_utils::get_registry;

    #[test]
    fn test_highlight_and_hide_lines() {
        let registry = get_registry();
        let code = "let a = 1;\n\nlet b = 2;\nlet c = 3;\nlet d = 4;\nlet e = 5;\n";
        let options = HighlightOptions::new("javascript", ThemeVariant::Single("vitesse-black"));
        let highlighted = registry.highlight(code, &options).unwrap();

        let render_options = RenderOptions {
            show_line_numbers: true,
            line_number_start: 10,
            highlight_lines: vec![3..=3, 5..=5],
            hide_lines: vec![4..=4],
        };

        let mut other_metadata = BTreeMap::new();
        other_metadata.insert("copy".to_string(), "true".to_string());
        other_metadata.insert("name".to_string(), "Hello world".to_string());
        other_metadata.insert("name with space1".to_string(), "other".to_string());

        let html = HtmlRenderer {
            other_metadata: other_metadata.clone(),
            css_class_prefix: None,
            ..Default::default()
        }
        .render(&highlighted, &render_options);
        insta::assert_snapshot!(html);

        let html = HtmlRenderer {
            other_metadata: other_metadata.clone(),
            css_class_prefix: None,
            data_attr_position: DataAttrPosition::Both,
            ..Default::default()
        }
        .render(&highlighted, &render_options);
        insta::assert_snapshot!(html);

        let html = HtmlRenderer {
            other_metadata: other_metadata.clone(),
            css_class_prefix: None,
            data_attr_position: DataAttrPosition::None,
            ..Default::default()
        }
        .render(&highlighted, &render_options);
        insta::assert_snapshot!(html);

        let html = HtmlRenderer {
            other_metadata: other_metadata.clone(),
            css_class_prefix: None,
            extra_html_content: ExtraHtmlContent {
                before: Some("<span>javascript</span>".to_owned()),
                after: None,
            },
            ..Default::default()
        }
        .render(&highlighted, &render_options);
        insta::assert_snapshot!(html);

        let html = HtmlRenderer {
            other_metadata,
            css_class_prefix: None,
            extra_html_content: ExtraHtmlContent {
                before: Some(
                    "<span><span>javascript</span><button>Copy</button></span>".to_owned(),
                ),
                after: Some("<span>index.js</span>".to_owned()),
            },
            ..Default::default()
        }
        .render(&highlighted, &render_options);
        insta::assert_snapshot!(html);
    }
}
