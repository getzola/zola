use config::highlighting::{resolve_syntax_and_theme, SyntaxAndTheme};
use config::Config;
use tera::escape_html;
use syntect::highlighting::{Style, FontStyle, Color, Theme, HighlightState, Highlighter, HighlightIterator};
use syntect::html::{
    start_highlighted_html_snippet, IncludeBackground,
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
        prev_style: Option<Style>
    },
    Classed {
        scope_stack: ScopeStack,
        open_spans: Vec<String>,
        need_reopen: bool
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
    fn close(&self, html: &mut String) {
        match self {
            Inline {prev_style, .. } => {
                if prev_style.is_some() {
                    html.push_str("</span>");
                }
            },
            Classed { open_spans, .. } => {
                html.extend((0..(open_spans.len())).map(|_| "</span>"));
            }
        }
    }
    
    fn close_to_root(&mut self, html: &mut String) {
        // Close open spans:
        match self {
            Inline { prev_style, .. } => {
                if prev_style.take().is_some() {
                    html.push_str("</span>");
                }
            },
            Classed { open_spans, need_reopen, .. } => {
                html.extend((0..open_spans.len()).map(|_| "</span>"));
                *need_reopen = true;
            }
        }
    }
    fn handle_line(&mut self, line: String, tokens: Vec<(usize, ScopeStackOp)>, html: &mut String) {
        match self {
            Inline { 
                highlight_state,
                highlighter,
                include_background,
                prev_style,
                ..
            } => {
                for (ref style, text) in HighlightIterator::new(
                    highlight_state,
                    &tokens,
                    &line,
                    highlighter
                ) {
                    let unify_style = if let Some(ps) = prev_style {
                        style == ps ||
                            (style.background == ps.background && text.trim().is_empty())
                    } else {
                        false
                    };
                    if unify_style {
                        html.push_str(&escape_html(text));
                    } else {
                        if prev_style.is_some() {
                            html.push_str("</span>");
                        }
                        *prev_style = Some(*style);
                        html.push_str("<span style=\"");
                        let include_bg = match include_background {
                            IncludeBackground::Yes => true,
                            IncludeBackground::No => false,
                            IncludeBackground::IfDifferent(c) => (style.background != *c),
                        };
                        if include_bg {
                            html.push_str("background-color:");
                            css_color(html, &style.background);
                            html.push(';');
                        }
                        if style.font_style.contains(FontStyle::UNDERLINE) {
                            html.push_str("text-decoration:underline;");
                        }
                        if style.font_style.contains(FontStyle::BOLD) {
                            html.push_str("font-weight:bold;");
                        }
                        if style.font_style.contains(FontStyle::ITALIC) {
                            html.push_str("font-style:italic;");
                        }
                        html.push_str("color:");
                        css_color(html, &style.foreground);
                        html.push_str(";\">");
                        html.push_str(&escape_html(text));
                    }
                }
            }
            Classed { scope_stack, open_spans, need_reopen } => {
                let repo =
                    SCOPE_REPO.lock().expect("A thread must have poisened the scope repo mutex.");
                let mut prev_i = 0usize;
                tokens.iter().for_each(|(i, op)| {
                    html.push_str(&escape_html(&line[prev_i..*i]));
                    prev_i = *i;
                    // TODO: Handle empty text and empty spans.
                    scope_stack.apply_with_hook(op, |basic_op, _| match basic_op {
                        BasicScopeStackOp::Pop => {
                            if !*need_reopen {
                                html.push_str("</span>");
                            }
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
                            if *need_reopen {
                                html.extend(open_spans.iter().map(String::as_str));
                                *need_reopen = false;
                            }
                            html.push_str(&new_span);
                            open_spans.push(new_span);
                        }
                    });
                });
                let remainder = &line[prev_i..];
                if !remainder.is_empty() {
                    if *need_reopen {
                        html.extend(open_spans.iter().map(String::as_str));
                        *need_reopen = false;
                    }
                    html.push_str(&escape_html(remainder));
                }
            }
        }
    }
}
use CodeBlockImplementation::{Classed, Inline};

pub struct CodeBlock<'config> {
    syntax_set: &'config SyntaxSet,
    line_numbers: bool,
    highlight_lines: Vec<Range>,
    current_line: usize,
    mark_open: bool,
    remainder: String,
    save: String,
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
            Some(1)
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
                prev_style: None,
                theme,
            }
        } else {
            Classed {
                scope_stack: ScopeStack::new(),
                open_spans: Vec::new(),
                need_reopen: false,
            }
        };
        
        let mut ret = Self {
            save: String::from("</td><td>"),
            parser: ParseState::new(syntax),
            mark_open: false,
            remainder: String::new(),
            syntax_set,
            line_numbers,
            // is_first_line: true,
            current_line,
            highlight_lines,
            inner,
        };
        let begin = ret.begin(language);

        (ret, begin)
    }

    fn begin(&mut self, language: Option<&str>) -> String {
        // Open the <pre> tag
        let mut html = match &mut self.inner {
            Inline { theme, .. } => start_highlighted_html_snippet(theme).0,
            Classed { .. } => {
                // When Syntect outputs CSS for a theme, it places the default color and background onto `.code`
                r#"<pre class="code">"#.into()
            }
        };
        // Open the <code> tag
        if let Some(lang) = language {
            html.push_str("<code class=\"language-");
            html.push_str(lang);
            html.push_str("\" data-lang=\"");
            html.push_str(lang);
            html.push_str(r#"">"#);
        } else {
            html.push_str("<code>");
        }
        // Open the table if line numbers are on
        if self.line_numbers {
            html.push_str("<table><tr><td>");
        }
        html
    }

    pub fn finish(self) -> String {
        let CodeBlock {
            mark_open,
            remainder,
            inner,
            line_numbers,
            mut save,
            ..
        } = self;
        // Output the remaining text (TODO: Also highlight the remainder before output)
        if self.mark_open {
            save.insert_str(0, "</mark>");
        }
        save.push_str(&escape_html(&remainder));
        // Close any open spans
        inner.close(&mut save);
        // Close <mark>
        if mark_open {
            save.push_str("</mark>");
        }
        // Close <td><tr><table> if line numbers
        if line_numbers {
            save.push_str("</td></tr></table>");
        }
        save.push_str("</code></pre>");
        save
    }

    fn get_line(&mut self) -> Option<String> {
        self.remainder.find('\n').map(|mut ind| {
            ind += '\n'.len_utf8(); // Include the newline in what we return.
            let mut line = self.remainder.split_off(ind);
            std::mem::swap(&mut line, &mut self.remainder);
            line
        })
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
            let is_highlighted = self.is_line_highlighted();
            
            // Where to store the highlighted output.  Without line numbers we'll just output it, but if we're using line numbers than we'll save the output while we output line numbers, and then when we finish we'll output the saved highlighted code.
            let output_location = if self.line_numbers {
                if is_highlighted && !self.mark_open {
                    self.inner.open_mark(&mut html);
                }
                if self.mark_open && !is_highlighted {
                    html.push_str("</mark>");
                }
                html.push_str(&self.current_line.to_string());
                html.push('\n');
                &mut self.save
            } else {
                &mut html
            };
            
            if is_highlighted != self.mark_open {
                self.inner.close_to_root(output_location);
                
                // Handle hl_lines
                if is_highlighted {
                    self.inner.open_mark(output_location);
                    self.mark_open = true;
                } else {
                    output_location.push_str("</mark>");
                    self.mark_open = false;
                }
            }

            // Parse the line:
            let tokens = self.parser.parse_line(line.as_str(), self.syntax_set);
            self.inner.handle_line(line, tokens, output_location);
            
            self.current_line += 1;
        }
        html
    }
}
