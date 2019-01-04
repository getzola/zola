use std::borrow::Cow::{Borrowed, Owned};

use self::cmark::{Event, Options, Parser, Tag};
use pulldown_cmark as cmark;
use slug::slugify;
use syntect::easy::HighlightLines;
use syntect::html::{
    start_highlighted_html_snippet, styled_line_to_highlighted_html, IncludeBackground,
};

use config::highlighting::{get_highlighter, SYNTAX_SET, THEME_SET};
use errors::Result;
use link_checker::check_url;
use utils::site::resolve_internal_link;

use context::RenderContext;
use table_of_contents::{make_table_of_contents, Header, TempHeader};

const CONTINUE_READING: &str = "<p id=\"zola-continue-reading\"><a name=\"continue-reading\"></a></p>\n";

#[derive(Debug)]
pub struct Rendered {
    pub body: String,
    pub summary_len: Option<usize>,
    pub toc: Vec<Header>,
}

// We might have cases where the slug is already present in our list of anchor
// for example an article could have several titles named Example
// We add a counter after the slug if the slug is already present, which
// means we will have example, example-1, example-2 etc
fn find_anchor(anchors: &[String], name: String, level: u8) -> String {
    if level == 0 && !anchors.contains(&name) {
        return name;
    }

    let new_anchor = format!("{}-{}", name, level + 1);
    if !anchors.contains(&new_anchor) {
        return new_anchor;
    }

    find_anchor(anchors, name, level + 1)
}

// Colocated asset links refers to the files in the same directory,
// there it should be a filename only
fn is_colocated_asset_link(link: &str) -> bool {
    !link.contains('/')  // http://, ftp://, ../ etc
        && !link.starts_with("mailto:")
}

pub fn markdown_to_html(content: &str, context: &RenderContext) -> Result<Rendered> {
    // the rendered html
    let mut html = String::with_capacity(content.len());
    // Set while parsing
    let mut error = None;

    let mut background = IncludeBackground::Yes;
    let mut highlighter: Option<(HighlightLines, bool)> = None;
    // If we get text in header, we need to insert the id and a anchor
    let mut in_header = false;
    // pulldown_cmark can send several text events for a title if there are markdown
    // specific characters like `!` in them. We only want to insert the anchor the first time
    let mut header_created = false;
    let mut anchors: Vec<String> = vec![];

    let mut headers = vec![];
    // Defaults to a 0 level so not a real header
    // It should be an Option ideally but not worth the hassle to update
    let mut temp_header = TempHeader::default();

    let mut opts = Options::empty();
    let mut has_summary = false;
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);

    {
        let parser = Parser::new_ext(content, opts).map(|event| {
            match event {
                Event::Text(text) => {
                    // Header first
                    if in_header {
                        if header_created {
                            temp_header.add_text(&text);
                            return Event::Html(Borrowed(""));
                        }
                        // += as we might have some <code> or other things already there
                        temp_header.add_text(&text);
                        header_created = true;
                        return Event::Html(Borrowed(""));
                    }

                    // if we are in the middle of a code block
                    if let Some((ref mut highlighter, in_extra)) = highlighter {
                        let highlighted = if in_extra {
                            if let Some(ref extra) = context.config.extra_syntax_set {
                                highlighter.highlight(&text, &extra)
                            } else {
                                unreachable!("Got a highlighter from extra syntaxes but no extra?");
                            }
                        } else {
                            highlighter.highlight(&text, &SYNTAX_SET)
                        };
                        //let highlighted = &highlighter.highlight(&text, ss);
                        let html = styled_line_to_highlighted_html(&highlighted, background);
                        return Event::Html(Owned(html));
                    }

                    // Business as usual
                    Event::Text(text)
                }
                Event::Start(Tag::CodeBlock(ref info)) => {
                    if !context.config.highlight_code {
                        return Event::Html(Borrowed("<pre><code>"));
                    }

                    let theme = &THEME_SET.themes[&context.config.highlight_theme];
                    highlighter = Some(get_highlighter(info, &context.config));
                    // This selects the background color the same way that start_coloured_html_snippet does
                    let color =
                        theme.settings.background.unwrap_or(::syntect::highlighting::Color::WHITE);
                    background = IncludeBackground::IfDifferent(color);
                    let snippet = start_highlighted_html_snippet(theme);
                    Event::Html(Owned(snippet.0))
                }
                Event::End(Tag::CodeBlock(_)) => {
                    if !context.config.highlight_code {
                        return Event::Html(Borrowed("</code></pre>\n"));
                    }
                    // reset highlight and close the code block
                    highlighter = None;
                    Event::Html(Borrowed("</pre>"))
                }
                Event::Start(Tag::Image(src, title)) => {
                    if is_colocated_asset_link(&src) {
                        return Event::Start(Tag::Image(
                            Owned(format!("{}{}", context.current_page_permalink, src)),
                            title,
                        ));
                    }

                    Event::Start(Tag::Image(src, title))
                }
                Event::Start(Tag::Link(link, title)) => {
                    // A few situations here:
                    // - it could be a relative link (starting with `./`)
                    // - it could be a link to a co-located asset
                    // - it could be a normal link
                    // - any of those can be in a header or not: if it's in a header
                    //   we need to append to a string
                    let fixed_link = if link.starts_with("./") {
                        match resolve_internal_link(&link, context.permalinks) {
                            Ok(url) => url,
                            Err(_) => {
                                error = Some(format!("Relative link {} not found.", link).into());
                                return Event::Html(Borrowed(""));
                            }
                        }
                    } else if is_colocated_asset_link(&link) {
                        format!("{}{}", context.current_page_permalink, link)
                    } else if context.config.check_external_links
                        && !link.starts_with('#')
                        && !link.starts_with("mailto:")
                    {
                        let res = check_url(&link);
                        if res.is_valid() {
                            link.to_string()
                        } else {
                            error = Some(
                                format!("Link {} is not valid: {}", link, res.message()).into(),
                            );
                            String::new()
                        }
                    } else {
                        link.to_string()
                    };

                    if in_header {
                        let html = if title.is_empty() {
                            format!("<a href=\"{}\">", fixed_link)
                        } else {
                            format!("<a href=\"{}\" title=\"{}\">", fixed_link, title)
                        };
                        temp_header.add_html(&html);
                        return Event::Html(Borrowed(""));
                    }

                    Event::Start(Tag::Link(Owned(fixed_link), title))
                }
                Event::End(Tag::Link(_, _)) => {
                    if in_header {
                        temp_header.add_html("</a>");
                        return Event::Html(Borrowed(""));
                    }
                    event
                }
                Event::Start(Tag::Code) => {
                    if in_header {
                        temp_header.add_html("<code>");
                        return Event::Html(Borrowed(""));
                    }
                    event
                }
                Event::End(Tag::Code) => {
                    if in_header {
                        temp_header.add_html("</code>");
                        return Event::Html(Borrowed(""));
                    }
                    event
                }
                Event::Start(Tag::Header(num)) => {
                    in_header = true;
                    temp_header = TempHeader::new(num);
                    Event::Html(Borrowed(""))
                }
                Event::End(Tag::Header(_)) => {
                    // End of a header, reset all the things and return the header string

                    let id = find_anchor(&anchors, slugify(&temp_header.title), 0);
                    anchors.push(id.clone());
                    temp_header.permalink = format!("{}#{}", context.current_page_permalink, id);
                    temp_header.id = id;

                    in_header = false;
                    header_created = false;
                    let val = temp_header.to_string(context.tera, context.insert_anchor);
                    headers.push(temp_header.clone());
                    temp_header = TempHeader::default();
                    Event::Html(Owned(val))
                }
                Event::Html(ref markup) if markup.contains("<!-- more -->") => {
                    has_summary = true;
                    Event::Html(Borrowed(CONTINUE_READING))
                }
                _ => event,
            }
        });

        cmark::html::push_html(&mut html, parser);
    }

    if let Some(e) = error {
        return Err(e);
    } else {
        Ok(Rendered {
            summary_len: if has_summary { html.find(CONTINUE_READING) } else { None },
            body: html,
            toc: make_table_of_contents(&headers),
        })
    }
}
