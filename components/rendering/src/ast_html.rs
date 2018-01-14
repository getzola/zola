use ast::{Content, Node};
use pulldown_cmark::{Event, Tag};

pub trait IntoHtml<C> {
    fn render(&mut self, ctx: &mut C, buf: &mut String);
}

enum TagType {
    Opening,
    Closing,
}

struct Context {
    tag_type: Option<TagType>,
}

impl Context {
    fn new() -> Context {
        Context {
            tag_type: None,
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
}

impl<'a> IntoHtml<Context> for Tag<'a> {
    fn render(&mut self, context: &mut Context, buf: &mut String) {
        match *self {
            Tag::Paragraph => context.render_tag("p", buf),
            Tag::Header(n) => context.render_tag(&format!("h{}", n), buf),
            Tag::CodeBlock(ref _info_string) => context.render_nested_tags(&["pre", "code"], buf),
            _ => (),
        }
    }
}

impl<'a> IntoHtml<Context> for Event<'a> {
    fn render(&mut self, _context: &mut Context, buf: &mut String) {
        match *self {
            Event::Start(_) | Event::End(_) => unreachable!(),
            Event::Text(ref text) => buf.push_str(text),
            _ => panic!("AHHHHHHH!!!!!!!!!!"),
        }
    }
}

impl<'a> IntoHtml<Context> for Node<'a> {
    fn render(&mut self, context: &mut Context, buf: &mut String) {
        match *self {
            Node::Block(ref mut tag, ref mut content) => {
                context.tag_type = Some(TagType::Opening);
                tag.render(context, buf);

                context.tag_type = None;
                content.render(context, buf);

                context.tag_type = Some(TagType::Closing);
                tag.render(context, buf);
                buf.push('\n');
                context.tag_type = None;
            },
            Node::Item(ref mut event) => event.render(context, buf),
        }

    }
}

impl<'a, I> IntoHtml<Context> for Content<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    fn render(&mut self, context: &mut Context, buf: &mut String) {
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
