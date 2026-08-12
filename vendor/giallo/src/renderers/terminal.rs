use crate::{HighlightedCode, RenderOptions, themes::compiled::ThemeType};

/// Terminal renderer via ANSI escape codes. Requires a terminal that supports truecolor
#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub struct TerminalRenderer {
    /// The theme type to use if [`ThemeVariant::Dual`](crate::ThemeVariant::Dual) is provided,
    /// since terminals don't allow light or dark theme
    pub theme_type: Option<ThemeType>,
}

impl TerminalRenderer {
    /// Render to the terminal with ANSI escape codes
    pub fn render(&self, highlighted: &HighlightedCode, options: &RenderOptions) -> String {
        let mut output = String::new();
        let line_numbers_size = options.line_number_width(highlighted.tokens.len());

        // Color of line numbers
        let (line_number_foreground, highlight_background_color) = match highlighted.theme {
            crate::ThemeVariant::Single(theme) => (
                theme.line_number_foreground,
                theme.highlight_background_color,
            ),
            crate::ThemeVariant::Dual { light, .. }
                if self.theme_type == Some(ThemeType::Light) =>
            {
                (
                    light.line_number_foreground,
                    light.highlight_background_color,
                )
            }
            crate::ThemeVariant::Dual { dark, .. } if self.theme_type == Some(ThemeType::Dark) => {
                (dark.line_number_foreground, dark.highlight_background_color)
            }
            _ => unreachable!(),
        };

        let line_count = highlighted.tokens.len();
        let mut tokens = highlighted.tokens.iter().enumerate().peekable();
        while let Some((idx, line_tokens)) = tokens.next() {
            let line_num = idx + 1; // 1-indexed
            let is_last_line = line_count == line_num;

            // Special case: If the current line is the last newline of the file,
            // then don't render it. This matches the behaviour of "cat" and "bat"
            if tokens.peek().is_none() && line_tokens.is_empty() {
                continue;
            }
            // Semantically, it's as if this newline is being added at the end of each iteration.
            // But if the previous condition fires, then we don't want the newline to have been added.
            else if idx != 0 && !is_last_line {
                output.push('\n');
            }

            // Skip hidden lines
            if options.hide_lines.iter().any(|r| r.contains(&line_num)) {
                continue;
            }

            let is_highlighted = options
                .highlight_lines
                .iter()
                .any(|r| r.contains(&line_num));

            if options.show_line_numbers {
                let line_num = options.line_number_start + (idx as isize);
                let line_num_s = line_num.to_string();
                let s = std::iter::repeat_n(' ', line_numbers_size - line_num_s.chars().count())
                    .chain(line_num_s.chars())
                    .collect::<String>();
                if let Some(line_number_foreground) = line_number_foreground {
                    output.push_str("\x1b[");
                    line_number_foreground.as_ansi_fg(&mut output);
                    output.push('m');
                }
                output.push_str(&format!("  {s} "));
                if line_number_foreground.is_some() {
                    // reset
                    output.push_str("\x1b[0m");
                }
            }

            // Highlight individual tokens
            for token in line_tokens {
                token.as_ansi(
                    &highlighted.theme,
                    self.theme_type,
                    highlight_background_color.filter(|_| is_highlighted),
                    &mut output,
                )
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ThemeVariant;
    use crate::registry::HighlightOptions;
    use crate::test_utils::get_registry;

    #[test]
    fn test_highlight_and_hide_lines() {
        let registry = get_registry();
        let code = "let a = 1;\nlet b = 2;\nlet c = 3;\nlet d = 4;\nlet e = 5;";
        let options = HighlightOptions::new("javascript", ThemeVariant::Single("vitesse-black"));
        let highlighted = registry.highlight(code, &options).unwrap();

        let render_options = RenderOptions {
            show_line_numbers: true,
            line_number_start: 10,
            highlight_lines: vec![2..=2, 4..=4],
            hide_lines: vec![3..=3],
        };

        let ansi = TerminalRenderer::default().render(&highlighted, &render_options);

        insta::assert_snapshot!(ansi);
    }
}
