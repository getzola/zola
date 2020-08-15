use syntect::html::{IncludeBackground, styled_line_to_highlighted_html};
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{Color, Theme};
use config::Config;
use config::highlighting::{get_highlighter, SYNTAX_SET, THEME_SET};

use super::fence::FenceSettings;

pub struct CodeBlock<'config> {
    highlighter: HighlightLines<'static>,
    extra_syntax_set: Option<&'config SyntaxSet>,
    background: IncludeBackground,
    theme: &'static Theme,

    highlight_lines: Vec<Range>,
    next_line_number: usize,
}

impl<'config> CodeBlock<'config> {
    pub fn new(
        fence_info: &str,
        config: &'config Config,
        background: IncludeBackground,
    ) -> Self {
        let fence_info = FenceSettings::new(fence_info);
        let theme = &THEME_SET.themes[&config.highlight_theme];
        let (highlighter, in_extra) = get_highlighter(fence_info.language, config);
        Self {
            highlighter,
            extra_syntax_set: match in_extra {
                true => config.extra_syntax_set.as_ref(),
                false => None,
            },
            background,
            theme,

            highlight_lines: fence_info.highlight_lines,
            next_line_number: 1,
        }
    }

    pub fn highlight(&mut self, text: &str) -> String {
        let highlighted = self.highlighter.highlight(
            text,
            self.extra_syntax_set.unwrap_or(&SYNTAX_SET),
        );

        let background = if self.is_line_highlighted() {
            let color = self.theme.settings.line_highlight
                .unwrap_or(Color { r: 0xff, g: 0xff, b: 0, a: 0 });
            IncludeBackground::IfDifferent(color)
        } else {
            self.background
        };

        self.next_line_number += 1;
        styled_line_to_highlighted_html(&highlighted, background)
    }

    fn is_line_highlighted(&self) -> bool {
        for range in &self.highlight_lines {
            if range.contains(self.next_line_number) {
                return true;
            }
        }
        false
    }
}

#[derive(Copy, Clone)]
pub struct Range {
    pub from: usize,
    pub to: usize,
}

impl Range {
    pub fn contains(self, value: usize) -> bool {
        self.from <= value && value <= self.to
    }
}
