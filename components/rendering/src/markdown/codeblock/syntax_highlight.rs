use syntect::highlighting::{
    Color, FontStyle, HighlightIterator, HighlightState, Highlighter, Style,
};
use syntect::parsing::{
    BasicScopeStackOp, ParseState, ScopeStack, SyntaxSet, SCOPE_REPO,
};
use tera::escape_html;
use super::{CodeBlockPass, css_color};


fn output_style(html: &mut String, style: &Style, include_bg: bool) {
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

pub enum SyntaxHighlight<'config> {
    Inline {
        syntax_set: &'config SyntaxSet,
        parser: ParseState,
        highlighter: Highlighter<'config>,
        hl_state: HighlightState,
        base_style: Style,
        hl_background: Color,
        prev_style: Option<Style>
    },
    Classed {
        syntax_set: &'config SyntaxSet,
        parser: ParseState,
        scope_stack: ScopeStack,
        open_spans: Vec<String>,
        need_reopen: bool
    },
    // I'm thinking that codeblock should handle all codeblocks even
    // when highlighting is off to reduce duplication and to enable
    // line numbers or highlighted lines when syntax highlighting is off.
    NoHighlight
}
use SyntaxHighlight::{Inline, Classed, NoHighlight};

impl<'config> CodeBlockPass for SyntaxHighlight<'config> {
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
            },
            _ => {}
        }
    }
    fn handle_line(&mut self, output: &mut String, _line_num: usize, line: &str) {
        match self {
            NoHighlight => {
                output.push_str(escape_html(line).as_str())
            },
            Inline { hl_state, highlighter, prev_style, base_style, parser, syntax_set, .. } => {
                let tokens = parser.parse_line(line, syntax_set);
                for (style, text) in
                    HighlightIterator::new(hl_state, &tokens, &line, highlighter)
                {
                    fn can_unify(text: &str, a: Style, b: Style) -> bool {
                        a == b || (text.trim().is_empty() && a.background == b.background)
                    }
                    if can_unify(text, style, *base_style) {
                        // If this style is the same as the default, then just escape to root and don't use a span.
                        if prev_style.take().is_some() {
                            output.push_str("</span>");
                        }
                    } else {
                        if prev_style.clone().map(|prev| !can_unify(text, prev, style)).unwrap_or(true) {
                            // Set our current style as the prev_style and output a </span> if there was a previous prev_style
                            if prev_style.replace(style).is_some() {
                                output.push_str("</span>");
                            }
                            output.push_str("<span style=\"");
                            output_style(output, &style, style.background != base_style.background);
                            output.push_str("\">");
                        }
                    }
                    output.push_str(&escape_html(text));
                }
            },
            Classed { scope_stack, open_spans, need_reopen, parser, syntax_set } => {
                let tokens = parser.parse_line(line, syntax_set);
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
                    output.push_str(&escape_html(&line[prev_i..*i]));
                    prev_i = *i;
                    // TODO: Handle empty text and empty spans.
                    scope_stack.apply_with_hook(op, |basic_op, _| match basic_op {
                        BasicScopeStackOp::Pop => {
                            if !*need_reopen {
                                // If we need_reopen then this span doesn't need to be closed because it hasn't been reopened yet.
                                output.push_str("</span>");
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

                            ensure_open(output, open_spans, need_reopen);

                            output.push_str(&new_span);
                            open_spans.push(new_span);
                        }
                    });
                });
                let remainder = &line[prev_i..];
                if !remainder.is_empty() {
                    ensure_open(output, open_spans, need_reopen);

                    output.push_str(&escape_html(remainder));
                }
            }
        }
    }
    fn pre_styles(&self, _line_num: usize) -> Option<String> {
        match self {
            Inline { base_style, .. } => {
                let mut styles = String::from("background-color:");
                css_color(&mut styles, &base_style.background);
                styles.push_str(";color:");
                css_color(&mut styles, &base_style.foreground);
                styles.push(';');
                Some(styles)
            },
            _ => None
        }
    }
    fn pre_class(&self) -> Option<String> {
        match self {
            Classed { .. } => {
                Some(String::from("code"))
            },
            _ => None
        }
    }
    fn mark_styles(&self) -> Option<String> {
        match self {
            Inline { hl_background, .. } => {
                let mut styles = String::from("color:currentcolor;background-color:");
                css_color(&mut styles, hl_background);
                styles.push(';');
                Some(styles)
            },
            _ => None
        }
    }
}