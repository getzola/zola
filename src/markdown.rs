use std::borrow::Cow::Owned;

use pulldown_cmark as cmark;
use self::cmark::{Parser, Event, Tag};

use syntect::dumps::from_binary;
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use syntect::html::{start_coloured_html_snippet, styles_to_coloured_html, IncludeBackground};


// We need to put those in a struct to impl Send and sync
pub struct Setup {
    syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
}

unsafe impl Send for Setup {}
unsafe impl Sync for Setup {}

lazy_static!{
    pub static ref SETUP: Setup = Setup {
        syntax_set: SyntaxSet::load_defaults_newlines(),
        theme_set: from_binary(include_bytes!("../sublime_themes/all.themedump"))
    };
}


struct CodeHighlightingParser<'a> {
    // The block we're currently highlighting
    highlighter: Option<HighlightLines<'a>>,
    parser: Parser<'a>,
    theme: &'a str,
}

impl<'a> CodeHighlightingParser<'a> {
    pub fn new(parser: Parser<'a>, theme: &'a str) -> CodeHighlightingParser<'a> {
        CodeHighlightingParser {
            highlighter: None,
            parser: parser,
            theme: theme,
        }
    }
}

impl<'a> Iterator for CodeHighlightingParser<'a> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Event<'a>> {
        // Not using pattern matching to reduce indentation levels
        let next_opt = self.parser.next();
        if next_opt.is_none() {
            return None;
        }

        let item = next_opt.unwrap();
        // Below we just look for the start of a code block and highlight everything
        // until we see the end of a code block.
        // Everything else happens as normal in pulldown_cmark
        match item {
            Event::Text(text) => {
                // if we are in the middle of a code block
                if let Some(ref mut highlighter) = self.highlighter {
                    let highlighted = &highlighter.highlight(&text);
                    let html = styles_to_coloured_html(highlighted, IncludeBackground::Yes);
                    Some(Event::Html(Owned(html)))
                } else {
                    Some(Event::Text(text))
                }
            },
            Event::Start(Tag::CodeBlock(ref info)) => {
                let theme = &SETUP.theme_set.themes[self.theme];
                let syntax = info
                    .split(' ')
                    .next()
                    .and_then(|lang| SETUP.syntax_set.find_syntax_by_token(lang))
                    .unwrap_or_else(|| SETUP.syntax_set.find_syntax_plain_text());
                self.highlighter = Some(
                    HighlightLines::new(syntax, theme)
                );
                let snippet = start_coloured_html_snippet(theme);
                Some(Event::Html(Owned(snippet)))
            },
            Event::End(Tag::CodeBlock(_)) => {
                // reset highlight and close the code block
                self.highlighter = None;
                Some(Event::Html(Owned("</pre>".to_owned())))
            },
            _ => Some(item)
        }

    }
}

pub fn markdown_to_html(content: &str, highlight_code: bool, highlight_theme: &str) -> String {
    // We try to be smart about highlighting code as it can be time-consuming
    // If the global config disables it, then we do nothing. However,
    // if we see a code block in the content, we assume that this page needs
    // to be highlighted. It could potentially have false positive if the content
    // has ``` in it but that seems kind of unlikely
    let should_highlight = if highlight_code {
        content.contains("```")
    } else {
        false
    };


    let mut html = String::new();
    if should_highlight {
        let parser = CodeHighlightingParser::new(Parser::new(content), highlight_theme);
        cmark::html::push_html(&mut html, parser);
    } else {
        let parser = Parser::new(content);
        cmark::html::push_html(&mut html, parser);
    };
    html
}


#[cfg(test)]
mod tests {
    use super::{markdown_to_html};

    #[test]
    fn test_markdown_to_html_simple() {
        let res = markdown_to_html("# hello", true, "base16-ocean-dark");
        assert_eq!(res, "<h1>hello</h1>\n");
    }

    #[test]
    fn test_markdown_to_html_code_block_highlighting_off() {
        let res = markdown_to_html("```\n$ gutenberg server\n```", false, "base16-ocean-dark");
        assert_eq!(
            res,
            "<pre><code>$ gutenberg server\n</code></pre>\n"
        );
    }

    #[test]
    fn test_markdown_to_html_code_block_no_lang() {
        let res = markdown_to_html("```\n$ gutenberg server\n$ ping\n```", true, "base16-ocean-dark");
        assert_eq!(
            res,
            "<pre style=\"background-color:#2b303b\">\n<span style=\"background-color:#2b303b;color:#c0c5ce;\">$ gutenberg server\n</span><span style=\"background-color:#2b303b;color:#c0c5ce;\">$ ping\n</span></pre>"
        );
    }

    #[test]
    fn test_markdown_to_html_code_block_with_lang() {
        let res = markdown_to_html("```python\nlist.append(1)\n```", true, "base16-ocean-dark");
        assert_eq!(
            res,
            "<pre style=\"background-color:#2b303b\">\n<span style=\"background-color:#2b303b;color:#c0c5ce;\">list</span><span style=\"background-color:#2b303b;color:#c0c5ce;\">.</span><span style=\"background-color:#2b303b;color:#bf616a;\">append</span><span style=\"background-color:#2b303b;color:#c0c5ce;\">(</span><span style=\"background-color:#2b303b;color:#d08770;\">1</span><span style=\"background-color:#2b303b;color:#c0c5ce;\">)</span><span style=\"background-color:#2b303b;color:#c0c5ce;\">\n</span></pre>"
        );
    }
    #[test]
    fn test_markdown_to_html_code_block_with_unknown_lang() {
        let res = markdown_to_html("```yolo\nlist.append(1)\n```", true, "base16-ocean-dark");
        // defaults to plain text
        assert_eq!(
            res,
            "<pre style=\"background-color:#2b303b\">\n<span style=\"background-color:#2b303b;color:#c0c5ce;\">list.append(1)\n</span></pre>"
        );
    }
}
