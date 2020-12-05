use config::highlighting::{resolve_syntax_and_theme, SyntaxAndTheme};
use config::Config;
use html_escape::encode_text_to_string;
use syntect::highlighting::{Color, Theme, HighlightState, Highlighter, HighlightIterator};
use syntect::html::{
    start_highlighted_html_snippet, append_highlighted_html_for_styled_line, IncludeBackground,
};
use syntect::parsing::{BasicScopeStackOp, ParseState, ScopeStack, ScopeStackOp, SyntaxSet, SCOPE_REPO};

use super::fence::{FenceSettings, Range};

enum CodeBlockImplementation<'config> {
    Inline {
        highlighter: Highlighter<'config>,
        highlight_state: HighlightState,
        hl_background: Color,
        include_background: IncludeBackground,
        theme: &'config Theme,
    },
    Classed {
        scope_stack: ScopeStack,
    },
}

fn css_color(html: &mut String, color: &Color) {
    // TODO: Could also output hex codes using something like Syntect's write_css_color
    html.push_str("rgb(");
    html.push_str(&color.r.to_string());
    html.push(' ');
    html.push_str(&color.g.to_string());
    html.push(' ');
    html.push_str(&color.b.to_string());
    html.push(')');
}

impl<'config> CodeBlockImplementation<'config> {
    fn open_mark(&self, html: &mut String) {
        match self {
            Inline { hl_background, .. } => {
                html.push_str("<mark style=\"background-color: ");
                css_color(html, hl_background);
                html.push_str(";\">");
            },
            Classed { .. } => html.push_str("<mark>")
        }
    }
    fn handle_line(&mut self, line: String, tokens: Vec<(usize, ScopeStackOp)>, open_spans: &mut Vec<String>, html: &mut String) {
        match self {
            Inline { highlight_state, highlighter, include_background, .. } => {
                let highlighted: Vec<_> = HighlightIterator::new(highlight_state, &tokens, &line, highlighter).collect();

                append_highlighted_html_for_styled_line(&highlighted, *include_background, html);
            }
            Classed { scope_stack } => {
                let repo =
                    SCOPE_REPO.lock().expect("A thread must have poisened the scope repo mutex.");
                let mut prev_i = 0usize;
                tokens.iter().for_each(|(i, op)| {
                    encode_text_to_string(&line[prev_i..*i], html);
                    prev_i = *i;
                    // TODO: Handle empty text and empty spans.
                    scope_stack.apply_with_hook(op, |basic_op, _| match basic_op {
                        BasicScopeStackOp::Pop => {
                            html.push_str("</span>");
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
                            html.push_str(&new_span);
                            open_spans.push(new_span);
                        }
                    });
                });
                encode_text_to_string(&line[prev_i..], html);
            }
        }
    }
}
use CodeBlockImplementation::{Classed, Inline};

pub struct CodeBlock<'config> {
    syntax_set: &'config SyntaxSet,
    #[allow(unused)]
    line_numbers: bool,
    highlight_lines: Vec<Range>,
    current_line: usize,
    mark_open: bool,
    remainder: String,
    open_spans: Vec<String>,
    parser: ParseState,
    inner: CodeBlockImplementation<'config>,
}

fn get_hl_background(theme: &Theme) -> Color {
    theme.settings.line_highlight.unwrap_or(Color { r: 255, g: 255, b: 0, a: 0 })
}

fn get_include_background(theme: &Theme) -> IncludeBackground {
    IncludeBackground::IfDifferent(theme.settings.background.unwrap_or(Color::WHITE))
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
        let current_line = line_numbers.unwrap_or(1);
        let line_numbers = line_numbers.is_some();

        let SyntaxAndTheme { syntax, syntax_set, theme } =
            resolve_syntax_and_theme(language, config);

        let inner = if let Some(theme) = theme {
            let highlighter = Highlighter::new(theme);
            Inline {
                highlight_state: HighlightState::new(&highlighter, ScopeStack::new()),
                highlighter,
                hl_background: get_hl_background(theme),
                include_background: get_include_background(theme),
                theme,
            }
        } else {
            Classed {
                scope_stack: ScopeStack::new(),
            }
        };
        
        let mut ret = Self {
            parser: ParseState::new(syntax),
            mark_open: false,
            remainder: String::new(),
            open_spans: Vec::new(),
            syntax_set,
            line_numbers,
            current_line,
            highlight_lines,
            inner,
        };
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
        let CodeBlock {
            mark_open,
            mut remainder,
            open_spans,
            ..
        } = self;
        
        remainder.extend((0..(open_spans.len())).map(|_| "</span>"));
        if mark_open {
            // Close <mark>
            remainder.push_str("</mark>");
        }
        remainder.push_str("</code></pre>");
        remainder
    }

    fn get_line(&mut self) -> Option<String> {
        self.remainder.find('\n').map(|mut ind| {
            loop {
                // Should always just run once because '\n' is one byte, but...
                ind += 1;
                if self.remainder.is_char_boundary(ind) {
                    break;
                }
            }
            let mut line = self.remainder.split_off(ind);
            std::mem::swap(&mut line, &mut self.remainder);
            line
        })
    }
    fn insert_at_root<T: FnOnce(&mut String)>(&self, html: &mut String, handler: T) {
        // Close open spans:
        html.extend((0..self.open_spans.len()).map(|_| "</span>"));
        // Make the modification at root
        handler(html);
        // Reopen the spans
        html.extend(self.open_spans.iter().map(|x| x.as_str()));
    }
    fn is_line_highlighted(&self) -> bool {
        let mut is_highlighted = false;
        for range in self.highlight_lines.iter() {
            // TODO: Don't check every range every line
            if (range.from..=range.to).contains(&self.current_line) {
                is_highlighted = true;
                break;
            }
        }
        is_highlighted
    }

    pub fn highlight(&mut self, text: &str) -> String {
        self.remainder.push_str(text);
        let mut html = String::new();
        while let Some(line) = self.get_line() {
            // Handle highlighted lines by closing all the open spans, inserting a <mark> tag, and then reopening them.  We'll need to do the same thing at the end of highlighted lines to close </mark>.
            let is_highlighted = self.is_line_highlighted();
            if is_highlighted != self.mark_open {
                if is_highlighted {
                    self.insert_at_root(&mut html, |html| self.inner.open_mark(html));
                    self.mark_open = true;
                } else {
                    self.insert_at_root(&mut html, |html| html.push_str("</mark>"));
                    self.mark_open = false;
                }
            }

            // Parse the line:
            let tokens = self.parser.parse_line(line.as_str(), self.syntax_set);
            self.inner.handle_line(line, tokens, &mut self.open_spans, &mut html);
            
            self.current_line += 1;
        }
        html
    }
}
