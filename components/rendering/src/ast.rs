use pulldown_cmark::{Event, Tag};

use std::iter::{Iterator, Peekable};
use std::marker::PhantomData;
use std::mem::discriminant;
use std::vec::IntoIter;

use collect_while::collect_while;

#[derive(Debug)]
pub enum Node<'a> {
    Block(Tag<'a>, Content<'a, IntoIter<Event<'a>>>),
    Item(Event<'a>),
}

impl<'a> Node<'a> {
    pub fn try_from<I>(iter: &mut Peekable<I>) -> Option<Node<'a>>
    where
        I: Iterator<Item = Event<'a>>,
    {
        match iter.next() {
            Some(Event::Start(start_tag)) => {
                let content: Vec<_> = collect_while(iter, |event| {
                    match *event {
                        Event::End(ref end_tag) =>
                            discriminant(&start_tag) != discriminant(&end_tag),
                        _ => true,
                    }
                });
                Node::Block(
                        start_tag,
                        Content::new(content.into_iter()),
                ).into()
            },
            Some(Event::End(_)) => Node::try_from(iter),
            Some(event) => Some(Node::Item(event)),
            None => None
        }
    }
}

#[derive(Debug)]
pub struct Content<'a, I>
where I: Iterator<Item = Event<'a>> {
    iter: Peekable<I>,
    _t: PhantomData<&'a str>,
}

impl<'a, I> Content<'a, I>
where I: Iterator<Item = Event<'a>> {
    pub fn new(iter: I) -> Content<'a, I> {
        Content {
            iter: iter.peekable(),
            _t: PhantomData,
        }
    }
}

impl<'a, I> Iterator for Content<'a, I>
where I: Iterator<Item = Event<'a>> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Node<'a>> {
        Node::try_from(&mut self.iter)
    }
}
