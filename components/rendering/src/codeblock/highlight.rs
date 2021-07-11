use std::fmt::Write;

use config::highlighting::{SyntaxAndTheme, CLASS_STYLE};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, Theme};
use syntect::html::{
    styled_line_to_highlighted_html, tokens_to_classed_spans, ClassStyle, IncludeBackground,
};
use syntect::parsing::{ParseState, ScopeStack, SyntaxReference, SyntaxSet};

/// Not public, but from syntect::html
fn write_css_color(s: &mut String, c: Color) {
    if c.a != 0xFF {
        write!(s, "#{:02x}{:02x}{:02x}{:02x}", c.r, c.g, c.b, c.a).unwrap();
    } else {
        write!(s, "#{:02x}{:02x}{:02x}", c.r, c.g, c.b).unwrap();
    }
}

pub(crate) struct ClassHighlighter<'config> {
    syntax_set: &'config SyntaxSet,
    open_spans: isize,
    parse_state: ParseState,
    scope_stack: ScopeStack,
}

impl<'config> ClassHighlighter<'config> {
    pub fn new(syntax: &SyntaxReference, syntax_set: &'config SyntaxSet) -> Self {
        let parse_state = ParseState::new(syntax);
        Self { syntax_set, open_spans: 0, parse_state, scope_stack: ScopeStack::new() }
    }

    /// Parse the line of code and update the internal HTML buffer with tagged HTML
    ///
    /// *Note:* This function requires `line` to include a newline at the end and
    /// also use of the `load_defaults_newlines` version of the syntaxes.
    pub fn highlight_line(&mut self, line: &str) -> String {
        debug_assert!(line.ends_with("\n"));
        let parsed_line = self.parse_state.parse_line(line, &self.syntax_set);
        let (formatted_line, delta) = tokens_to_classed_spans(
            line,
            parsed_line.as_slice(),
            CLASS_STYLE,
            &mut self.scope_stack,
        );
        self.open_spans += delta;
        formatted_line
    }

    /// Close all open `<span>` tags and return the finished HTML string
    pub fn finalize(&mut self) -> String {
        let mut html = String::with_capacity((self.open_spans * 7) as usize);
        for _ in 0..self.open_spans {
            html.push_str("</span>");
        }
        html
    }
}

pub(crate) struct InlineHighlighter<'config> {
    theme: &'config Theme,
    fg_color: String,
    bg_color: Color,
    syntax_set: &'config SyntaxSet,
    h: HighlightLines<'config>,
}

impl<'config> InlineHighlighter<'config> {
    pub fn new(
        syntax: &'config SyntaxReference,
        syntax_set: &'config SyntaxSet,
        theme: &'config Theme,
    ) -> Self {
        let h = HighlightLines::new(syntax, theme);
        let mut color = String::new();
        write_css_color(&mut color, theme.settings.foreground.unwrap_or(Color::BLACK));
        let fg_color = format!(r#" style="color:{};""#, color);
        let bg_color = theme.settings.background.unwrap_or(Color::WHITE);
        Self { theme, fg_color, bg_color, syntax_set, h }
    }

    pub fn highlight_line(&mut self, line: &str) -> String {
        let regions = self.h.highlight(line, &self.syntax_set);
        // TODO: add a param like `IncludeBackground` for `IncludeForeground` in syntect
        let highlighted = styled_line_to_highlighted_html(
            &regions,
            IncludeBackground::IfDifferent(self.bg_color),
        );
        highlighted.replace(&self.fg_color, "")
    }
}

pub(crate) enum SyntaxHighlighter<'config> {
    Inlined(InlineHighlighter<'config>),
    Classed(ClassHighlighter<'config>),
    /// We might not want highlighting but we want line numbers or to hide some lines
    NoHighlight,
}

impl<'config> SyntaxHighlighter<'config> {
    pub fn new(highlight_code: bool, s: SyntaxAndTheme<'config>) -> Self {
        if highlight_code {
            if let Some(theme) = s.theme {
                SyntaxHighlighter::Inlined(InlineHighlighter::new(s.syntax, s.syntax_set, theme))
            } else {
                SyntaxHighlighter::Classed(ClassHighlighter::new(s.syntax, s.syntax_set))
            }
        } else {
            SyntaxHighlighter::NoHighlight
        }
    }

    pub fn highlight_line(&mut self, line: &str) -> String {
        use SyntaxHighlighter::*;

        match self {
            Inlined(h) => h.highlight_line(line),
            Classed(h) => h.highlight_line(line),
            NoHighlight => line.to_owned(),
        }
    }

    pub fn finalize(&mut self) -> Option<String> {
        use SyntaxHighlighter::*;

        match self {
            Inlined(_) | NoHighlight => None,
            Classed(h) => Some(h.finalize()),
        }
    }

    /// Inlined needs to set the background/foreground colour on <pre>
    pub fn pre_style(&self) -> Option<String> {
        use SyntaxHighlighter::*;

        match self {
            Classed(_) | NoHighlight => None,
            Inlined(h) => {
                let mut styles = String::from("background-color:");
                write_css_color(&mut styles, h.theme.settings.background.unwrap_or(Color::WHITE));
                styles.push_str(";color:");
                write_css_color(&mut styles, h.theme.settings.foreground.unwrap_or(Color::BLACK));
                styles.push(';');
                Some(styles)
            }
        }
    }

    /// Classed needs to set a class on the pre
    pub fn pre_class(&self) -> Option<String> {
        use SyntaxHighlighter::*;

        match self {
            Classed(_) => {
                if let ClassStyle::SpacedPrefixed { prefix } = CLASS_STYLE {
                    Some(format!("{}code", prefix))
                } else {
                    unreachable!()
                }
            }
            Inlined(_) | NoHighlight => None,
        }
    }

    /// Inlined needs to set the background/foreground colour
    pub fn mark_style(&self) -> Option<String> {
        use SyntaxHighlighter::*;

        match self {
            Classed(_) | NoHighlight => None,
            Inlined(h) => {
                let mut styles = String::from("background-color:");
                write_css_color(
                    &mut styles,
                    h.theme.settings.line_highlight.unwrap_or(Color { r: 255, g: 255, b: 0, a: 0 }),
                );
                styles.push_str(";");
                Some(styles)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::highlighting::resolve_syntax_and_theme;
    use config::Config;
    use syntect::util::LinesWithEndings;

    #[test]
    fn can_highlight_with_classes() {
        let mut config = Config::default();
        config.markdown.highlight_code = true;
        let code = "import zen\nz = x + y\nprint('hello')\n";
        let syntax_and_theme = resolve_syntax_and_theme(Some("py"), &config);
        let mut highlighter =
            ClassHighlighter::new(syntax_and_theme.syntax, syntax_and_theme.syntax_set);
        let mut out = String::new();
        for line in LinesWithEndings::from(&code) {
            out.push_str(&highlighter.highlight_line(line));
        }
        out.push_str(&highlighter.finalize());

        assert!(out.starts_with("<span class"));
        assert!(out.ends_with("</span>"));
        assert!(out.contains("z-"));
    }

    #[test]
    fn can_highlight_inline() {
        let mut config = Config::default();
        config.markdown.highlight_code = true;
        let code = "import zen\nz = x + y\nprint('hello')\n";
        let syntax_and_theme = resolve_syntax_and_theme(Some("py"), &config);
        let mut highlighter = InlineHighlighter::new(
            syntax_and_theme.syntax,
            syntax_and_theme.syntax_set,
            syntax_and_theme.theme.unwrap(),
        );
        let mut out = String::new();
        for line in LinesWithEndings::from(&code) {
            out.push_str(&highlighter.highlight_line(line));
        }

        assert!(out.starts_with(r#"<span style="color"#));
        assert!(out.ends_with("</span>"));
    }
}
