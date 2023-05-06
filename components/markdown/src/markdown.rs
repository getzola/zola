use std::fmt::Write;

use errors::bail;
use libs::gh_emoji::Replacer as EmojiReplacer;
use libs::once_cell::sync::Lazy;
use libs::pulldown_cmark as cmark;
use libs::tera;
use utils::net::is_external_link;

use crate::context::RenderContext;
use errors::{Context, Error, Result};
use libs::pulldown_cmark::escape::escape_html;
use libs::regex::Regex;
use utils::site::resolve_internal_link;
use utils::slugs::slugify_anchors;
use utils::table_of_contents::{make_table_of_contents, Heading};
use utils::types::InsertAnchor;

use self::cmark::{Event, LinkType, Options, Parser, Tag};
use crate::codeblock::{CodeBlock, FenceSettings};
use crate::shortcode::{Shortcode, SHORTCODE_PLACEHOLDER};

const CONTINUE_READING: &str = "<span id=\"continue-reading\"></span>";
const ANCHOR_LINK_TEMPLATE: &str = "anchor-link.html";
static EMOJI_REPLACER: Lazy<EmojiReplacer> = Lazy::new(EmojiReplacer::new);

/// Although there exists [a list of registered URI schemes][uri-schemes], a link may use arbitrary,
/// private schemes. This regex checks if the given string starts with something that just looks
/// like a scheme, i.e., a case-insensitive identifier followed by a colon.
///
/// [uri-schemes]: https://www.iana.org/assignments/uri-schemes/uri-schemes.xhtml
static STARTS_WITH_SCHEMA_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[0-9A-Za-z\-]+:").unwrap());

/// Matches a <a>..</a> tag, getting the opening tag in a capture group.
/// Used only with AnchorInsert::Heading to grab it from the template
static A_HTML_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"(<\s*a[^>]*>).*?<\s*/\s*a>").unwrap());

/// Efficiently insert multiple element in their specified index.
/// The elements should sorted in ascending order by their index.
///
/// This is done in O(n) time.
fn insert_many<T>(input: &mut Vec<T>, elem_to_insert: Vec<(usize, T)>) {
    let mut inserted = vec![];
    let mut last_idx = 0;

    for (idx, elem) in elem_to_insert.into_iter() {
        let head_len = idx - last_idx;
        inserted.extend(input.splice(0..head_len, std::iter::empty()));
        inserted.push(elem);
        last_idx = idx;
    }
    let len = input.len();
    inserted.extend(input.drain(0..len));

    *input = inserted;
}

/// Colocated asset links refers to the files in the same directory.
fn is_colocated_asset_link(link: &str) -> bool {
    !link.starts_with('/')
        && !link.starts_with("..")
        && !link.starts_with('#')
        && !STARTS_WITH_SCHEMA_RE.is_match(link)
}

#[derive(Debug)]
pub struct Rendered {
    pub body: String,
    pub summary_len: Option<usize>,
    pub toc: Vec<Heading>,
    /// Links to site-local pages: relative path plus optional anchor target.
    pub internal_links: Vec<(String, Option<String>)>,
    /// Outgoing links to external webpages (i.e. HTTP(S) targets).
    pub external_links: Vec<String>,
}

/// Tracks a heading in a slice of pulldown-cmark events
#[derive(Debug)]
struct HeadingRef {
    start_idx: usize,
    end_idx: usize,
    level: u32,
    id: Option<String>,
    classes: Vec<String>,
}

impl HeadingRef {
    fn new(start: usize, level: u32, anchor: Option<String>, classes: &[String]) -> HeadingRef {
        HeadingRef { start_idx: start, end_idx: 0, level, id: anchor, classes: classes.to_vec() }
    }

    fn to_html(&self, id: &str) -> String {
        let mut buffer = String::with_capacity(100);
        buffer.write_str("<h").unwrap();
        buffer.write_str(&format!("{}", self.level)).unwrap();

        buffer.write_str(" id=\"").unwrap();
        escape_html(&mut buffer, id).unwrap();
        buffer.write_str("\"").unwrap();

        if !self.classes.is_empty() {
            buffer.write_str(" class=\"").unwrap();
            let num_classes = self.classes.len();

            for (i, class) in self.classes.iter().enumerate() {
                escape_html(&mut buffer, class).unwrap();
                if i < num_classes - 1 {
                    buffer.write_str(" ").unwrap();
                }
            }

            buffer.write_str("\"").unwrap();
        }

        buffer.write_str(">").unwrap();
        buffer
    }
}

// We might have cases where the slug is already present in our list of anchor
// for example an article could have several titles named Example
// We add a counter after the slug if the slug is already present, which
// means we will have example, example-1, example-2 etc
fn find_anchor(anchors: &[String], name: String, level: u16) -> String {
    if level == 0 && !anchors.contains(&name) {
        return name;
    }

    let new_anchor = format!("{}-{}", name, level + 1);
    if !anchors.contains(&new_anchor) {
        return new_anchor;
    }

    find_anchor(anchors, name, level + 1)
}

fn fix_link(
    link_type: LinkType,
    link: &str,
    context: &RenderContext,
    internal_links: &mut Vec<(String, Option<String>)>,
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
        match resolve_internal_link(link, &context.permalinks) {
            Ok(resolved) => {
                internal_links.push((resolved.md_path, resolved.anchor));
                resolved.permalink
            }
            Err(_) => {
                let msg = format!(
                    "Broken relative link `{}` in {}",
                    link,
                    context.current_page_path.unwrap_or("unknown"),
                );
                match context.config.link_checker.internal_level {
                    config::LinkCheckerLevel::Error => bail!(msg),
                    config::LinkCheckerLevel::Warn => {
                        console::warn(&msg);
                        link.to_string()
                    }
                }
            }
        }
    } else if is_colocated_asset_link(link) {
        format!("{}{}", context.current_page_permalink, link)
    } else if is_external_link(link) {
        external_links.push(link.to_owned());
        link.to_owned()
    } else if link == "#" {
        link.to_string()
    } else if let Some(stripped_link) = link.strip_prefix('#') {
        // local anchor without the internal zola path
        if let Some(current_path) = context.current_page_path {
            internal_links.push((current_path.to_owned(), Some(stripped_link.to_owned())));
            format!("{}{}", context.current_page_permalink, &link)
        } else {
            link.to_string()
        }
    } else {
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
            Event::Start(Tag::Heading(level, anchor, classes)) => {
                heading_refs.push(HeadingRef::new(
                    i,
                    *level as u32,
                    anchor.map(|a| a.to_owned()),
                    &classes.iter().map(|x| x.to_string()).collect::<Vec<_>>(),
                ));
            }
            Event::End(Tag::Heading(_, _, _)) => {
                heading_refs.last_mut().expect("Heading end before start?").end_idx = i;
            }
            _ => (),
        }
    }

    heading_refs
}

pub fn markdown_to_html(
    content: &str,
    context: &RenderContext,
    html_shortcodes: Vec<Shortcode>,
) -> Result<Rendered> {
    let path = context
        .tera_context
        .get("page")
        .or_else(|| context.tera_context.get("section"))
        .map(|x| x.as_object().unwrap().get("relative_path").unwrap().as_str().unwrap());
    // the rendered html
    let mut html = String::with_capacity(content.len());
    // Set while parsing
    let mut error = None;

    let mut code_block: Option<CodeBlock> = None;

    let mut headings: Vec<Heading> = vec![];
    let mut internal_links = Vec::new();
    let mut external_links = Vec::new();

    let mut stop_next_end_p = false;

    let lazy_async_image = context.config.markdown.lazy_async_image;

    let mut opts = Options::empty();
    let mut has_summary = false;
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    if context.config.markdown.smart_punctuation {
        opts.insert(Options::ENABLE_SMART_PUNCTUATION);
    }

    // we reverse their order so we can pop them easily in order
    let mut html_shortcodes: Vec<_> = html_shortcodes.into_iter().rev().collect();
    let mut next_shortcode = html_shortcodes.pop();
    let contains_shortcode = |txt: &str| -> bool { txt.contains(SHORTCODE_PLACEHOLDER) };

    {
        let mut events = Vec::new();
        macro_rules! render_shortcodes {
            ($is_text:expr, $text:expr, $range:expr) => {
                let orig_range_start = $range.start;
                loop {
                    if let Some(ref shortcode) = next_shortcode {
                        if !$range.contains(&shortcode.span.start) {
                            break;
                        }
                        let sc_span = shortcode.span.clone();

                        // we have some text before the shortcode, push that first
                        if $range.start != sc_span.start {
                            let content = $text[($range.start - orig_range_start)
                                ..(sc_span.start - orig_range_start)]
                                .to_string()
                                .into();
                            events.push(if $is_text {
                                Event::Text(content)
                            } else {
                                Event::Html(content)
                            });
                            $range.start = sc_span.start;
                        }

                        // Now we should be at the same idx as the shortcode
                        let shortcode = next_shortcode.take().unwrap();
                        match shortcode.render(&context.tera, &context.tera_context) {
                            Ok(s) => {
                                events.push(Event::Html(s.into()));
                                $range.start += SHORTCODE_PLACEHOLDER.len();
                            }
                            Err(e) => {
                                error = Some(e);
                                break;
                            }
                        }
                        next_shortcode = html_shortcodes.pop();
                        continue;
                    }

                    break;
                }

                if !$range.is_empty() {
                    // The $range value is for the whole document, not for this slice of text
                    let content = $text[($range.start - orig_range_start)..].to_string().into();
                    events.push(if $is_text { Event::Text(content) } else { Event::Html(content) });
                }
            };
        }

        let mut accumulated_block = String::new();
        for (event, mut range) in Parser::new_ext(content, opts).into_offset_iter() {
            match event {
                Event::Text(text) => {
                    if let Some(ref mut _code_block) = code_block {
                        if contains_shortcode(text.as_ref()) {
                            // mark the start of the code block events
                            let stack_start = events.len();
                            render_shortcodes!(true, text, range);
                            // after rendering the shortcodes we will collect all the text events
                            // and re-render them as code blocks
                            for event in events[stack_start..].iter() {
                                match event {
                                    Event::Html(t) | Event::Text(t) => accumulated_block += t,
                                    _ => {
                                        error = Some(Error::msg(format!(
                                            "Unexpected event while expanding the code block: {:?}",
                                            event
                                        )));
                                        break;
                                    }
                                }
                            }

                            // remove all the original events from shortcode rendering
                            events.truncate(stack_start);
                        } else {
                            accumulated_block += &text;
                        }
                    } else {
                        let text = if context.config.markdown.render_emoji {
                            EMOJI_REPLACER.replace_all(&text).to_string().into()
                        } else {
                            text
                        };

                        if !contains_shortcode(text.as_ref()) {
                            events.push(Event::Text(text));
                            continue;
                        }

                        render_shortcodes!(true, text, range);
                    }
                }
                Event::Start(Tag::CodeBlock(ref kind)) => {
                    let fence = match kind {
                        cmark::CodeBlockKind::Fenced(fence_info) => FenceSettings::new(fence_info),
                        _ => FenceSettings::new(""),
                    };
                    let (block, begin) = CodeBlock::new(fence, context.config, path);
                    code_block = Some(block);
                    events.push(Event::Html(begin.into()));
                }
                Event::End(Tag::CodeBlock(_)) => {
                    if let Some(ref mut code_block) = code_block {
                        let html = code_block.highlight(&accumulated_block);
                        events.push(Event::Html(html.into()));
                        accumulated_block.clear();
                    }

                    // reset highlight and close the code block
                    code_block = None;
                    events.push(Event::Html("</code></pre>\n".into()));
                }
                Event::Start(Tag::Image(link_type, src, title)) => {
                    let link = if is_colocated_asset_link(&src) {
                        let link = format!("{}{}", context.current_page_permalink, &*src);
                        link.into()
                    } else {
                        src
                    };

                    events.push(if lazy_async_image {
                        let mut img_before_alt: String = "<img src=\"".to_string();
                        cmark::escape::escape_href(&mut img_before_alt, &link)
                            .expect("Could not write to buffer");
                        if !title.is_empty() {
                            img_before_alt
                                .write_str("\" title=\"")
                                .expect("Could not write to buffer");
                            cmark::escape::escape_href(&mut img_before_alt, &title)
                                .expect("Could not write to buffer");
                        }
                        img_before_alt.write_str("\" alt=\"").expect("Could not write to buffer");
                        Event::Html(img_before_alt.into())
                    } else {
                        Event::Start(Tag::Image(link_type, link, title))
                    });
                }
                Event::End(Tag::Image(..)) => events.push(if lazy_async_image {
                    Event::Html("\" loading=\"lazy\" decoding=\"async\" />".into())
                } else {
                    event
                }),
                Event::Start(Tag::Link(link_type, link, title)) if link.is_empty() => {
                    error = Some(Error::msg("There is a link that is missing a URL"));
                    events.push(Event::Start(Tag::Link(link_type, "#".into(), title)));
                }
                Event::Start(Tag::Link(link_type, link, title)) => {
                    let fixed_link = match fix_link(
                        link_type,
                        &link,
                        context,
                        &mut internal_links,
                        &mut external_links,
                    ) {
                        Ok(fixed_link) => fixed_link,
                        Err(err) => {
                            error = Some(err);
                            events.push(Event::Html("".into()));
                            continue;
                        }
                    };

                    events.push(
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
                        },
                    )
                }
                Event::Start(Tag::Paragraph) => {
                    // We have to compare the start and the trimmed length because the content
                    // will sometimes contain '\n' at the end which we want to avoid.
                    //
                    // NOTE: It could be more efficient to remove this search and just keep
                    // track of the shortcodes to come and compare it to that.
                    if let Some(ref next_shortcode) = next_shortcode {
                        if next_shortcode.span.start == range.start
                            && next_shortcode.span.len() == content[range].trim().len()
                        {
                            stop_next_end_p = true;
                            events.push(Event::Html("".into()));
                            continue;
                        }
                    }

                    events.push(event);
                }
                Event::End(Tag::Paragraph) => {
                    events.push(if stop_next_end_p {
                        stop_next_end_p = false;
                        Event::Html("".into())
                    } else {
                        event
                    });
                }
                Event::Html(text) => {
                    if text.contains("<!-- more -->") {
                        has_summary = true;
                        events.push(Event::Html(CONTINUE_READING.into()));
                        continue;
                    }
                    if !contains_shortcode(text.as_ref()) {
                        events.push(Event::Html(text));
                        continue;
                    }

                    render_shortcodes!(false, text, range);
                }
                _ => events.push(event),
            }
        }

        // We remove all the empty things we might have pushed before so we don't get some random \n
        events.retain(|e| match e {
            Event::Text(text) | Event::Html(text) => !text.is_empty(),
            _ => true,
        });

        let heading_refs = get_heading_refs(&events);

        let mut anchors_to_insert = vec![];
        let mut inserted_anchors = vec![];
        for heading in &heading_refs {
            if let Some(s) = &heading.id {
                inserted_anchors.push(s.to_owned());
            }
        }

        // Second heading pass: auto-generate remaining IDs, and emit HTML
        for mut heading_ref in heading_refs {
            let start_idx = heading_ref.start_idx;
            let end_idx = heading_ref.end_idx;
            let title = get_text(&events[start_idx + 1..end_idx]);

            if heading_ref.id.is_none() {
                heading_ref.id = Some(find_anchor(
                    &inserted_anchors,
                    slugify_anchors(&title, context.config.slugify.anchors),
                    0,
                ));
            }

            inserted_anchors.push(heading_ref.id.clone().unwrap());
            let id = inserted_anchors.last().unwrap();

            let html = heading_ref.to_html(id);
            events[start_idx] = Event::Html(html.into());

            // generate anchors and places to insert them
            if context.insert_anchor != InsertAnchor::None {
                let anchor_idx = match context.insert_anchor {
                    InsertAnchor::Left => start_idx + 1,
                    InsertAnchor::Right => end_idx,
                    InsertAnchor::Heading => 0, // modified later to the correct value
                    InsertAnchor::None => unreachable!(),
                };
                let mut c = tera::Context::new();
                c.insert("id", &id);
                c.insert("level", &heading_ref.level);
                c.insert("lang", &context.lang);

                let anchor_link = utils::templates::render_template(
                    ANCHOR_LINK_TEMPLATE,
                    &context.tera,
                    c,
                    &None,
                )
                .context("Failed to render anchor link template")?;
                if context.insert_anchor != InsertAnchor::Heading {
                    anchors_to_insert.push((anchor_idx, Event::Html(anchor_link.into())));
                } else if let Some(captures) = A_HTML_TAG.captures(&anchor_link) {
                    let opening_tag = captures.get(1).map_or("", |m| m.as_str()).to_string();
                    anchors_to_insert.push((start_idx + 1, Event::Html(opening_tag.into())));
                    anchors_to_insert.push((end_idx, Event::Html("</a>".into())));
                }
            }

            // record heading to make table of contents
            let permalink = format!("{}#{}", context.current_page_permalink, id);
            let h = Heading {
                level: heading_ref.level,
                id: id.to_owned(),
                permalink,
                title,
                children: Vec::new(),
            };
            headings.push(h);
        }

        if context.insert_anchor != InsertAnchor::None {
            insert_many(&mut events, anchors_to_insert);
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
            internal_links,
            external_links,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]

    fn insert_many_works() {
        let mut v = vec![1, 2, 3, 4, 5];
        insert_many(&mut v, vec![(0, 0), (2, -1), (5, 6)]);
        assert_eq!(v, &[0, 1, 2, -1, 3, 4, 5, 6]);

        let mut v2 = vec![1, 2, 3, 4, 5];
        insert_many(&mut v2, vec![(0, 0), (2, -1)]);
        assert_eq!(v2, &[0, 1, 2, -1, 3, 4, 5]);
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

    #[test]
    // Tests for link  that points to files in the same directory
    fn test_is_colocated_asset_link_true() {
        let links: [&str; 3] = ["./same-dir.md", "file.md", "qwe.js"];
        for link in links {
            assert!(is_colocated_asset_link(link));
        }
    }

    #[test]
    // Tests for files where the link points to a different directory
    fn test_is_colocated_asset_link_false() {
        let links: [&str; 2] = ["/other-dir/file.md", "../sub-dir/file.md"];
        for link in links {
            assert!(!is_colocated_asset_link(link));
        }
    }
}
