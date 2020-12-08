use config::highlighting::{resolve_syntax_and_theme, SyntaxAndTheme};
use config::Config;
use syntect::highlighting::{
    Color, FontStyle, HighlightIterator, HighlightState, Highlighter, Style, Theme,
};
use syntect::html::{IncludeBackground};
use syntect::parsing::{
    BasicScopeStackOp, ParseState, ScopeStack, ScopeStackOp, SyntaxSet, SCOPE_REPO,
};
use tera::escape_html;

use super::fence::{FenceSettings, Range};

enum CodeBlockImplementation<'config> {
    Inline {
        highlighter: Highlighter<'config>,
        highlight_state: HighlightState,
        hl_background: Color,
        include_background: IncludeBackground,
        default_style: Style,
        prev_style: Option<Style>,
    },
    Classed {
        scope_stack: ScopeStack,
        open_spans: Vec<String>,
        need_reopen: bool,
    },
}

fn output_style(html: &mut String, style: &Style, bg: &IncludeBackground) {
    let include_bg = match bg {
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
    html.push(';');
}

fn get_default_style(theme: &Theme) -> Style {
    Style {
        foreground: theme.settings.foreground.unwrap_or(Color::BLACK),
        background: theme.settings.background.unwrap_or(Color::WHITE),
        font_style: FontStyle::empty()
    }
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

impl<'config> CodeBlockImplementation<'config> {
    fn open_mark(&self, html: &mut String) {
        match self {
            Inline { hl_background, .. } => {
                // TODO: Should the mark have a background color applied? (pro) Having the background color is more automatic.  (con) Harder to override.
                html.push_str("<mark style=\"background-color:");
                css_color(html, hl_background);
                html.push_str(";\">");
            }
            Classed { .. } => html.push_str("<mark>"),
        }
    }
    fn close_to_root(&mut self, html: &mut String) {
        // Close open spans:
        match self {
            Inline { prev_style, .. } => {
                if prev_style.take().is_some() {
                    html.push_str("</span>");
                }
            }
            Classed { open_spans, need_reopen, .. } => {
                html.extend((0..open_spans.len()).map(|_| "</span>"));
                *need_reopen = true;
            }
        }
    }
    fn handle_line(&mut self, line: String, tokens: Vec<(usize, ScopeStackOp)>, html: &mut String) {
        match self {
            Inline { highlight_state, highlighter, include_background, prev_style, default_style, .. } => {
                for (style, text) in
                    HighlightIterator::new(highlight_state, &tokens, &line, highlighter)
                {
                    let can_unify = prev_style.clone().map(|ps| {
                        ps == style || (
                            text.trim().is_empty() && ps.background == style.background
                        )
                    }).unwrap_or(false);
                    if style == *default_style {
                        // If this style is the same as the default, then just escape to root and don't use a span.
                        if prev_style.take().is_some() {
                            html.push_str("</span>");
                        }
                    } else if !can_unify {
                        // Set our current style as the prev_style and output a </span> if there was a previous prev_style
                        if prev_style.replace(style).is_some() {
                            html.push_str("</span>");
                        }
                        html.push_str("<span style=\"");
                        output_style(html, &style, include_background);
                        html.push_str("\">");
                    }
                    html.push_str(&escape_html(text));
                }
            }
            Classed { scope_stack, open_spans, need_reopen } => {
                // When we close_to_root (to insert a mark for example) we don't want to reopen all the spans because they might be immediately closed due to a scope stack pop.  Instead, we wait to reopen the spans until we push a new scope or before outputting text.
                fn ensure_open(html: &mut String, open_spans: &[String], need_reopen: &mut bool) {
                    if *need_reopen {
                        html.extend(open_spans.iter().map(String::as_str));
                        *need_reopen = false;
                    }
                }
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
                                // If we need_reopen then this span doesn't need to be closed because it hasn't been reopened yet.
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
                                    new_span.push(' ');
                                }
                                new_span.push_str(atom_s);
                            }
                            new_span.push_str("\">");

                            ensure_open(html, open_spans, need_reopen);

                            html.push_str(&new_span);
                            open_spans.push(new_span);
                        }
                    });
                });
                let remainder = &line[prev_i..];
                if !remainder.is_empty() {
                    ensure_open(html, open_spans, need_reopen);

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
        let FenceSettings {
            language,
            line_numbers,
            line_number_start: current_line,
            highlight_lines,
        } = fence;

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
                default_style: get_default_style(theme),
            }
        } else {
            Classed { scope_stack: ScopeStack::new(), open_spans: Vec::new(), need_reopen: false }
        };
        let save = if line_numbers {
            // We seed save with </td><td> which is what needs to be inserted between the line numbers (which are output line by line) and the highlighted code (which is collected into save).
            String::from("</td><td>")
        } else {
            String::new()
        };

        let mut ret = Self {
            save,
            parser: ParseState::new(syntax),
            mark_open: false,
            remainder: String::new(),
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
        // Open the <pre> tag
        let mut html = String::from("<pre");
        match &mut self.inner {
            Inline { default_style, .. } => {
                // Output the default style which will be used for line numbers and stuff.
                html.push_str(" style=\"");
                output_style(&mut html, default_style, &IncludeBackground::Yes);
                html.push('"');
            },
            Classed { .. } => {
                // When Syntect outputs CSS for a theme, it places the default color and background onto `.code`
                html.push_str(r#" class="code""#);
            }
        };
        html.push('>');
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

    pub fn finish(mut self) -> String {
        // Output any remaining text (there shouldn't be any).
        if !self.remainder.is_empty() {
            // If there's a remainder, then there must not have been a newline so add one.
            let temp = self.highlight("\n");
            self.save.insert_str(0, &temp);
        }

        // Close any open spans
        self.inner.close_to_root(&mut self.save);
        // Close <mark>
        if self.mark_open {
            self.save.push_str("</mark>");
        }
        // Close <td><tr><table> if line numbers
        if self.line_numbers {
            self.save.push_str("</td></tr></table>");
        }
        // Close the <code> and <pre>
        self.save.push_str("</code></pre>");
        self.save
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
        // Add the new text into the remainder
        self.remainder.push_str(text);

        let mut html = String::new();
        while let Some(line) = self.get_line() {
            let is_highlighted = self.is_line_highlighted();

            // Where to store the highlighted output.  Without line numbers we'll just output it, but if we're using line numbers than we'll save the output while we output line numbers, and then when we finish we'll output the saved highlighted code.
            let output_location = if self.line_numbers {
                // A highlighted line has two mark tags: one around the line number and one around the line
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
            // Syntax highlight the line and output to either html or save
            self.inner.handle_line(line, tokens, output_location);

            self.current_line += 1;
        }
        html
    }
}

#[cfg(test)]
mod linenos_tests {
    use super::*;

    fn highlight(config: &Config, fence_info: &str, text: &str) -> String {
        let fence = FenceSettings::new(fence_info);
        let (mut block, mut highlighted) = CodeBlock::new(fence, config);
        highlighted.push_str(&block.highlight(text));
        highlighted.push_str(&block.finish());
        highlighted
    }
    fn output_inline_plaintext(linenumbers: &str, code: &str) -> String {
        format!("<pre style=\"background-color:#2b303b;color:#c0c5ce;\">\
                <code>\
                    <table>\
                        <tr>\
                            <td>{}</td>\
                            <td>\
                                {}\
                            </td>\
                        </tr>\
                    </table>\
                </code>\
            </pre>", linenumbers, code)
    }

    #[test]
    fn simple() {
        let text = "\
foo
bar
bar
baz
";
        assert_eq!(
            highlight(&Config::default(), "linenos", text),
            output_inline_plaintext("1\n2\n3\n4\n", "foo\nbar\nbar\nbaz\n")
        );
    }

    #[test]
    fn non_one_start() {
        let text = "\
foo
bar
bar
baz
";
        assert_eq!(
            highlight(&Config::default(), "linenos, linenostart=3", text),
            output_inline_plaintext("3\n4\n5\n6\n", "foo\nbar\nbar\nbaz\n")
        );
    }

    #[test]
    fn highlighted_line() {
        let text = "\
foo
bar
bar
baz
";
        assert_eq!(
            highlight(&Config::default(), "linenos, linenostart=3, hl_lines=5", text),
            "<pre style=\"background-color:#2b303b;color:#c0c5ce;\">\
                <code>\
                    <table>\
                        <tr>\
                            <td>\
                                3\n4\n\
                                <mark style=\"background-color:#65737e30;\">5\n</mark>\
                                6\n\
                            </td>\
                            <td>\
                                foo\nbar\n\
                                <mark style=\"background-color:#65737e30;\">\
                                    bar\n\
                                </mark>\
                                baz\n\
                            </td>\
                        </tr>\
                    </table>\
                </code>\
            </pre>"
        );
    }
}
