use std::ops::RangeInclusive;

use super::CodeBlockPass;

pub struct HighlightLines<T> {
    hl_lines: Vec<RangeInclusive<usize>>,
    mark_open: bool,
    next: T
}
impl<T: Sized> HighlightLines<T> {
    pub fn new(hl_lines: Vec<RangeInclusive<usize>>, next: T) -> Self {
        Self {
            hl_lines,
            mark_open: false,
            next
        }
    }
    fn is_line_highlighted(&self, line_num: usize) -> bool {
        self.hl_lines.iter().any(|range| {
            // TODO: Don't check every range every line, and prune ranges that end before line_num
            range.contains(&line_num)
        })
    }
}
impl<T: CodeBlockPass> CodeBlockPass for HighlightLines<T> {
    fn close_to_root(&mut self, output: &mut String) {
        self.next.close_to_root(output);
        
        if self.mark_open {
            output.push_str("</mark>");
            self.mark_open = false;
        }
    }
    fn handle_line(&mut self, output: &mut String, line_num: usize, input: &str) {
        let is_highlighted = self.is_line_highlighted(line_num);
        if is_highlighted != self.mark_open {
            if !self.mark_open {
                self.next.close_to_root(output);
                // TODO: Add inline styling to the mark
                output.push_str("<mark");
                if let Some(styles) = self.next.mark_styles() {
                    output.push_str(" style=\"");
                    output.push_str(styles.as_str());
                    output.push('"');
                }
                output.push('>');
                self.mark_open = true;
            } else {
                self.close_to_root(output);
            }
        }
		self.next.handle_line(output, line_num, input);
    }
    // Pass Overrides up to parent passes
    fn pre_styles(&self, line_num: usize) -> Option<String> {
        self.next.pre_styles(line_num)
    }
    fn pre_class(&self) -> Option<String> {
        self.next.pre_class()
    }
    fn mark_styles(&self) -> Option<String> {
        self.next.mark_styles()
    }
}