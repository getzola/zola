use std::fmt::{self, Write};
use std::borrow::Cow;
use std::collections::HashMap;
use ast::{Content, Node};
use pulldown_cmark::{Alignment, Event, Tag};

pub trait IntoHtml<C> {
    fn render(&mut self, ctx: &mut C, buf: &mut String);
}

enum TagType {
    Opening,
    Closing,
}

struct Context<'a> {
    tag_type: Option<TagType>,
    footnote_indices: HashMap<Cow<'a, str>, usize>,
    in_thead: bool,
    table_column_index: usize,
    table_column_alignments: Option<Vec<Alignment>>,
}

impl<'a> Context<'a> {
    fn new() -> Context<'a> {
        Context {
            tag_type: None,
            footnote_indices: HashMap::new(),
            in_thead: false,
            table_column_index: 0,
            table_column_alignments: None,
        }
    }

    fn render_tag(&self, tag: &str, buf: &mut String) {
        let tag_closer = match self.tag_type {
            Some(TagType::Closing) => "/",
            _ => "",
        };
        buf.push_str(&format!("<{}{}>", tag_closer, tag));
    }

    fn render_nested_tags(&self, tags: &[&str], buf: &mut String) {
        match self.tag_type {
            Some(TagType::Opening) => {
                tags.into_iter().for_each(|t| self.render_tag(t, buf));
            },
            Some(TagType::Closing) => {
                tags.into_iter().rev().for_each(|t| self.render_tag(t, buf));
            },
            None => (),
        };
    }

    fn get_footnote_index(&mut self, id: Cow<'a, str>) -> usize {
        let num_footnotes = self.footnote_indices.len() + 1;
        *self.footnote_indices.entry(id).or_insert(num_footnotes)
    }

    fn render_footnote_reference(&mut self, id: Cow<'a, str>, buf: &mut String) {
        buf.push_str("<sup class=\"footnote-reference\"><a href=\"#");
        // We unwrap here because the String writer implementation will never
        // fail.
        escape_html(buf, &id).unwrap();
        buf.push_str("\">");
        buf.push_str(&*format!("{}", self.get_footnote_index(id)));
        buf.push_str("</a></sup>");
    }

    fn render_footnote_definition(&mut self, id: Cow<'a, str>, buf: &mut String) {
        match self.tag_type {
            Some(TagType::Opening) => {
                buf.push_str(
                    "<div class=\"footnote-definition\" id=\"",
                );
                // We unwrap here because the String writer implementation will never
                // fail.
                escape_html(buf, &*id).unwrap();
                buf.push_str("\"><sup class=\"footnote-definition-label\">");
                buf.push_str(&*format!("{}", self.get_footnote_index(id)));
                buf.push_str("</sup>");
            },
            Some(TagType::Closing) => {
                buf.push_str("</div>")
            },
            None => (),
        }
    }

    fn render_table(&mut self, alignments: Vec<Alignment>, buf: &mut String) {
        match self.tag_type {
            Some(TagType::Opening) => {
                self.table_column_index = 0;
                self.table_column_alignments = Some(alignments);
                buf.push_str("<table>");
            },
            // The parser does not emit tbody events, so for now we are forced
            // to insert a closing tbody tag. This is gross. We feel bad.
            Some(TagType::Closing) => {
                self.table_column_index = 0;
                self.table_column_alignments = None;
                buf.push_str("</tbody></table>")
            },
            None => (),
        }
    }

    fn render_table_head(&mut self, buf: &mut String) {
        // The parser does not emit tr events inside thead (whyyyy?), so for
        // now we are forced to insert an opening and closing tr tag. This is
        // gross. We feel bad.
        match self.tag_type {
            Some(TagType::Opening) => {
                self.in_thead = true;
                buf.push_str("<thead><tr>");
            },
            // The parser does not emit tbody events, so for now we are forced
            // to insert an opening tbody tag. This is gross. We feel bad.
            Some(TagType::Closing) => {
                self.in_thead = false;
                buf.push_str("</tr></thead><tbody>");
            },
            None => (),
        }
    }

    fn render_table_row(&mut self, buf: &mut String) {
        self.table_column_index = 0;
        self.render_tag("tr", buf);
    }

    fn render_table_cell(&mut self, buf: &mut String) {
        // TODO: Consider self.render_tag_with_attr().
        let tag = if self.in_thead { "th" } else { "td" };
        match self.tag_type {
            Some(TagType::Opening) => {
                let attr = match self.table_column_alignments {
                    Some(ref alignments) => {
                        match alignments.get(self.table_column_index) {
                            Some(&Alignment::Left) => " style=\"text-align: left;\"",
                            Some(&Alignment::Center) => " style=\"text-align: center;\"",
                            Some(&Alignment::Right) => " style=\"text-align: right;\"",
                            Some(&Alignment::None) | None => "",
                        }
                    },
                    None => unreachable!(),
                };
                self.table_column_index += 1;
                buf.push_str(&format!("<{}{}>", tag, attr));
            },
            Some(TagType::Closing) => {
                buf.push_str(&format!("</{}>", tag));
            },
            // TODO: Consider unreachable! vs. doing nothing.
            None => (),
        }
    }
}

impl<'a> IntoHtml<Context<'a>> for Tag<'a> {
    fn render(&mut self, context: &mut Context<'a>, buf: &mut String) {
        match *self {
            Tag::Paragraph => context.render_tag("p", buf),
            Tag::Header(n) => context.render_tag(&format!("h{}", n), buf),
            Tag::CodeBlock(ref _info_string) => context.render_nested_tags(&["pre", "code"], buf),
            Tag::FootnoteDefinition(ref id) => context.render_footnote_definition(id.clone(), buf),
            Tag::Table(ref alignments) => context.render_table(alignments.clone(), buf),
            Tag::TableHead => context.render_table_head(buf),
            Tag::TableCell => context.render_table_cell(buf),
            Tag::TableRow => context.render_table_row(buf),
            _ => (),
        }
    }
}

impl<'a> IntoHtml<Context<'a>> for Event<'a> {
    fn render(&mut self, context: &mut Context<'a>, buf: &mut String) {
        match *self {
            Event::Text(ref text) | Event::Html(ref text) | Event::InlineHtml(ref text) => buf.push_str(text),
            Event::FootnoteReference(ref id) => context.render_footnote_reference(id.clone(), buf),
            Event::Start(_) | Event::End(_) => unreachable!(),
            _ => panic!("AHHHHHHH!!!!!!!!!!"),
        }
    }
}

impl<'a> IntoHtml<Context<'a>> for Node<'a> {
    fn render(&mut self, context: &mut Context<'a>, buf: &mut String) {
        match *self {
            Node::Block(ref mut tag, ref mut content) => {
                context.tag_type = Some(TagType::Opening);
                tag.render(context, buf);

                context.tag_type = None;
                content.render(context, buf);

                context.tag_type = Some(TagType::Closing);
                tag.render(context, buf);
                context.tag_type = None;
            },
            Node::Item(ref mut event) => event.render(context, buf),
        }

    }
}

impl<'a, I> IntoHtml<Context<'a>> for Content<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    fn render(&mut self, context: &mut Context<'a>, buf: &mut String) {
        for mut node in self {
            node.render(context, buf);
        }
    }
}

pub fn into_html<'a, I>(content: &mut Content<'a, I>, buf: &mut String)
where
    I: Iterator<Item = Event<'a>>
{
    let mut context = Context::new();
    content.render(&mut context, buf);
}

fn escape_html<W: Write>(buf: &mut W, html: &str) -> Result<(), fmt::Error> {
    for c in html.as_bytes() {
        match *c {
            b'"' => buf.write_str("&quot;")?,
            b'&' => buf.write_str("&amp;")?,
            b'\'' => buf.write_str("&#47;")?,
            b'<' => buf.write_str("&lt;")?,
            b'>' => buf.write_str("&gt;")?,
            _ => buf.write_char(*c as char)?,
        }
    }
    Ok(())
}
