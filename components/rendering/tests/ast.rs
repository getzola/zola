extern crate rendering;
extern crate pulldown_cmark;

use std::fmt::Debug;

use pulldown_cmark::{Parser, Event, Tag};
use rendering::ast::{Content, Node};

fn print_iter<I, D>(iter: I)
    where
    D: Debug,
    I: Iterator<Item = D>
{
    for i in iter {
        println!("{:?}", i);
    }
}

#[test]
fn ast_contents_first_node() {
    let original = r##"# [link](/to/here)
"##;

    let p = Parser::new(&original);
    let mut content = Content::new(p);
    let content_head = content.next();
    assert!(content_head.is_some());
    match content_head {
        Some(Node::Block(tag, _)) =>
            match tag {
                Tag::Header(n) => assert_eq!(n, 1),
                _ => panic!("Expected a header tag, got {:?}", tag),
            },
        _ => assert!(false)
    }


}

#[test]
fn first_child_of_first_node() {
    let original = r##"# [link](/to/here)
    "##;

    let p = Parser::new(&original);
    let v: Vec<_>  = p.collect();
    print_iter(v.iter());
    println!("\n\n");

    let mut content = Content::new(v.into_iter());

    match content.next() {
        Some(Node::Block(_, mut content)) => {
            println!("{:?}", content);
            match content.next() {
                Some(Node::Block(first_child, _)) => {
                    match first_child {
                        Tag::Link(href, _title) => assert_eq!(href, "/to/here"),
                        _ => panic!("Expected a link tag, got {:?}", first_child),
                    }
                },
                Some(Node::Item(e)) => {
                    match e {
                        Event::SoftBreak => assert!(true),
                        _ => panic!("Expected a soft break event, got {:?}", e),
                    }
                },
                None => assert!(false),
            }
        },
        _ => assert!(false),
    }
}

#[test]
fn ast_contents_second_node() {
    let original = r##"# [link](/to/here)

Hello
"##;

    let p = Parser::new(&original);
    let v: Vec<_>  = p.collect();
    print_iter(v.iter());
    println!("\n\n");

    let mut content = Content::new(v.into_iter());
    println!("{:?}\n\n", content);
    let content_head = {
        content.next(); // Skip the first Node
        println!("{:?}\n\n", content);
        content.next()
    };
    println!("{:?}\n\n", content);
    assert!(content_head.is_some());
    match content_head {
        Some(Node::Block(tag, _)) =>
            match tag {
                Tag::Paragraph => assert!(true),
                _ => panic!("Expected a paragraph tag, got {:?}", tag),
            }
        _ => assert!(false)
    }


}
