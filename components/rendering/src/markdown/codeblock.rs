use syntect::html::{IncludeBackground, styled_line_to_highlighted_html};
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{Color, Theme, Style};
use config::Config;
use config::highlighting::{get_highlighter, SYNTAX_SET, THEME_SET};
use std::cmp::min;
use std::collections::HashSet;

use super::fence::FenceSettings;

pub struct CodeBlock<'config> {
    highlighter: HighlightLines<'static>,
    extra_syntax_set: Option<&'config SyntaxSet>,
    background: IncludeBackground,
    theme: &'static Theme,

    highlight_lines: Vec<Range>,
    num_lines: usize,
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
            num_lines: 0,
        }
    }

    pub fn highlight(&mut self, text: &str) -> String {
        let highlighted = self.highlighter.highlight(
            text,
            self.extra_syntax_set.unwrap_or(&SYNTAX_SET),
        );
        let line_boundaries = self.find_line_boundaries(&highlighted);

        // First we make sure that `highlighted` is split at every line
        // boundary. The `styled_line_to_highlighted_html` function will
        // merge split items with identical styles, so this is not a
        // problem.
        //
        // Note that this invalidates the values in `line_boundaries`.
        // The `perform_split` function takes it by value to ensure that
        // we don't use it later.
        let mut highlighted = perform_split(&highlighted, line_boundaries);

        let hl_background = self.theme.settings.line_highlight
                .unwrap_or(Color { r: 255, g: 255, b: 0, a: 0 });


        let hl_lines = self.get_highlighted_lines();
        color_highlighted_lines(&mut highlighted, &hl_lines, hl_background);

        styled_line_to_highlighted_html(&highlighted, self.background)
    }

    fn find_line_boundaries(&mut self, styled: &[(Style, &str)]) -> Vec<StyledIdx> {
        let mut boundaries = Vec::new();
        for (vec_idx, (_style, s)) in styled.iter().enumerate() {
            for (str_idx, character) in s.char_indices() {
                if character == '\n' {
                    boundaries.push(StyledIdx {
                        vec_idx,
                        str_idx,
                    });
                }
            }
        }
        self.num_lines = boundaries.len() + 1;
        boundaries
    }

    fn get_highlighted_lines(&self) -> HashSet<usize> {
        let mut lines = HashSet::new();
        for range in &self.highlight_lines {
            for line in range.from ..= min(range.to, self.num_lines) {
                // Ranges are one-indexed
                lines.insert(line.saturating_sub(1));
            }
        }
        lines
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Range {
    pub from: usize,
    pub to: usize,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct StyledIdx {
    vec_idx: usize,
    str_idx: usize,
}

fn get_str_if_vec_is(idx: Option<&StyledIdx>, vec_idx: usize) -> Option<usize> {
    match idx {
        Some(idx) if idx.vec_idx == vec_idx => Some(idx.str_idx),
        _ => None,
    }
}

// This function assumes that the line boundaries is sorted.
fn perform_split<'b>(
    split: &[(Style, &'b str)],
    line_boundaries: Vec<StyledIdx>
) -> Vec<(Style, &'b str)> {
    let mut result = Vec::new();

    let mut idxs_iter = line_boundaries.into_iter().peekable();

    for (split_idx, item) in split.iter().enumerate() {
        let mut last_split = 0;

        // For each split in this item, append the left half of that split.
        while let Some(str_idx) = get_str_if_vec_is(idxs_iter.peek(), split_idx) {
            // Consume the value we just peeked.
            idxs_iter.next();

            // We want to include the newline together with its own line.
            let split_at = min(str_idx + 1, item.1.len());

            debug_assert!(split_at >= last_split);
            if last_split != split_at {
                result.push((item.0, &item.1[last_split..split_at]));
                last_split = split_at;
            }
        }

        // Now append the remainder.
        if last_split != item.1.len() {
            result.push((item.0, &item.1[last_split..]));
        }
    }

    result
}

fn color_highlighted_lines(
    data: &mut [(Style, &str)],
    lines: &HashSet<usize>,
    background: Color,
) {
    if lines.is_empty() {
        return;
    }

    let mut current_line = 0;

    for item in data {
        if lines.contains(&current_line) {
            item.0.background = background;
        }

        // We split the lines such that every newline is at the end of an item.
        if item.1.ends_with('\n') {
            current_line += 1;
        }
    }
}
