use config::highlighting::{resolve_syntax_and_theme, SyntaxAndTheme};
use config::Config;
use html_escape::encode_text_to_string;
use std::cmp::min;
use std::collections::HashSet;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, Style, Theme};
use syntect::html::{
    start_highlighted_html_snippet, styled_line_to_highlighted_html, IncludeBackground,
};
use syntect::parsing::{BasicScopeStackOp, ParseState, ScopeStack, SyntaxSet, SCOPE_REPO};
use syntect::util::LinesWithEndings;

use super::fence::{FenceSettings, Range};

enum CodeBlockImplementation<'config> {
    Inline {
        highlighter: HighlightLines<'config>,
        hl_background: Color,
        include_background: IncludeBackground,
        theme: &'config Theme,
    },
    Classed {
        parser: ParseState,
        scope_stack: ScopeStack,
        // Open spans doesn't need to be a vec yet, but when line numbers are implemented, classed highlighting will close and then reopen any spans.
        open_spans: Vec<String>,
        // TODO handle if the text doesn't end on a newline
        #[allow(unused)]
        remainder: String,
    },
}
use CodeBlockImplementation::{Classed, Inline};
pub struct CodeBlock<'config> {
    syntax_set: &'config SyntaxSet,
    #[allow(unused)]
    line_numbers: Option<usize>,
    highlight_lines: Vec<Range>,
    inner: CodeBlockImplementation<'config>,
}

fn get_hl_background(theme: &Theme) -> Color {
    theme.settings.line_highlight.unwrap_or(Color { r: 255, g: 255, b: 0, a: 0 })
}

fn get_include_background(theme: &Theme) -> IncludeBackground {
    IncludeBackground::IfDifferent(
        theme.settings.background.unwrap_or(Color::WHITE),
    )
}

impl<'config, 'fence_info> CodeBlock<'config> {
    pub fn new(fence: FenceSettings<'fence_info>, config: &'config Config) -> (Self, String) {
        let FenceSettings { language, line_numbers, highlight_lines } = fence;
        let line_numbers = if line_numbers {
            // TODO: Update fence to enable setting custom number start.
            Some(0)
        } else {
            None
        };

        let SyntaxAndTheme { syntax, syntax_set, theme } =
            resolve_syntax_and_theme(language, config);

        let inner = if let Some(theme) = theme {
            Inline {
                highlighter: HighlightLines::new(syntax, theme),
                hl_background: get_hl_background(theme),
                include_background: get_include_background(theme),
                theme,
            }
        } else {
            Classed {
                parser: ParseState::new(syntax),
                scope_stack: ScopeStack::new(),
                open_spans: Vec::new(),
                remainder: String::new(),
            }
        };

        let mut ret = Self { syntax_set, line_numbers, highlight_lines, inner };
        let begin = ret.begin(language);

        (ret, begin)
    }

    fn begin(&mut self, language: Option<&str>) -> String {
        let mut html = match &mut self.inner {
            Inline { theme, .. } => start_highlighted_html_snippet(theme).0,
            Classed { .. } => {
                // When Syntect outputs CSS for a theme, it places the default color and background onto `.code`
                r#"<pre class="code">"#.into()
            }
        };
        if let Some(lang) = language {
            html.push_str("<code class=\"language-");
            html.push_str(lang);
            html.push_str("\" data-lang=\"");
            html.push_str(lang);
            html.push_str(r#"">"#);
        } else {
            html.push_str("<code>");
        }
        html
    }

    pub fn finish(self) -> String {
        let html = match self.inner {
            Inline { .. } => String::new(),
            Classed { open_spans, .. } => (0..(open_spans.len())).map(|_| "</span>").collect(),
        };
        return html + "</code></pre>";
    }

    pub fn highlight(&mut self, text: &str) -> String {
        match &mut self.inner {
            Inline { highlighter, hl_background, include_background, .. } => {
                let highlighted = highlighter.highlight(text, self.syntax_set);
                let (line_boundaries, num_lines) = find_line_boundaries(&highlighted);

                // First we make sure that `highlighted` is split at every line
                // boundary. The `styled_line_to_highlighted_html` function will
                // merge split items with identical styles, so this is not a
                // problem.
                //
                // Note that this invalidates the values in `line_boundaries`.
                // The `perform_split` function takes it by value to ensure that
                // we don't use it later.
                let mut highlighted = perform_split(&highlighted, line_boundaries);

                let hl_lines = get_highlighted_lines(&self.highlight_lines, num_lines);
                color_highlighted_lines(&mut highlighted, &hl_lines, *hl_background);

                styled_line_to_highlighted_html(&highlighted, *include_background)
            }
            Classed { parser, scope_stack, open_spans, .. } => {
                // This essentially does the same thing as ClassedHtmlGenerator, except:
                // 1. It outputs each line one at a time
                //   * This will be helpful for line numbers
                // 2. It shares a scope_stack across lines which solves the JSON syntax crash
                // TODO: Support highlighting lines
                // TODO: Handle if text doesn't end in a newline?
                let repo =
                    SCOPE_REPO.lock().expect("A thread must have poisened the scope repo mutex.");
                let mut html = String::new();
                for line in LinesWithEndings::from(text) {
                    let tokens = parser.parse_line(line, self.syntax_set);
                    let mut prev_i = 0usize;
                    tokens.iter().for_each(|(i, op)| {
                        encode_text_to_string(&line[prev_i..*i], &mut html);
                        prev_i = *i;
                        // TODO: Handle empty text and empty spans.
                        scope_stack.apply_with_hook(op, |basic_op, _| match basic_op {
                            BasicScopeStackOp::Pop => {
                                html += "</span>";
                                open_spans.pop();
                            }
                            BasicScopeStackOp::Push(scope) => {
                                let mut new_span = String::from(r#"<span class=""#);
                                for i in 0..(scope.len()) {
                                    let atom = scope.atom_at(i as usize);
                                    let atom_s = repo.atom_str(atom);
                                    if i != 0 {
                                        new_span.push_str(" ");
                                    }
                                    new_span.push_str(atom_s);
                                }
                                new_span.push_str("\">");
                                html += &new_span;
                                open_spans.push(new_span);
                            }
                        });
                    });
                    encode_text_to_string(&line[prev_i..], &mut html);
                }
                html
            }
        }
    }
}

fn find_line_boundaries(styled: &[(Style, &str)]) -> (Vec<StyledIdx>, usize) {
    let mut boundaries = Vec::new();
    for (vec_idx, (_style, s)) in styled.iter().enumerate() {
        for (str_idx, character) in s.char_indices() {
            if character == '\n' {
                boundaries.push(StyledIdx { vec_idx, str_idx });
            }
        }
    }
    let num_lines = boundaries.len() + 1;

    (boundaries, num_lines)
}

fn get_highlighted_lines(highlight_lines: &[Range], num_lines: usize) -> HashSet<usize> {
    let mut lines = HashSet::new();
    for range in highlight_lines {
        for line in range.from..=min(range.to, num_lines) {
            // Ranges are one-indexed
            lines.insert(line.saturating_sub(1));
        }
    }
    lines
}

/// This is an index of a character in a `&[(Style, &'b str)]`. The `vec_idx` is the
/// index in the slice, and `str_idx` is the byte index of the character in the
/// corresponding string slice.
///
/// The `Ord` impl on this type sorts lexiographically on `vec_idx`, and then `str_idx`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct StyledIdx {
    vec_idx: usize,
    str_idx: usize,
}

/// This is a utility used by `perform_split`. If the `vec_idx` in the `StyledIdx` is
/// equal to the provided value, return the `str_idx`, otherwise return `None`.
fn get_str_idx_if_vec_idx_is(idx: Option<&StyledIdx>, vec_idx: usize) -> Option<usize> {
    match idx {
        Some(idx) if idx.vec_idx == vec_idx => Some(idx.str_idx),
        _ => None,
    }
}

/// This function assumes that `line_boundaries` is sorted according to the `Ord` impl on
/// the `StyledIdx` type.
fn perform_split<'b>(
    split: &[(Style, &'b str)],
    line_boundaries: Vec<StyledIdx>,
) -> Vec<(Style, &'b str)> {
    let mut result = Vec::new();

    let mut idxs_iter = line_boundaries.into_iter().peekable();

    for (split_idx, item) in split.iter().enumerate() {
        let mut last_split = 0;

        // Since `line_boundaries` is sorted, we know that any remaining indexes in
        // `idxs_iter` have `vec_idx >= split_idx`, and that if there are any with
        // `vec_idx == split_idx`, they will be first.
        //
        // Using the `get_str_idx_if_vec_idx_is` utility, this loop will keep consuming
        // indexes from `idxs_iter` as long as `vec_idx == split_idx` holds. Once
        // `vec_idx` becomes larger than `split_idx`, the loop will finish without
        // consuming that index.
        //
        // If `idxs_iter` is empty, or there are no indexes with `vec_idx == split_idx`,
        // the loop does nothing.
        while let Some(str_idx) = get_str_idx_if_vec_idx_is(idxs_iter.peek(), split_idx) {
            // Consume the value we just peeked.
            idxs_iter.next();

            // This consumes the index to split at. We add one to include the newline
            // together with its own line, rather than as the first character in the next
            // line.
            let split_at = min(str_idx + 1, item.1.len());

            // This will fail if `line_boundaries` is not sorted.
            debug_assert!(split_at >= last_split);

            // Skip splitting if the string slice would be empty.
            if last_split != split_at {
                result.push((item.0, &item.1[last_split..split_at]));
                last_split = split_at;
            }
        }

        // Now append the remainder. If the current item was not split, this will
        // append the entire item.
        if last_split != item.1.len() {
            result.push((item.0, &item.1[last_split..]));
        }
    }

    result
}

fn color_highlighted_lines(data: &mut [(Style, &str)], lines: &HashSet<usize>, background: Color) {
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
