use config::highlighting::{resolve_syntax_and_theme, SyntaxAndTheme};
use config::Config;
use syntect::highlighting::{
    Color, FontStyle, HighlightState, Highlighter, Style, Theme,
};
use syntect::parsing::{
    ParseState, ScopeStack,
};
use tera::escape_html;

use super::fence::FenceSettings;

mod highlight_lines;
use highlight_lines::HighlightLines;
mod line_numbers;
use line_numbers::LineNumbers;
mod syntax_highlight;
use syntax_highlight::SyntaxHighlight;

pub trait CodeBlockPass: Sized {
    fn close_to_root(&mut self, _output: &mut String) {}
    fn handle_line(&mut self, output: &mut String, _line_num: usize, input: &str) {
        output.push_str(escape_html(input).as_str());
    }

    // Pass Overrides
    fn pre_styles(&self, _line_num: usize) -> Option<String> { None }
    fn pre_class(&self) -> Option<String> { None }
    fn mark_styles(&self) -> Option<String> { None }
}
impl CodeBlockPass for () {}

pub struct CodeBlock<T> {
    remainder: String,
    current_line: usize,
    passes: T
}

fn css_color(html: &mut String, color: &Color) {
    // TODO: hex codes or rgb()?
    // html.push_str("rgb(");
    // html.push_str(&color.r.to_string());
    // html.push(' ');
    // html.push_str(&color.g.to_string());
    // html.push(' ');
    // html.push_str(&color.b.to_string());
    // html.push(')');
    html.push_str(format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b).as_str());
    if color.a != 0xFF {
        html.push_str(format!("{:02x}", color.a).as_str());
    }
}

fn get_base_style(theme: &Theme) -> Style {
    Style {
        foreground: theme.settings.foreground.unwrap_or(Color::BLACK),
        background: theme.settings.background.unwrap_or(Color::WHITE),
        font_style: FontStyle::empty()
    }
}

fn get_hl_background(theme: &Theme) -> Color {
    theme.settings.line_highlight.unwrap_or(Color { r: 255, g: 255, b: 0, a: 0 })
}

pub type ZolaCodeBlock<'config> = CodeBlock<LineNumbers<HighlightLines<SyntaxHighlight<'config>>, HighlightLines<()>>>;

impl<'config> ZolaCodeBlock<'config> {
    pub fn new<'fence_info>(fence: FenceSettings<'fence_info>, config: &'config Config) -> (Self, String) {
        let FenceSettings {
            language,
            line_numbers,
            line_number_start: current_line,
            highlight_lines,
        } = fence;

        let SyntaxAndTheme { syntax, syntax_set, theme } =
            resolve_syntax_and_theme(language, config);

        let syntax_highlighter = if config.highlight_code() {
            if let Some(theme) = theme {
                let highlighter = Highlighter::new(theme);
                SyntaxHighlight::Inline {
                    parser: ParseState::new(syntax),
                    syntax_set,
                    hl_state: HighlightState::new(&highlighter, ScopeStack::new()),
                    highlighter,
                    hl_background: get_hl_background(theme),
                    prev_style: None,
                    base_style: get_base_style(theme),
                }
            } else {
                SyntaxHighlight::Classed {
                    parser: ParseState::new(syntax),
                    syntax_set,
                    scope_stack: ScopeStack::new(),
                    open_spans: Vec::new(),
                    need_reopen: false
                }
            }
        } else {
            SyntaxHighlight::NoHighlight
        };

        let line_highlighter = HighlightLines::new(
            highlight_lines.iter().map(|range| range.from..=range.to).collect(),
            syntax_highlighter
        );

        let line_numbers = if line_numbers {
            LineNumbers::RowPerLine {
            // LineNumbers::TwoCell {
            // LineNumbers::CSSCounter {
                code_pass: line_highlighter,
                // TODO: The following is a gross hack, it would be nice if one highlight lines could be shared to highlight the numbers and code.  To do that would require changing the pass trait somehow I think.
                num_pass: HighlightLines::new(highlight_lines.iter().map(|range| range.from..=range.to).collect(), ()),
                openned: false,
                // code_cell: String::new()
            }
        } else {
            LineNumbers::NoLineNumbers {
                code_pass: line_highlighter
            }
        };

        let ret = Self {
            remainder: String::new(),
            current_line,
            passes: line_numbers
        };
        let begin = ret.begin(language);

        (ret, begin)
    }
}
impl<T: CodeBlockPass> CodeBlock<T> {
    pub fn code(&mut self, text: &str) -> String {
        // Add the new text into the remainder
        self.remainder.push_str(text);

        let mut html = String::new();
        while let Some(line) = self.get_line() {
            self.passes.handle_line(&mut html, self.current_line, line.as_str());
            self.current_line += 1;
        }
        html
    }
    fn begin(&self, language: Option<&str>) -> String {
        let mut html = String::from("<pre");
        if let Some(styles) = self.passes.pre_styles(self.current_line) {
            html.push_str(" style=\"");
            html.push_str(styles.as_str());
            html.push('"');
        }
        if let Some(classes) = self.passes.pre_class() {
            html.push_str(" class=\"");
            html.push_str(classes.as_str());
            html.push('"');
        }
        html.push_str("><code");
        if let Some(lang) = language {
            html.push_str(" class=\"language-");
            html.push_str(lang);
            html.push_str("\" data-lang=\"");
            html.push_str(lang);
            html.push('"');
        }
        html.push('>');
        html
    }
    fn get_line(&mut self) -> Option<String> {
        self.remainder.find('\n').map(|mut ind| {
            ind += '\n'.len_utf8(); // Include the newline in what we return.
            let mut line = self.remainder.split_off(ind);
            std::mem::swap(&mut line, &mut self.remainder);
            line
        })
    }
    pub fn close(mut self) -> String {
        // Output any remaining text (there shouldn't be any because code block fences get their own line).
        let mut finish = if !self.remainder.is_empty() {
            // If there's a remainder, then there must not have been a closing newline so add one.
            self.code("\n")
        } else {
            String::new()
        };

        // Close all intermediate passes
        self.passes.close_to_root(&mut finish);

        // Close code and pre
        finish.push_str("</code></pre>");
        finish
    }
}
