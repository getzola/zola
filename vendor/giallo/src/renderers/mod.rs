use std::ops::RangeInclusive;

pub mod html;
pub mod terminal;

#[derive(Clone, Debug, PartialEq, Eq)]
/// All options you can select across renderers
pub struct RenderOptions {
    /// Whether to show the line numbers in a gutter. Defaults to false.
    pub show_line_numbers: bool,
    /// At which number do the line numbering start. Defaults to 1.
    pub line_number_start: isize,
    /// Which lines to highlight. Lines start from 1, not 0.
    /// If the selected theme doesn't have a highlight colour, this is a noop.
    pub highlight_lines: Vec<RangeInclusive<usize>>,
    /// Which lines to not render. Lines start from 1, not 0.
    pub hide_lines: Vec<RangeInclusive<usize>>,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            show_line_numbers: false,
            line_number_start: 1,
            highlight_lines: Vec::new(),
            hide_lines: Vec::new(),
        }
    }
}

impl RenderOptions {
    /// Calculate the width needed for line numbers
    pub(crate) fn line_number_width(&self, line_count: usize) -> usize {
        if !self.show_line_numbers {
            return 0;
        }
        let first_line = self.line_number_start.to_string().chars().count();
        let last_line = line_count
            .saturating_add_signed(self.line_number_start)
            .to_string()
            .chars()
            .count();
        first_line.max(last_line)
    }
}
