use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use gh_emoji::Replacer as EmojiReplacer;
use giallo::{HtmlRenderer, ParsedFence, parse_markdown_fence};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, LinkType, Parser, Tag, TagEnd};
use pulldown_cmark_escape::{escape_href, escape_html};
use regex::{Regex, RegexBuilder};

use errors::{Error, Result, bail};
use render::render_anchor_link;
use utils::net::is_external_link;
use utils::site::resolve_internal_link;
use utils::slugs::slugify_anchors;
use utils::table_of_contents::{Heading, make_table_of_contents};
use utils::types::InsertAnchor;

use crate::MarkdownContext;

const CONTINUE_READING: &str = "<span id=\"continue-reading\"></span>";
static EMOJI_REPLACER: LazyLock<EmojiReplacer> = LazyLock::new(EmojiReplacer::new);

static FOOTNOTES_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<sup class="footnote-reference"( id=\s*.*?)?><a href=\s*.*?>\s*.*?</a></sup>"#)
        .unwrap()
});

/// Set as a regex to help match some extra cases. This way, spaces and case don't matter.
static MORE_DIVIDER_RE: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(r#"<!--\s*more\s*-->"#)
        .case_insensitive(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap()
});

/// Matches a <a>..</a> tag, getting the opening tag in a capture group.
/// Used only with AnchorInsert::Heading to grab it from the template
static A_HTML_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(<\s*a[^>]*>).*?<\s*/\s*a>").unwrap());

/// Although there exists [a list of registered URI schemes][uri-schemes], a link may use arbitrary,
/// private schemes. This regex checks if the given string starts with something that just looks
/// like a scheme, i.e., a case-insensitive identifier followed by a colon.
///
/// [uri-schemes]: https://www.iana.org/assignments/uri-schemes/uri-schemes.xhtml
static STARTS_WITH_SCHEMA_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9A-Za-z\-]+:").unwrap());

/// Colocated asset links refers to the files in the same directory.
fn is_colocated_asset_link(link: &str) -> bool {
    !link.starts_with('/')
        && !link.starts_with("..")
        && !link.starts_with('#')
        && !STARTS_WITH_SCHEMA_RE.is_match(link)
}

#[inline]
fn escape_html_string(s: &str) -> String {
    let mut out = String::new();
    escape_html(&mut out, s).unwrap();
    out
}

#[inline]
fn escape_href_string(s: &str) -> String {
    let mut out = String::new();
    escape_href(&mut out, s).unwrap();
    out
}

/// We might have cases where the slug is already present in our list of anchor
/// for example an article could have several titles named Example
/// We add a counter after the slug if the slug is already present, which
/// means we will have example, example-1, example-2 etc
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

#[derive(Debug)]
pub struct Rendered {
    pub body: String,
    pub summary: Option<String>,
    pub toc: Vec<Heading>,
    /// Links to site-local pages: relative path plus optional anchor target.
    pub internal_links: Vec<(String, Option<String>)>,
    /// Outgoing links to external webpages (i.e. HTTP(S) targets).
    pub external_links: Vec<String>,
}

#[derive(Debug)]
struct ImageBuffer {
    url: String,
    title: String,
    alt: Vec<Event<'static>>,
}

impl ImageBuffer {
    /// Extract plain text for alt attribute
    fn alt_text(&self) -> String {
        self.alt
            .iter()
            .filter_map(|e| match e {
                Event::Text(t) | Event::Code(t) => Some(t.as_ref()),
                _ => None,
            })
            .collect()
    }
}

/// We need to get all the codeblock content before passing it to giallo
#[derive(Debug)]
struct CodeBlock {
    fence: ParsedFence,
    /// Accumulated text content inside the code block.
    content: String,
}

impl CodeBlock {
    /// Returning an Option<Error> instead of a Result because we do always want to output HTML
    /// The only error being if `error_on_missing_language` is `true`
    fn render(self, ctx: &MarkdownContext) -> (String, Option<Error>) {
        let mut err = None;
        if let Some(hl) = &ctx.config.markdown.highlighting {
            if !hl.registry.contains_grammar(&self.fence.lang) {
                let msg =
                    format!("Language `{}` not found in {:?}", self.fence.lang, ctx.current_path);
                if hl.error_on_missing_language {
                    err = Some(Error::msg(msg));
                } else {
                    log::warn!("{msg}");
                }
            }
            let renderer = HtmlRenderer {
                other_metadata: self.fence.rest,
                css_class_prefix: if hl.uses_classes() { Some("z-".to_string()) } else { None },
                data_attr_position: ctx
                    .config
                    .markdown
                    .highlighting
                    .as_ref()
                    .map(|x| x.data_attr_position.clone())
                    .unwrap_or_default(),
            };
            let out =
                match hl.registry.highlight(&self.content, &hl.highlight_options(&self.fence.lang))
                {
                    Ok(highlighted) => renderer.render(&highlighted, &self.fence.options),
                    Err(e) => {
                        err = Some(Error::msg(e));
                        format!("<pre><code>{}</code></pre>\n", escape_html_string(&self.content))
                    }
                };
            (out, err)
        } else {
            let lang = if self.fence.lang != giallo::PLAIN_GRAMMAR_NAME {
                format!(r#" data-lang="{}""#, self.fence.lang)
            } else {
                String::new()
            };
            (format!("<pre><code{lang}>{}</code></pre>\n", escape_html_string(&self.content)), err)
        }
    }
}

impl From<CodeBlockKind<'_>> for CodeBlock {
    fn from(kind: CodeBlockKind) -> Self {
        let fence = match kind {
            CodeBlockKind::Fenced(info) => parse_markdown_fence(&info),
            _ => ParsedFence::default(),
        };

        Self { fence, content: String::new() }
    }
}

/// Buffers heading because we need to:
/// 1. Extract text to generate the slug/ID
/// 2. Emit the opening <hN id="..."> tag BEFORE the content
/// 3. Insert anchor links in the right position (left/right/wrap)
#[derive(Debug)]
struct HeadingBuffer {
    /// Current heading level (1-6)
    level: u32,
    /// Explicit ID from {#my-id} syntax, if provided.
    explicit_id: Option<String>,
    /// CSS classes from {.class1 .class2} syntax.
    classes: Vec<String>,
    /// Buffered events inside the heading (text, code, emphasis, etc.)
    content: Vec<Event<'static>>,
}

impl HeadingBuffer {
    /// Extract plain text from buffered content for slug generation.
    /// e.g., "Hello **world**" -> "Hello world"
    fn text(&self) -> String {
        self.content
            .iter()
            .filter_map(|e| match e {
                Event::Text(t) | Event::Code(t) => Some(t.as_ref()),
                _ => None,
            })
            .collect()
    }

    /// Build the opening tag: <hN id="..." class="...">
    fn opening_tag(&self, id: &str) -> String {
        let escaped_id = escape_html_string(id);
        if self.classes.is_empty() {
            format!("<h{} id=\"{}\">", self.level, escaped_id)
        } else {
            let escaped_classes: Vec<_> =
                self.classes.iter().map(|c| escape_html_string(c)).collect();
            format!(
                "<h{} id=\"{}\" class=\"{}\">",
                self.level,
                escaped_id,
                escaped_classes.join(" ")
            )
        }
    }
}

#[derive(Debug)]
struct FootnoteDef {
    name: String,
    content: Vec<Event<'static>>,
}

#[derive(Debug, Default)]
pub struct State<'a> {
    output: Vec<Event<'a>>,
    code_block: Option<CodeBlock>,
    heading: Option<HeadingBuffer>,
    image: Option<ImageBuffer>,
    footnote: Option<FootnoteDef>,
    complete_footnotes: Vec<FootnoteDef>,
    /// name -> (number, count)
    footnote_numbers: HashMap<String, (usize, usize)>,
    /// All heading IDs we've generated so far, for collision detection.
    anchors: Vec<String>,
    /// Explicit heading IDs we've already processed (for collision detection)
    seen_explicit_ids: HashSet<String>,
    toc: Vec<Heading>,
    /// At which event we've seen <!-- summary -->
    summary_index: Option<usize>,
    /// Internal links (@/) for validation: (target_path, optional_anchor).
    internal_links: Vec<(String, Option<String>)>,
    /// External links for validation.
    external_links: Vec<String>,
    /// Errors collected during rendering. Combined into one error at the end.
    errors: Vec<Error>,
}

impl<'a> State<'a> {
    #[inline]
    fn push_html(&mut self, html: impl Into<CowStr<'a>>) {
        self.push(Event::Html(html.into()));
    }

    /// We need to buffer a lot of the events for our various custom handling and it should only
    /// be one at a time. Buffers (heading, image, footnote) need 'static events since they outlive
    /// the parser iteration, so we call into_static() only when buffering.
    #[inline]
    fn push(&mut self, event: Event<'a>) {
        if let Some(ref mut block) = self.code_block {
            if let Event::Text(t) = event {
                block.content.push_str(&t);
            }
            return;
        }
        if let Some(ref mut img) = self.image {
            img.alt.push(event.into_static());
        } else if let Some(ref mut h) = self.heading {
            h.content.push(event.into_static());
        } else if let Some(ref mut f) = self.footnote {
            f.content.push(event.into_static());
        } else {
            self.output.push(event);
        }
    }

    pub fn render(mut self, content: &'a str, ctx: &MarkdownContext) -> Result<Rendered> {
        let events: Vec<_> = Parser::new_ext(content, ctx.options()).collect();

        // Pre-scan for explicit heading IDs to reserve them
        for event in &events {
            if let Event::Start(Tag::Heading { id: Some(explicit_id), .. }) = event {
                self.anchors.push(explicit_id.to_string());
            }
        }

        self.output.reserve(events.len());
        for event in events {
            self.process(event, ctx);
        }
        self.finish(ctx)
    }

    fn end_heading(&mut self, ctx: &MarkdownContext) {
        let h = self.heading.take().unwrap();
        let title = h.text();

        // Generate or use explicit ID
        let id = if let Some(ref explicit) = h.explicit_id {
            // insert() returns false if already present = duplicate explicit ID
            if !self.seen_explicit_ids.insert(explicit.clone()) {
                log::warn!("Heading ID '{}' collision in {}", explicit, ctx.current_path);
            }
            explicit.to_owned()
        } else {
            let id =
                find_anchor(&self.anchors, slugify_anchors(&title, ctx.config.slugify.anchors), 0);
            self.anchors.push(id.clone());
            id
        };

        // Opening tag first
        self.output.push(Event::Html(h.opening_tag(&id).into()));

        // Then we have to handle the anchor and content
        if ctx.insert_anchor == InsertAnchor::None {
            self.output.extend(h.content);
        } else {
            let anchor = match render_anchor_link(ctx.tera, &id, h.level, ctx.lang) {
                Ok(a) => Some(a),
                Err(e) => {
                    self.errors.push(e);
                    None
                }
            };
            match ctx.insert_anchor {
                InsertAnchor::Left => {
                    if let Some(a) = anchor {
                        self.push_html(a);
                    }
                    self.output.extend(h.content);
                }
                InsertAnchor::Right => {
                    self.output.extend(h.content);
                    if let Some(a) = anchor {
                        self.push_html(a);
                    }
                }
                InsertAnchor::Heading => {
                    if let Some(a) = anchor {
                        if let Some(caps) = A_HTML_TAG.captures(&a) {
                            self.push_html(caps.get(1).map_or("", |m| m.as_str()).to_string());
                            self.output.extend(h.content);
                            self.push_html("</a>");
                        } else {
                            self.output.extend(h.content);
                        }
                    } else {
                        self.output.extend(h.content);
                    }
                }
                InsertAnchor::None => unreachable!(),
            }
        }

        self.output.push(Event::Html(format!("</h{}>\n", h.level).into()));
        self.toc.push(Heading {
            level: h.level,
            permalink: format!("{}#{}", ctx.current_permalink, id),
            id,
            title,
            // This will be filled later
            children: Vec::new(),
        });
    }

    /// This does 2 things:
    /// 1. resolve Zola internal links + colocated assets + current page anchor
    /// 2. Tracks all links for the link checker
    fn fix_link(
        &mut self,
        ctx: &MarkdownContext,
        link_type: LinkType,
        link: &str,
    ) -> Result<String> {
        if link_type == LinkType::Email {
            return Ok(link.to_string());
        }

        let result = if link.starts_with("@/") {
            match resolve_internal_link(link, ctx.permalinks) {
                Ok(resolved) => {
                    self.internal_links.push((resolved.md_path, resolved.anchor));
                    resolved.permalink
                }
                Err(_) => {
                    let msg = format!("Broken relative link `{}` in {}", link, ctx.current_path,);
                    match ctx.config.link_checker.internal_level {
                        config::LinkCheckerLevel::Error => bail!(msg),
                        config::LinkCheckerLevel::Warn => {
                            log::warn!("{msg}");
                            link.to_string()
                        }
                    }
                }
            }
        } else if is_colocated_asset_link(link) {
            format!("{}{}", ctx.current_permalink, link)
        } else if is_external_link(link) {
            self.external_links.push(link.to_owned());
            link.to_owned()
        } else if link == "#" {
            link.to_string()
        } else if let Some(stripped_link) = link.strip_prefix('#') {
            // local anchor without the internal zola path
            if !ctx.current_path.is_empty() {
                self.internal_links
                    .push((ctx.current_path.to_owned(), Some(stripped_link.to_owned())));
            }
            format!("{}{}", ctx.current_permalink, &link)
        } else {
            link.to_string()
        };

        Ok(result)
    }

    fn start_link(
        &mut self,
        ctx: &MarkdownContext,
        link_type: LinkType,
        dest_url: CowStr<'a>,
        title: CowStr<'a>,
        id: CowStr<'a>,
    ) {
        if dest_url.is_empty() {
            self.errors.push(Error::msg("There is a link that is missing a URL"));
            self.push(Event::Start(Tag::Link { link_type, dest_url: "#".into(), title, id }));
            return;
        }
        let fixed = match self.fix_link(ctx, link_type, dest_url.as_ref()) {
            Ok(url) => url,
            Err(e) => {
                self.errors.push(e);
                return;
            }
        };

        if is_external_link(&dest_url) && ctx.config.markdown.has_external_link_tweaks() {
            let escaped = escape_href_string(&dest_url);
            self.push_html(ctx.config.markdown.construct_external_link_tag(&escaped, &title));
        } else {
            self.push(Event::Start(Tag::Link { link_type, dest_url: fixed.into(), title, id }));
        }
    }

    fn end_image(&mut self, ctx: &MarkdownContext) {
        let img = self.image.take().unwrap();
        let alt = escape_html_string(&img.alt_text());
        let src = escape_href_string(&img.url);
        let title_attr = if img.title.is_empty() {
            String::new()
        } else {
            format!(r#" title="{}""#, escape_html_string(&img.title))
        };
        let html = if ctx.config.markdown.lazy_async_image {
            format!(
                r#"<img src="{src}" alt="{alt}"{title_attr} loading="lazy" decoding="async" />"#
            )
        } else {
            format!(r#"<img src="{src}" alt="{alt}"{title_attr} />"#)
        };
        self.push_html(html);
    }

    fn handle_footnote_ref(&mut self, name: CowStr<'a>) {
        let name = name.to_string();
        let n = self.footnote_numbers.len() + 1;
        let (n, nr) = self.footnote_numbers.entry(name.clone()).or_insert((n, 0));
        *nr += 1;
        let reference = format!(
            r##"<sup class="footnote-reference" id="fr-{name}-{nr}"><a href="#fn-{name}">[{n}]</a></sup>"##
        );
        self.push_html(reference);
    }

    #[inline]
    fn process(&mut self, event: Event<'a>, ctx: &MarkdownContext) {
        match event {
            // Code blocks
            Event::Start(Tag::CodeBlock(kind)) => {
                self.code_block = Some(CodeBlock::from(kind));
            }
            Event::End(TagEnd::CodeBlock) => {
                let code_block = self.code_block.take().unwrap();
                let (out, err) = code_block.render(ctx);
                if let Some(e) = err {
                    self.errors.push(e);
                }
                self.push_html(out);
            }

            // Headings
            Event::Start(Tag::Heading { level, id, classes, .. }) => {
                self.heading = Some(HeadingBuffer {
                    level: level as u32,
                    explicit_id: id.map(|x| x.to_string()),
                    classes: classes.into_iter().map(|c| c.to_string()).collect(),
                    content: Vec::new(),
                });
            }
            Event::End(TagEnd::Heading(_)) => self.end_heading(ctx),

            // Links, we only need the start event
            Event::Start(Tag::Link { link_type, dest_url, title, id }) => {
                self.start_link(ctx, link_type, dest_url, title, id)
            }

            // Images
            Event::Start(Tag::Image { dest_url, title, .. }) => {
                let url = if is_colocated_asset_link(&dest_url) {
                    format!("{}{}", ctx.current_permalink, dest_url)
                } else {
                    dest_url.to_string()
                };
                self.image = Some(ImageBuffer { url, title: title.to_string(), alt: Vec::new() });
            }
            Event::End(TagEnd::Image) => self.end_image(ctx),

            // Footnotes
            Event::Start(Tag::FootnoteDefinition(name)) if ctx.config.markdown.bottom_footnotes => {
                self.footnote = Some(FootnoteDef { name: name.to_string(), content: Vec::new() });
            }
            Event::End(TagEnd::FootnoteDefinition) if ctx.config.markdown.bottom_footnotes => {
                let f = self.footnote.take().unwrap();
                self.complete_footnotes.push(f);
            }
            Event::FootnoteReference(name) if ctx.config.markdown.bottom_footnotes => {
                self.handle_footnote_ref(name)
            }

            // Replacing emojis when not in a code block
            Event::Text(text) => {
                let text = if self.code_block.is_none() && ctx.config.markdown.render_emoji {
                    EMOJI_REPLACER.replace_all(&text).to_string().into()
                } else {
                    text
                };
                self.push(Event::Text(text));
            }

            // HTML (check for summary marker)
            Event::Html(html) | Event::InlineHtml(html) => {
                if self.summary_index.is_none() && MORE_DIVIDER_RE.is_match(&html) {
                    self.summary_index = Some(self.output.len());
                    self.push_html(CONTINUE_READING);
                } else {
                    self.push_html(html);
                }
            }

            // Everything else
            _ => self.push(event),
        }
    }

    fn build_summary(&mut self, ctx: &MarkdownContext) -> Option<String> {
        let idx = self.summary_index?;
        let events = &self.output[..idx];

        // Compute open tags
        let mut open_tags: Vec<TagEnd> = Vec::new();

        for event in events {
            match event {
                Event::Start(Tag::HtmlBlock) | Event::End(TagEnd::HtmlBlock) => (),
                Event::Start(tag) => open_tags.push(tag.to_end()),
                Event::End(tag) => {
                    open_tags.truncate(open_tags.iter().rposition(|t| *t == *tag).unwrap_or(0));
                }
                _ => (),
            }
        }

        // Render to HTML
        let mut html = String::new();
        pulldown_cmark::html::push_html(&mut html, events.iter().cloned());
        // remove footnotes
        html = FOOTNOTES_RE.replace_all(&html, "").into_owned();
        // truncate trailing whitespace
        html.truncate(html.trim_end().len());
        // add cutoff template
        if !open_tags.is_empty() {
            match render::render_summary_cutoff(ctx.tera, &html, ctx.lang) {
                Ok(cutoff) => {
                    html.push_str(&cutoff);
                }
                Err(e) => self.errors.push(e),
            }
        }

        // close remaining tags
        pulldown_cmark::html::push_html(&mut html, open_tags.into_iter().rev().map(Event::End));
        Some(html)
    }

    fn render_footnotes(&mut self, ctx: &MarkdownContext) {
        let mut footnotes: Vec<_> = self
            .complete_footnotes
            .drain(..)
            .filter(|def| self.footnote_numbers.contains_key(&def.name))
            .collect();

        if !ctx.config.markdown.bottom_footnotes || footnotes.is_empty() {
            return;
        }

        footnotes.sort_by_cached_key(|def| {
            self.footnote_numbers.get(&def.name).map(|(n, _)| *n).unwrap_or(0)
        });

        self.push_html("<section class=\"footnotes\">\n<ol class=\"footnotes-list\">\n");

        for f in footnotes {
            let (_, count) = self.footnote_numbers.get(&f.name).copied().unwrap_or((0, 0));
            self.push_html(format!(r#"<li id="fn-{}">"#, f.name));

            let backlinks: String = (1..=count)
                .map(|u| {
                    let suffix = if u == 1 { String::new() } else { u.to_string() };
                    format!(r##" <a href="#fr-{}-{u}">↩{suffix}</a>"##, f.name)
                })
                .collect();

            // We insert the backref in the last paragraph
            let last_p_idx =
                f.content.iter().rposition(|ev| matches!(ev, Event::End(TagEnd::Paragraph)));

            for (i, ev) in f.content.into_iter().enumerate() {
                if Some(i) == last_p_idx {
                    self.push_html(format!("{backlinks}</p>\n"));
                } else {
                    self.output.push(ev);
                }
            }

            if last_p_idx.is_none() {
                self.output.push(Event::Html(backlinks.into()));
            }

            self.push_html("</li>\n");
        }

        self.push_html("</ol>\n</section>\n");
    }

    fn finish(mut self, ctx: &MarkdownContext) -> Result<Rendered> {
        self.render_footnotes(ctx);
        let summary = self.build_summary(ctx);

        if !self.errors.is_empty() {
            return Err(Error::msg(
                self.errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n"),
            ));
        }

        let mut body = String::with_capacity(self.output.len() * 32);
        pulldown_cmark::html::push_html(&mut body, self.output.into_iter());
        Ok(Rendered {
            body,
            summary,
            toc: make_table_of_contents(self.toc),
            internal_links: self.internal_links,
            external_links: self.external_links,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::Config;
    use insta::assert_snapshot;
    use templates::ZOLA_TERA;

    fn make_context<'a>(
        config: &'a Config,
        tera: &'a tera::Tera,
        permalinks: &'a HashMap<String, String>,
    ) -> MarkdownContext<'a> {
        MarkdownContext {
            tera,
            config,
            permalinks,
            lang: &config.default_language,
            current_permalink: "",
            current_path: "",
            insert_anchor: InsertAnchor::None,
        }
    }

    #[test]
    fn test_is_colocated_asset_link_true() {
        let links: [&str; 3] = ["./same-dir.md", "file.md", "qwe.js"];
        for link in links {
            assert!(is_colocated_asset_link(link));
        }
    }

    #[test]
    fn test_is_colocated_asset_link_false() {
        let links: [&str; 2] = ["/other-dir/file.md", "../sub-dir/file.md"];
        for link in links {
            assert!(!is_colocated_asset_link(link));
        }
    }

    #[test]
    fn test_summary_split() {
        let top = "Here's a compelling summary.";
        let top_rendered = format!("<p>{top}</p>");
        let bottom = "Here's the compelling conclusion.";
        let bottom_rendered = format!("<p>{bottom}</p>");
        let mores =
            ["<!-- more -->", "<!--more-->", "<!-- MORE -->", "<!--MORE-->", "<!--\t MoRe \t-->"];
        let config = Config::default();
        let tera = ZOLA_TERA.clone();
        let permalinks = HashMap::new();
        let context = make_context(&config, &tera, &permalinks);
        for more in mores {
            let content = format!("{top}\n\n{more}\n\n{bottom}");
            let rendered = State::default().render(&content, &context).unwrap();
            assert!(rendered.summary.is_some(), "no summary when splitting on {more}");
            let summary = rendered.summary.unwrap();
            let summary = summary.trim();
            let body = rendered.body[summary.len()..].trim();
            let continue_reading = &body[..CONTINUE_READING.len()];
            let body = &body[CONTINUE_READING.len()..].trim();
            assert_eq!(summary, &top_rendered);
            assert_eq!(continue_reading, CONTINUE_READING);
            assert_eq!(body, &bottom_rendered);
        }
    }

    #[test]
    fn no_footnotes() {
        let mut config = Config::default();
        config.markdown.bottom_footnotes = true;
        let tera = ZOLA_TERA.clone();
        let permalinks = HashMap::new();
        let context = make_context(&config, &tera, &permalinks);

        let content = "Some text *without* footnotes.\n\nOnly ~~fancy~~ formatting.";
        let rendered = State::default().render(content, &context).unwrap();
        assert_snapshot!(rendered.body);
    }

    #[test]
    fn single_footnote() {
        let mut config = Config::default();
        config.markdown.bottom_footnotes = true;
        let tera = ZOLA_TERA.clone();
        let permalinks = HashMap::new();
        let context = make_context(&config, &tera, &permalinks);

        let content = "This text has a footnote[^1]\n [^1]:But it is meaningless.";
        let rendered = State::default().render(content, &context).unwrap();
        assert_snapshot!(rendered.body);
    }

    #[test]
    fn reordered_footnotes() {
        let mut config = Config::default();
        config.markdown.bottom_footnotes = true;
        let tera = ZOLA_TERA.clone();
        let permalinks = HashMap::new();
        let context = make_context(&config, &tera, &permalinks);

        let content = "This text has two[^2] footnotes[^1]\n[^1]: not sorted.\n[^2]: But they are";
        let rendered = State::default().render(content, &context).unwrap();
        assert_snapshot!(rendered.body);
    }

    #[test]
    fn def_before_use() {
        let mut config = Config::default();
        config.markdown.bottom_footnotes = true;
        let tera = ZOLA_TERA.clone();
        let permalinks = HashMap::new();
        let context = make_context(&config, &tera, &permalinks);

        let content = "[^1]:It's before the reference.\n\n There is footnote definition?[^1]";
        let rendered = State::default().render(content, &context).unwrap();
        assert_snapshot!(rendered.body);
    }

    #[test]
    fn multiple_refs() {
        let mut config = Config::default();
        config.markdown.bottom_footnotes = true;
        let tera = ZOLA_TERA.clone();
        let permalinks = HashMap::new();
        let context = make_context(&config, &tera, &permalinks);

        let content = "This text has two[^1] identical footnotes[^1]\n[^1]: So one is present.\n[^2]: But another in not.";
        let rendered = State::default().render(content, &context).unwrap();
        assert_snapshot!(rendered.body);
    }

    #[test]
    fn footnote_inside_footnote() {
        let mut config = Config::default();
        config.markdown.bottom_footnotes = true;
        let tera = ZOLA_TERA.clone();
        let permalinks = HashMap::new();
        let context = make_context(&config, &tera, &permalinks);

        let content = "This text has a footnote[^1]\n[^1]: But the footnote has another footnote[^2].\n[^2]: That's it.";
        let rendered = State::default().render(content, &context).unwrap();
        assert_snapshot!(rendered.body);
    }
}
