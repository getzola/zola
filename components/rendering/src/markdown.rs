use pulldown_cmark as cmark;
use slug::slugify;
use syntect::easy::HighlightLines;
use syntect::html::{
    start_highlighted_html_snippet, styled_line_to_highlighted_html, IncludeBackground,
};

use config::highlighting::{get_highlighter, SYNTAX_SET, THEME_SET};
use context::RenderContext;
use errors::{Error, Result};
use front_matter::InsertAnchor;
use table_of_contents::{make_table_of_contents, Header};
use utils::site::resolve_internal_link;
use utils::vec::InsertMany;

use self::cmark::{Event, LinkType, Options, Parser, Tag};

const CONTINUE_READING: &str =
    "<p id=\"zola-continue-reading\"><a name=\"continue-reading\"></a></p>\n";
const ANCHOR_LINK_TEMPLATE: &str = "anchor-link.html";

#[derive(Debug)]
pub struct Rendered {
    pub body: String,
    pub summary_len: Option<usize>,
    pub toc: Vec<Header>,
    pub internal_links_with_anchors: Vec<(String, String)>,
    pub external_links: Vec<String>,
}

// tracks a header in a slice of pulldown-cmark events
#[derive(Debug)]
struct HeaderRef {
    start_idx: usize,
    end_idx: usize,
    level: i32,
    id: Option<String>,
}

impl HeaderRef {
    fn new(start: usize, level: i32) -> HeaderRef {
        HeaderRef { start_idx: start, end_idx: 0, level, id: None }
    }
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

fn fix_link(
    link_type: LinkType,
    link: &str,
    context: &RenderContext,
    internal_links_with_anchors: &mut Vec<(String, String)>,
    external_links: &mut Vec<String>,
) -> Result<String> {
    if link_type == LinkType::Email {
        return Ok(link.to_string());
    }
    // A few situations here:
    // - it could be a relative link (starting with `@/`)
    // - it could be a link to a co-located asset
    // - it could be a normal link
    let result = if link.starts_with("@/") {
        match resolve_internal_link(&link, context.permalinks) {
            Ok(resolved) => {
                if resolved.anchor.is_some() {
                    internal_links_with_anchors
                        .push((resolved.md_path.unwrap(), resolved.anchor.unwrap()));
                }
                resolved.permalink
            }
            Err(_) => {
                return Err(format!("Relative link {} not found.", link).into());
            }
        }
    } else if is_colocated_asset_link(&link) {
        format!("{}{}", context.current_page_permalink, link)
    } else {
        if !link.starts_with('#') && !link.starts_with("mailto:") {
            external_links.push(link.to_owned());
        }
        link.to_string()
    };
    Ok(result)
}

/// get only text in a slice of events
fn get_text(parser_slice: &[Event]) -> String {
    let mut title = String::new();

    for event in parser_slice.iter() {
        match event {
            Event::Text(text) | Event::Code(text) => title += text,
            _ => continue,
        }
    }

    title
}

fn get_header_refs(events: &[Event]) -> Vec<HeaderRef> {
    let mut header_refs = vec![];

    for (i, event) in events.iter().enumerate() {
        match event {
            Event::Start(Tag::Header(level)) => {
                header_refs.push(HeaderRef::new(i, *level));
            }
            Event::End(Tag::Header(_)) => {
                let msg = "Header end before start?";
                header_refs.last_mut().expect(msg).end_idx = i;
            }
            _ => (),
        }
    }

    header_refs
}

pub fn markdown_to_html(content: &str, context: &RenderContext) -> Result<Rendered> {
    // the rendered html
    let mut html = String::with_capacity(content.len());
    // Set while parsing
    let mut error = None;

    let mut background = IncludeBackground::Yes;
    let mut highlighter: Option<(HighlightLines, bool)> = None;

    let mut inserted_anchors: Vec<String> = vec![];
    let mut headers: Vec<Header> = vec![];
    let mut internal_links_with_anchors = Vec::new();
    let mut external_links = Vec::new();

    let mut opts = Options::empty();
    let mut has_summary = false;
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);

    {
        let mut events = Parser::new_ext(content, opts)
            .map(|event| {
                match event {
                    Event::Text(text) => {
                        // if we are in the middle of a code block
                        if let Some((ref mut highlighter, in_extra)) = highlighter {
                            let highlighted = if in_extra {
                                if let Some(ref extra) = context.config.extra_syntax_set {
                                    highlighter.highlight(&text, &extra)
                                } else {
                                    unreachable!(
                                        "Got a highlighter from extra syntaxes but no extra?"
                                    );
                                }
                            } else {
                                highlighter.highlight(&text, &SYNTAX_SET)
                            };
                            //let highlighted = &highlighter.highlight(&text, ss);
                            let html = styled_line_to_highlighted_html(&highlighted, background);
                            return Event::Html(html.into());
                        }

                        // Business as usual
                        Event::Text(text)
                    }
                    Event::Start(Tag::CodeBlock(ref info)) => {
                        if !context.config.highlight_code {
                            return Event::Html("<pre><code>".into());
                        }

                        let theme = &THEME_SET.themes[&context.config.highlight_theme];
                        highlighter = Some(get_highlighter(info, &context.config));
                        // This selects the background color the same way that start_coloured_html_snippet does
                        let color = theme
                            .settings
                            .background
                            .unwrap_or(::syntect::highlighting::Color::WHITE);
                        background = IncludeBackground::IfDifferent(color);
                        let snippet = start_highlighted_html_snippet(theme);
                        Event::Html(snippet.0.into())
                    }
                    Event::End(Tag::CodeBlock(_)) => {
                        if !context.config.highlight_code {
                            return Event::Html("</code></pre>\n".into());
                        }
                        // reset highlight and close the code block
                        highlighter = None;
                        Event::Html("</pre>".into())
                    }
                    Event::Start(Tag::Image(link_type, src, title)) => {
                        if is_colocated_asset_link(&src) {
                            let link = format!("{}{}", context.current_page_permalink, &*src);
                            return Event::Start(Tag::Image(link_type, link.into(), title));
                        }

                        Event::Start(Tag::Image(link_type, src, title))
                    }
                    Event::Start(Tag::Link(link_type, link, title)) => {
                        let fixed_link = match fix_link(
                            link_type,
                            &link,
                            context,
                            &mut internal_links_with_anchors,
                            &mut external_links,
                        ) {
                            Ok(fixed_link) => fixed_link,
                            Err(err) => {
                                error = Some(err);
                                return Event::Html("".into());
                            }
                        };

                        Event::Start(Tag::Link(link_type, fixed_link.into(), title))
                    }
                    Event::Html(ref markup) if markup.contains("<!-- more -->") => {
                        has_summary = true;
                        Event::Html(CONTINUE_READING.into())
                    }
                    _ => event,
                }
            })
            .collect::<Vec<_>>(); // We need to collect the events to make a second pass

        let mut header_refs = get_header_refs(&events);

        let mut anchors_to_insert = vec![];

        // First header pass: look for a manually-specified IDs, e.g. `# Heading text {#hash}`
        // (This is a separate first pass so that auto IDs can avoid collisions with manual IDs.)
        for header_ref in header_refs.iter_mut() {
            let end_idx = header_ref.end_idx;
            if let Event::Text(ref mut text) = events[end_idx - 1] {
                if text.as_bytes().last() == Some(&b'}') {
                    if let Some(mut i) = text.find("{#") {
                        let id = text[i + 2..text.len() - 1].to_owned();
                        inserted_anchors.push(id.clone());
                        while i > 0 && text.as_bytes()[i - 1] == b' ' {
                            i -= 1;
                        }
                        header_ref.id = Some(id);
                        *text = text[..i].to_owned().into();
                    }
                }
            }
        }

        // Second header pass: auto-generate remaining IDs, and emit HTML
        for header_ref in header_refs {
            let start_idx = header_ref.start_idx;
            let end_idx = header_ref.end_idx;
            let title = get_text(&events[start_idx + 1..end_idx]);
            let id =
                header_ref.id.unwrap_or_else(|| find_anchor(&inserted_anchors, slugify(&title), 0));
            inserted_anchors.push(id.clone());

            // insert `id` to the tag
            let html = format!("<h{lvl} id=\"{id}\">", lvl = header_ref.level, id = id);
            events[start_idx] = Event::Html(html.into());

            // generate anchors and places to insert them
            if context.insert_anchor != InsertAnchor::None {
                let anchor_idx = match context.insert_anchor {
                    InsertAnchor::Left => start_idx + 1,
                    InsertAnchor::Right => end_idx,
                    InsertAnchor::None => 0, // Not important
                };
                let mut c = tera::Context::new();
                c.insert("id", &id);

                let anchor_link = utils::templates::render_template(
                    &ANCHOR_LINK_TEMPLATE,
                    context.tera,
                    c,
                    &None,
                )
                .map_err(|e| Error::chain("Failed to render anchor link template", e))?;
                anchors_to_insert.push((anchor_idx, Event::Html(anchor_link.into())));
            }

            // record header to make table of contents
            let permalink = format!("{}#{}", context.current_page_permalink, id);
            let h = Header { level: header_ref.level, id, permalink, title, children: Vec::new() };
            headers.push(h);
        }

        if context.insert_anchor != InsertAnchor::None {
            events.insert_many(anchors_to_insert);
        }

        cmark::html::push_html(&mut html, events.into_iter());
    }

    if let Some(e) = error {
        return Err(e);
    } else {
        Ok(Rendered {
            summary_len: if has_summary { html.find(CONTINUE_READING) } else { None },
            body: html,
            toc: make_table_of_contents(headers),
            internal_links_with_anchors,
            external_links,
        })
    }
}
