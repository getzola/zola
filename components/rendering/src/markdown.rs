use lazy_static::lazy_static;
use pulldown_cmark as cmark;
use regex::Regex;
use syntect::html::{start_highlighted_html_snippet, IncludeBackground};

use crate::context::RenderContext;
use crate::table_of_contents::{make_table_of_contents, Heading};
use config::highlighting::THEME_SET;
use errors::{Error, Result};
use front_matter::InsertAnchor;
use utils::site::resolve_internal_link;
use utils::slugs::slugify_anchors;
use utils::vec::InsertMany;

use self::cmark::{Event, LinkType, Options, Parser, Tag};

mod codeblock;
mod fence;
use self::codeblock::CodeBlock;

const CONTINUE_READING: &str = "<span id=\"continue-reading\"></span>";
const ANCHOR_LINK_TEMPLATE: &str = "anchor-link.html";

#[derive(Debug)]
pub struct Rendered {
    pub body: String,
    pub summary_len: Option<usize>,
    pub toc: Vec<Heading>,
    pub internal_links_with_anchors: Vec<(String, String)>,
    pub external_links: Vec<String>,
}

// tracks a heading in a slice of pulldown-cmark events
#[derive(Debug)]
struct HeadingRef {
    start_idx: usize,
    end_idx: usize,
    level: u32,
    id: Option<String>,
}

impl HeadingRef {
    fn new(start: usize, level: u32) -> HeadingRef {
        HeadingRef { start_idx: start, end_idx: 0, level, id: None }
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

// Returns whether the given string starts with a schema.
//
// Although there exists [a list of registered URI schemes][uri-schemes], a link may use arbitrary,
// private schemes. This function checks if the given string starts with something that just looks
// like a scheme, i.e., a case-insensitive identifier followed by a colon.
//
// [uri-schemes]: https://www.iana.org/assignments/uri-schemes/uri-schemes.xhtml
fn starts_with_schema(s: &str) -> bool {
    lazy_static! {
        static ref PATTERN: Regex = Regex::new(r"^[0-9A-Za-z\-]+:").unwrap();
    }

    PATTERN.is_match(s)
}

// Colocated asset links refers to the files in the same directory,
// there it should be a filename only
fn is_colocated_asset_link(link: &str) -> bool {
    !link.contains('/')  // http://, ftp://, ../ etc
        && !starts_with_schema(link)
}

// Returns whether a link starts with an HTTP(s) scheme.
fn is_external_link(link: &str) -> bool {
    link.starts_with("http:") || link.starts_with("https:")
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
        match resolve_internal_link(&link, &context.permalinks) {
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
        if is_external_link(link) {
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

fn get_heading_refs(events: &[Event]) -> Vec<HeadingRef> {
    let mut heading_refs = vec![];

    for (i, event) in events.iter().enumerate() {
        match event {
            Event::Start(Tag::Heading(level)) => {
                heading_refs.push(HeadingRef::new(i, *level));
            }
            Event::End(Tag::Heading(_)) => {
                let msg = "Heading end before start?";
                heading_refs.last_mut().expect(msg).end_idx = i;
            }
            _ => (),
        }
    }

    heading_refs
}

pub fn markdown_to_html(content: &str, context: &RenderContext) -> Result<Rendered> {
    lazy_static! {
        static ref EMOJI_REPLACER: gh_emoji::Replacer = gh_emoji::Replacer::new();
    }

    // the rendered html
    let mut html = String::with_capacity(content.len());
    // Set while parsing
    let mut error = None;

    let mut highlighter: Option<CodeBlock> = None;

    let mut inserted_anchors: Vec<String> = vec![];
    let mut headings: Vec<Heading> = vec![];
    let mut internal_links_with_anchors = Vec::new();
    let mut external_links = Vec::new();

    let mut opts = Options::empty();
    let mut has_summary = false;
    let mut in_html_block = false;
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);

    if context.config.markdown.smart_punctuation {
        opts.insert(Options::ENABLE_SMART_PUNCTUATION);
    }

    {
        let mut events = Parser::new_ext(content, opts)
            .map(|event| {
                match event {
                    Event::Text(text) => {
                        // if we are in the middle of a highlighted code block
                        if let Some(ref mut code_block) = highlighter {
                            let html = code_block.highlight(&text);
                            Event::Html(html.into())
                        } else if context.config.markdown.render_emoji {
                            let processed_text = EMOJI_REPLACER.replace_all(&text);
                            Event::Text(processed_text.to_string().into())
                        } else {
                            // Business as usual
                            Event::Text(text)
                        }
                    }
                    Event::Start(Tag::CodeBlock(ref kind)) => {
                        let language = match kind {
                            cmark::CodeBlockKind::Fenced(fence_info) => {
                                let fence_info = fence::FenceSettings::new(fence_info);
                                fence_info.language
                            }
                            _ => None,
                        };

                        if !context.config.highlight_code() {
                            if let Some(lang) = language {
                                let html = format!(
                                    r#"<pre><code class="language-{}" data-lang="{}">"#,
                                    lang, lang
                                );
                                return Event::Html(html.into());
                            }
                            return Event::Html("<pre><code>".into());
                        }

                        let theme = &THEME_SET.themes[context.config.highlight_theme()];
                        match kind {
                            cmark::CodeBlockKind::Indented => (),
                            cmark::CodeBlockKind::Fenced(fence_info) => {
                                // This selects the background color the same way that
                                // start_coloured_html_snippet does
                                let color = theme
                                    .settings
                                    .background
                                    .unwrap_or(::syntect::highlighting::Color::WHITE);

                                highlighter = Some(CodeBlock::new(
                                    fence_info,
                                    &context.config,
                                    IncludeBackground::IfDifferent(color),
                                ));
                            }
                        };
                        let snippet = start_highlighted_html_snippet(theme);
                        let mut html = snippet.0;
                        if let Some(lang) = language {
                            html.push_str(&format!(
                                r#"<code class="language-{}" data-lang="{}">"#,
                                lang, lang
                            ));
                        } else {
                            html.push_str("<code>");
                        }
                        Event::Html(html.into())
                    }
                    Event::End(Tag::CodeBlock(_)) => {
                        if !context.config.highlight_code() {
                            return Event::Html("</code></pre>\n".into());
                        }
                        // reset highlight and close the code block
                        highlighter = None;
                        Event::Html("</code></pre>".into())
                    }
                    Event::Start(Tag::Image(link_type, src, title)) => {
                        if is_colocated_asset_link(&src) {
                            let link = format!("{}{}", context.current_page_permalink, &*src);
                            return Event::Start(Tag::Image(link_type, link.into(), title));
                        }

                        Event::Start(Tag::Image(link_type, src, title))
                    }
                    Event::Start(Tag::Link(link_type, link, title)) if link.is_empty() => {
                        error = Some(Error::msg("There is a link that is missing a URL"));
                        Event::Start(Tag::Link(link_type, "#".into(), title))
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
                        if is_external_link(&link)
                            && context.config.markdown.has_external_link_tweaks()
                        {
                            let mut escaped = String::new();
                            // write_str can fail but here there are no reasons it should (afaik?)
                            cmark::escape::escape_href(&mut escaped, &link)
                                .expect("Could not write to buffer");
                            Event::Html(
                                context
                                    .config
                                    .markdown
                                    .construct_external_link_tag(&escaped, &title)
                                    .into(),
                            )
                        } else {
                            Event::Start(Tag::Link(link_type, fixed_link.into(), title))
                        }
                    }
                    Event::Html(ref markup) => {
                        if markup.contains("<!-- more -->") {
                            has_summary = true;
                            Event::Html(CONTINUE_READING.into())
                        } else if in_html_block && markup.contains("</pre>") {
                            in_html_block = false;
                            Event::Html(markup.replacen("</pre>", "", 1).into())
                        } else if markup.contains("pre data-shortcode") {
                            in_html_block = true;
                            let m = markup.replacen("<pre data-shortcode>", "", 1);
                            if m.contains("</pre>") {
                                in_html_block = false;
                                Event::Html(m.replacen("</pre>", "", 1).into())
                            } else {
                                Event::Html(m.into())
                            }
                        } else {
                            event
                        }
                    }
                    _ => event,
                }
            })
            .collect::<Vec<_>>(); // We need to collect the events to make a second pass

        let mut heading_refs = get_heading_refs(&events);

        let mut anchors_to_insert = vec![];

        // First heading pass: look for a manually-specified IDs, e.g. `# Heading text {#hash}`
        // (This is a separate first pass so that auto IDs can avoid collisions with manual IDs.)
        for heading_ref in heading_refs.iter_mut() {
            let end_idx = heading_ref.end_idx;
            if let Event::Text(ref mut text) = events[end_idx - 1] {
                if text.as_bytes().last() == Some(&b'}') {
                    if let Some(mut i) = text.find("{#") {
                        let id = text[i + 2..text.len() - 1].to_owned();
                        inserted_anchors.push(id.clone());
                        while i > 0 && text.as_bytes()[i - 1] == b' ' {
                            i -= 1;
                        }
                        heading_ref.id = Some(id);
                        *text = text[..i].to_owned().into();
                    }
                }
            }
        }

        // Second heading pass: auto-generate remaining IDs, and emit HTML
        for heading_ref in heading_refs {
            let start_idx = heading_ref.start_idx;
            let end_idx = heading_ref.end_idx;
            let title = get_text(&events[start_idx + 1..end_idx]);
            let id = heading_ref.id.unwrap_or_else(|| {
                find_anchor(
                    &inserted_anchors,
                    slugify_anchors(&title, context.config.slugify.anchors),
                    0,
                )
            });
            inserted_anchors.push(id.clone());

            // insert `id` to the tag
            let html = format!("<h{lvl} id=\"{id}\">", lvl = heading_ref.level, id = id);
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
                c.insert("level", &heading_ref.level);

                let anchor_link = utils::templates::render_template(
                    &ANCHOR_LINK_TEMPLATE,
                    &context.tera,
                    c,
                    &None,
                )
                .map_err(|e| Error::chain("Failed to render anchor link template", e))?;
                anchors_to_insert.push((anchor_idx, Event::Html(anchor_link.into())));
            }

            // record heading to make table of contents
            let permalink = format!("{}#{}", context.current_page_permalink, id);
            let h =
                Heading { level: heading_ref.level, id, permalink, title, children: Vec::new() };
            headings.push(h);
        }

        if context.insert_anchor != InsertAnchor::None {
            events.insert_many(anchors_to_insert);
        }

        cmark::html::push_html(&mut html, events.into_iter());
    }

    if let Some(e) = error {
        Err(e)
    } else {
        Ok(Rendered {
            summary_len: if has_summary { html.find(CONTINUE_READING) } else { None },
            body: html,
            toc: make_table_of_contents(headings),
            internal_links_with_anchors,
            external_links,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starts_with_schema() {
        // registered
        assert!(starts_with_schema("https://example.com/"));
        assert!(starts_with_schema("ftp://example.com/"));
        assert!(starts_with_schema("mailto:user@example.com"));
        assert!(starts_with_schema("xmpp:node@example.com"));
        assert!(starts_with_schema("tel:18008675309"));
        assert!(starts_with_schema("sms:18008675309"));
        assert!(starts_with_schema("h323:user@example.com"));

        // arbitrary
        assert!(starts_with_schema("zola:post?content=hi"));

        // case-insensitive
        assert!(starts_with_schema("MailTo:user@example.com"));
        assert!(starts_with_schema("MAILTO:user@example.com"));
    }

    #[test]
    fn test_is_external_link() {
        assert!(is_external_link("http://example.com/"));
        assert!(is_external_link("https://example.com/"));
        assert!(is_external_link("https://example.com/index.html#introduction"));

        assert!(!is_external_link("mailto:user@example.com"));
        assert!(!is_external_link("tel:18008675309"));

        assert!(!is_external_link("#introduction"));

        assert!(!is_external_link("http.jpg"))
    }
}
