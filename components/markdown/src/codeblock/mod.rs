mod fence;
mod highlight;

use std::ops::RangeInclusive;

use libs::syntect::util::LinesWithEndings;

use crate::codeblock::highlight::SyntaxHighlighter;
use config::highlighting::{resolve_syntax_and_theme, HighlightSource};
use config::Config;
pub(crate) use fence::FenceSettings;

fn opening_html(
    language: Option<&str>,
    pre_style: Option<String>,
    pre_class: Option<String>,
    line_numbers: bool,
) -> String {
    let mut html = String::from("<pre");
    if line_numbers {
        html.push_str(" data-linenos");
    }
    let mut classes = String::new();

    if let Some(lang) = language {
        classes.push_str("language-");
        classes.push_str(lang);
        classes.push(' ');

        html.push_str(" data-lang=\"");
        html.push_str(lang);
        html.push('"');
    }

    if let Some(styles) = pre_style {
        html.push_str(" style=\"");
        html.push_str(styles.as_str());
        html.push('"');
    }

    if let Some(c) = pre_class {
        classes.push_str(&c);
    }

    if !classes.is_empty() {
        html.push_str(" class=\"");
        html.push_str(&classes);
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

pub struct CodeBlock<'config> {
    highlighter: SyntaxHighlighter<'config>,
    // fence options
    line_numbers: bool,
    line_number_start: usize,
    highlight_lines: Vec<RangeInclusive<usize>>,
    hide_lines: Vec<RangeInclusive<usize>>,
}

impl<'config> CodeBlock<'config> {
    pub fn new<'fence_info>(
        fence: FenceSettings<'fence_info>,
        config: &'config Config,
        // path to the current file if there is one, to point where the error is
        path: Option<&'config str>,
    ) -> (Self, String) {
        let syntax_and_theme = resolve_syntax_and_theme(fence.language, config);
        if syntax_and_theme.source == HighlightSource::NotFound && config.markdown.highlight_code {
            let lang = fence.language.unwrap();
            if let Some(p) = path {
                eprintln!("Warning: Highlight language {} not found in {}", lang, p);
            } else {
                eprintln!("Warning: Highlight language {} not found", lang);
            }
        }
        let highlighter = SyntaxHighlighter::new(config.markdown.highlight_code, syntax_and_theme);

        let html_start = opening_html(
            fence.language,
            highlighter.pre_style(),
            highlighter.pre_class(),
            fence.line_numbers,
        );
        (
            Self {
                highlighter,
                line_numbers: fence.line_numbers,
                line_number_start: fence.line_number_start,
                highlight_lines: fence.highlight_lines,
                hide_lines: fence.hide_lines,
            },
            html_start,
        )
    }

    pub fn highlight(&mut self, content: &str) -> String {
        let mut buffer = String::new();
        let mark_style = self.highlighter.mark_style();

        if self.line_numbers {
            buffer.push_str("<table><tbody>");
        }

        // syntect leaking here in this file
        for (i, line) in LinesWithEndings::from(content).enumerate() {
            let one_indexed = i + 1;
            // first do we need to skip that line?
            let mut skip = false;
            for range in &self.hide_lines {
                if range.contains(&one_indexed) {
                    skip = true;
                    break;
                }
            }
            if skip {
                continue;
            }

            // Next is it supposed to be higlighted?
            let mut is_higlighted = false;
            for range in &self.highlight_lines {
                if range.contains(&one_indexed) {
                    is_higlighted = true;
                }
            }

            let maybe_mark = |buffer: &mut String, s: &str| {
                if is_higlighted {
                    buffer.push_str("<mark");
                    if let Some(ref style) = mark_style {
                        buffer.push_str(" style=\"");
                        buffer.push_str(style);
                        buffer.push_str("\">");
                    } else {
                        buffer.push('>')
                    }
                    buffer.push_str(s);
                    buffer.push_str("</mark>");
                } else {
                    buffer.push_str(s);
                }
            };

            if self.line_numbers {
                buffer.push_str("<tr><td>");
                let num = format!("{}", self.line_number_start + i);
                maybe_mark(&mut buffer, &num);
                buffer.push_str("</td><td>");
            }

            let highlighted_line = self.highlighter.highlight_line(line);
            maybe_mark(&mut buffer, &highlighted_line);

            if self.line_numbers {
                buffer.push_str("</td></tr>");
            }
        }

        if self.line_numbers {
            buffer.push_str("</tbody></table>");
        }

        buffer
    }
}
