use std::borrow::Cow::Owned;

use pulldown_cmark as cmark;
use self::cmark::{Parser, Event, Tag, Options, OPTION_ENABLE_TABLES, OPTION_ENABLE_FOOTNOTES};
use regex::Regex;
use slug::slugify;
use syntect::easy::HighlightLines;
use syntect::html::{start_coloured_html_snippet, styles_to_coloured_html, IncludeBackground};
use tera::{Context as TeraContext};

use errors::{Result};
use site::resolve_internal_link;
use front_matter::InsertAnchor;
use rendering::context::Context;
use rendering::highlighting::THEME_SET;
use rendering::parsing::SYNTAX_SET;
use rendering::short_code::{ShortCode, parse_shortcode, render_simple_shortcode};
use content::{TempHeader, Header, make_table_of_contents};

lazy_static!{
    static ref SHORTCODE_RE: Regex = Regex::new(r#"\{(?:%|\{)\s+([[:alnum:]]+?)\(([[:alnum:]]+?="?.+?"?)\)\s+(?:%|\})\}"#).unwrap();
}


pub fn markdown_to_html(content: &str, context: &Context) -> Result<(String, Vec<Header>)> {
    // We try to be smart about highlighting code as it can be time-consuming
    // If the global config disables it, then we do nothing. However,
    // if we see a code block in the content, we assume that this page needs
    // to be highlighted. It could potentially have false positive if the content
    // has ``` in it but that seems kind of unlikely
    let should_highlight = if context.highlight_code {
        content.contains("```")
    } else {
        false
    };
    // Set while parsing
    let mut error = None;
    let mut highlighter: Option<HighlightLines> = None;
    let mut shortcode_block = None;
    // shortcodes live outside of paragraph so we need to ensure we don't close
    // a paragraph that has already been closed
    let mut added_shortcode = false;
    // Don't transform things that look like shortcodes in code blocks
    let mut in_code_block = false;
    // If we get text in header, we need to insert the id and a anchor
    let mut in_header = false;
    // pulldown_cmark can send several text events for a title if there are markdown
    // specific characters like `!` in them. We only want to insert the anchor the first time
    let mut header_already_inserted = false;
    let mut anchors: Vec<String> = vec![];

    // the rendered html
    let mut html = String::new();

    // We might have cases where the slug is already present in our list of anchor
    // for example an article could have several titles named Example
    // We add a counter after the slug if the slug is already present, which
    // means we will have example, example-1, example-2 etc
    fn find_anchor(anchors: &[String], name: String, level: u8) -> String {
        if level == 0 && !anchors.contains(&name) {
            return name.to_string();
        }

        let new_anchor = format!("{}-{}", name, level + 1);
        if !anchors.contains(&new_anchor) {
            return new_anchor;
        }

        find_anchor(anchors, name, level + 1)
    }

    let mut headers = vec![];
    // Defaults to a 0 level so not a real header
    // It should be an Option ideally but not worth the hassle to update
    let mut temp_header = TempHeader::default();

    let mut opts = Options::empty();
    opts.insert(OPTION_ENABLE_TABLES);
    opts.insert(OPTION_ENABLE_FOOTNOTES);

    {
        let parser = Parser::new_ext(content, opts).map(|event| match event {
            Event::Text(text) => {
                // if we are in the middle of a code block
                if let Some(ref mut highlighter) = highlighter {
                    let highlighted = &highlighter.highlight(&text);
                    let html = styles_to_coloured_html(highlighted, IncludeBackground::Yes);
                    return Event::Html(Owned(html));
                }

                if in_code_block {
                    return Event::Text(text);
                }

                // Shortcode without body
                if shortcode_block.is_none() && text.starts_with("{{") && text.ends_with("}}") && SHORTCODE_RE.is_match(&text) {
                    let (name, args) = parse_shortcode(&text);
                    added_shortcode = true;
                    match render_simple_shortcode(context.tera, &name, &args) {
                        Ok(s) => return Event::Html(Owned(format!("</p>{}", s))),
                        Err(e) => {
                            error = Some(e);
                            return Event::Html(Owned("".to_string()));
                        }
                    }
                    // non-matching will be returned normally below
                }

                // Shortcode with a body
                if shortcode_block.is_none() && text.starts_with("{%") && text.ends_with("%}") {
                    if SHORTCODE_RE.is_match(&text) {
                        let (name, args) = parse_shortcode(&text);
                        shortcode_block = Some(ShortCode::new(&name, args));
                    }
                    // Don't return anything
                    return Event::Text(Owned("".to_string()));
                }

                // If we have some text while in a shortcode, it's either the body
                // or the end tag
                if shortcode_block.is_some() {
                    if let Some(ref mut shortcode) = shortcode_block {
                        if text.trim() == "{% end %}" {
                            added_shortcode = true;
                            match shortcode.render(context.tera) {
                                Ok(s) => return Event::Html(Owned(format!("</p>{}", s))),
                                Err(e) => {
                                    error = Some(e);
                                    return Event::Html(Owned("".to_string()));
                                }
                            }
                        } else {
                            shortcode.append(&text);
                            return Event::Html(Owned("".to_string()));
                        }
                    }
                }

                if in_header {
                    if header_already_inserted {
                        return Event::Text(text);
                    }
                    let id = find_anchor(&anchors, slugify(&text), 0);
                    anchors.push(id.clone());
                    let anchor_link = if context.should_insert_anchor() {
                        let mut c = TeraContext::new();
                        c.add("id", &id);
                        context.tera.render("anchor-link.html", &c).unwrap()
                    } else {
                        String::new()
                    };
                    // update the header and add it to the list
                    temp_header.id = id.clone();
                    temp_header.title = text.clone().into_owned();
                    temp_header.permalink = format!("{}#{}", context.current_page_permalink, id);
                    headers.push(temp_header.clone());
                    temp_header = TempHeader::default();

                    header_already_inserted = true;
                    let event = match context.insert_anchor {
                        InsertAnchor::Left => Event::Html(Owned(format!(r#"id="{}">{}{}"#, id, anchor_link, text))),
                        InsertAnchor::Right => Event::Html(Owned(format!(r#"id="{}">{}{}"#, id, text, anchor_link))),
                        InsertAnchor::None => Event::Html(Owned(format!(r#"id="{}">{}"#, id, text)))
                    };
                    return event;
                }

                // Business as usual
                Event::Text(text)
            },
            Event::Start(Tag::CodeBlock(ref info)) => {
                in_code_block = true;
                if !should_highlight {
                    return Event::Html(Owned("<pre><code>".to_owned()));
                }
                let theme = &THEME_SET.themes[&context.highlight_theme];
                highlighter = SYNTAX_SET.with(|ss| {
                    let syntax = info
                        .split(' ')
                        .next()
                        .and_then(|lang| ss.find_syntax_by_token(lang))
                        .unwrap_or_else(|| ss.find_syntax_plain_text());
                    Some(HighlightLines::new(syntax, theme))
                });
                let snippet = start_coloured_html_snippet(theme);
                Event::Html(Owned(snippet))
            },
            Event::End(Tag::CodeBlock(_)) => {
                in_code_block = false;
                if !should_highlight{
                    return Event::Html(Owned("</code></pre>\n".to_owned()))
                }
                // reset highlight and close the code block
                highlighter = None;
                Event::Html(Owned("</pre>".to_owned()))
            },
            // Need to handle relative links
            Event::Start(Tag::Link(ref link, ref title)) => {
                if in_header {
                    return Event::Html(Owned("".to_owned()));
                }
                if link.starts_with("./") {
                    match resolve_internal_link(link, context.permalinks) {
                        Ok(url) => {
                            return Event::Start(Tag::Link(Owned(url), title.clone()));
                        },
                        Err(_) => {
                            error = Some(format!("Relative link {} not found.", link).into());
                            return Event::Html(Owned("".to_string()));
                        }
                    };
                }

                Event::Start(Tag::Link(link.clone(), title.clone()))
            },
            Event::End(Tag::Link(_, _)) => {
                if in_header {
                    return Event::Html(Owned("".to_owned()));
                }
                event
            }
            // need to know when we are in a code block to disable shortcodes in them
            Event::Start(Tag::Code) => {
                in_code_block = true;
                event
            },
            Event::End(Tag::Code) => {
                in_code_block = false;
                event
            },
            Event::Start(Tag::Header(num)) => {
                in_header = true;
                temp_header = TempHeader::new(num);
                // ugly eh
                Event::Html(Owned(format!("<h{} ", num)))
            },
            Event::End(Tag::Header(_)) => {
                in_header = false;
                header_already_inserted = false;
                event
            },
            // If we added shortcodes, don't close a paragraph since there's none
            Event::End(Tag::Paragraph) => {
                if added_shortcode {
                    added_shortcode = false;
                    return Event::Html(Owned("".to_owned()));
                }
                event
            },
            // Ignore softbreaks inside shortcodes
            Event::SoftBreak => {
                if shortcode_block.is_some() {
                    return Event::Html(Owned("".to_owned()));
                }
                event
            },
             _ => {
                 // println!("event = {:?}", event);
                 event
             },
        });

        cmark::html::push_html(&mut html, parser);
    }

    match error {
        Some(e) => Err(e),
        None => Ok((html.replace("<p></p>", ""), make_table_of_contents(headers))),
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tera::Tera;

    use config::Config;
    use front_matter::InsertAnchor;
    use templates::GUTENBERG_TERA;
    use rendering::context::Context;

    use super::markdown_to_html;

    #[test]
    fn can_do_markdown_to_html_simple() {
        let tera_ctx = Tera::default();
        let permalinks_ctx = HashMap::new();
        let config_ctx = Config::default();
        let context = Context::new(&tera_ctx, &config_ctx, "", &permalinks_ctx, InsertAnchor::None);
        let res = markdown_to_html("hello", &context).unwrap();
        assert_eq!(res.0, "<p>hello</p>\n");
    }

    #[test]
    fn doesnt_highlight_code_block_with_highlighting_off() {
        let tera_ctx = Tera::default();
        let permalinks_ctx = HashMap::new();
        let config_ctx = Config::default();
        let mut context = Context::new(&tera_ctx, &config_ctx, "", &permalinks_ctx, InsertAnchor::None);
        context.highlight_code = false;
        let res = markdown_to_html("```\n$ gutenberg server\n```", &context).unwrap();
        assert_eq!(
            res.0,
            "<pre><code>$ gutenberg server\n</code></pre>\n"
        );
    }

    #[test]
    fn can_highlight_code_block_no_lang() {
        let tera_ctx = Tera::default();
        let permalinks_ctx = HashMap::new();
        let config_ctx = Config::default();
        let context = Context::new(&tera_ctx, &config_ctx, "", &permalinks_ctx, InsertAnchor::None);
        let res = markdown_to_html("```\n$ gutenberg server\n$ ping\n```", &context).unwrap();
        assert_eq!(
            res.0,
            "<pre style=\"background-color:#2b303b\">\n<span style=\"background-color:#2b303b;color:#c0c5ce;\">$ gutenberg server\n</span><span style=\"background-color:#2b303b;color:#c0c5ce;\">$ ping\n</span></pre>"
        );
    }

    #[test]
    fn can_highlight_code_block_with_lang() {
        let tera_ctx = Tera::default();
        let permalinks_ctx = HashMap::new();
        let config_ctx = Config::default();
        let context = Context::new(&tera_ctx, &config_ctx, "", &permalinks_ctx, InsertAnchor::None);
        let res = markdown_to_html("```python\nlist.append(1)\n```", &context).unwrap();
        assert_eq!(
            res.0,
            "<pre style=\"background-color:#2b303b\">\n<span style=\"background-color:#2b303b;color:#c0c5ce;\">list</span><span style=\"background-color:#2b303b;color:#c0c5ce;\">.</span><span style=\"background-color:#2b303b;color:#bf616a;\">append</span><span style=\"background-color:#2b303b;color:#c0c5ce;\">(</span><span style=\"background-color:#2b303b;color:#d08770;\">1</span><span style=\"background-color:#2b303b;color:#c0c5ce;\">)</span><span style=\"background-color:#2b303b;color:#c0c5ce;\">\n</span></pre>"
        );
    }

    #[test]
    fn can_higlight_code_block_with_unknown_lang() {
        let tera_ctx = Tera::default();
        let permalinks_ctx = HashMap::new();
        let config_ctx = Config::default();
        let context = Context::new(&tera_ctx, &config_ctx, "", &permalinks_ctx, InsertAnchor::None);
        let res = markdown_to_html("```yolo\nlist.append(1)\n```", &context).unwrap();
        // defaults to plain text
        assert_eq!(
            res.0,
            "<pre style=\"background-color:#2b303b\">\n<span style=\"background-color:#2b303b;color:#c0c5ce;\">list.append(1)\n</span></pre>"
        );
    }

    #[test]
    fn can_render_shortcode() {
        let permalinks_ctx = HashMap::new();
        let config_ctx = Config::default();
        let context = Context::new(&GUTENBERG_TERA, &config_ctx, "", &permalinks_ctx, InsertAnchor::None);
        let res = markdown_to_html(r#"
Hello

{{ youtube(id="ub36ffWAqgQ") }}
        "#, &context).unwrap();
        assert!(res.0.contains("<p>Hello</p>\n<div >"));
        assert!(res.0.contains(r#"<iframe src="https://www.youtube.com/embed/ub36ffWAqgQ""#));
    }

    #[test]
    fn can_render_several_shortcode_in_row() {
        let permalinks_ctx = HashMap::new();
        let config_ctx = Config::default();
        let context = Context::new(&GUTENBERG_TERA, &config_ctx, "", &permalinks_ctx, InsertAnchor::None);
        let res = markdown_to_html(r#"
Hello

{{ youtube(id="ub36ffWAqgQ") }}

{{ youtube(id="ub36ffWAqgQ", autoplay=true) }}

{{ vimeo(id="210073083") }}

{{ streamable(id="c0ic") }}

{{ gist(url="https://gist.github.com/Keats/32d26f699dcc13ebd41b") }}

        "#, &context).unwrap();
        assert!(res.0.contains("<p>Hello</p>\n<div >"));
        assert!(res.0.contains(r#"<iframe src="https://www.youtube.com/embed/ub36ffWAqgQ""#));
        assert!(res.0.contains(r#"<iframe src="https://www.youtube.com/embed/ub36ffWAqgQ?autoplay=1""#));
        assert!(res.0.contains(r#"<iframe src="https://www.streamable.com/e/c0ic""#));
        assert!(res.0.contains(r#"//player.vimeo.com/video/210073083""#));
    }

    #[test]
    fn doesnt_render_shortcode_in_code_block() {
        let permalinks_ctx = HashMap::new();
        let config_ctx = Config::default();
        let context = Context::new(&GUTENBERG_TERA, &config_ctx, "", &permalinks_ctx, InsertAnchor::None);
        let res = markdown_to_html(r#"```{{ youtube(id="w7Ft2ymGmfc") }}```"#, &context).unwrap();
        assert_eq!(res.0, "<p><code>{{ youtube(id=&quot;w7Ft2ymGmfc&quot;) }}</code></p>\n");
    }

    #[test]
    fn can_render_shortcode_with_body() {
        let mut tera = Tera::default();
        tera.extend(&GUTENBERG_TERA).unwrap();
        tera.add_raw_template("shortcodes/quote.html", "<blockquote>{{ body }} - {{ author}}</blockquote>").unwrap();
        let permalinks_ctx = HashMap::new();
        let config_ctx = Config::default();
        let context = Context::new(&tera, &config_ctx, "", &permalinks_ctx, InsertAnchor::None);

        let res = markdown_to_html(r#"
Hello
{% quote(author="Keats") %}
A quote
{% end %}
        "#, &context).unwrap();
        assert_eq!(res.0, "<p>Hello\n</p><blockquote>A quote - Keats</blockquote>");
    }

    #[test]
    fn errors_rendering_unknown_shortcode() {
        let tera_ctx = Tera::default();
        let permalinks_ctx = HashMap::new();
        let config_ctx = Config::default();
        let context = Context::new(&tera_ctx, &config_ctx, "", &permalinks_ctx, InsertAnchor::None);
        let res = markdown_to_html("{{ hello(flash=true) }}", &context);
        assert!(res.is_err());
    }

    #[test]
    fn can_make_valid_relative_link() {
        let mut permalinks = HashMap::new();
        permalinks.insert("pages/about.md".to_string(), "https://vincent.is/about".to_string());
        let tera_ctx = Tera::default();
        let config_ctx = Config::default();
        let context = Context::new(&tera_ctx, &config_ctx, "", &permalinks, InsertAnchor::None);
        let res = markdown_to_html(
            r#"[rel link](./pages/about.md), [abs link](https://vincent.is/about)"#,
            &context
        ).unwrap();

        assert!(
            res.0.contains(r#"<p><a href="https://vincent.is/about">rel link</a>, <a href="https://vincent.is/about">abs link</a></p>"#)
        );
    }

    #[test]
    fn can_make_relative_links_with_anchors() {
        let mut permalinks = HashMap::new();
        permalinks.insert("pages/about.md".to_string(), "https://vincent.is/about".to_string());
        let tera_ctx = Tera::default();
        let config_ctx = Config::default();
        let context = Context::new(&tera_ctx, &config_ctx, "", &permalinks, InsertAnchor::None);
        let res = markdown_to_html(r#"[rel link](./pages/about.md#cv)"#, &context).unwrap();

        assert!(
            res.0.contains(r#"<p><a href="https://vincent.is/about#cv">rel link</a></p>"#)
        );
    }

    #[test]
    fn errors_relative_link_inexistant() {
        let tera_ctx = Tera::default();
        let permalinks_ctx = HashMap::new();
        let config_ctx = Config::default();
        let context = Context::new(&tera_ctx, &config_ctx, "", &permalinks_ctx, InsertAnchor::None);
        let res = markdown_to_html("[rel link](./pages/about.md)", &context);
        assert!(res.is_err());
    }

    #[test]
    fn can_add_id_to_headers() {
        let tera_ctx = Tera::default();
        let permalinks_ctx = HashMap::new();
        let config_ctx = Config::default();
        let context = Context::new(&tera_ctx, &config_ctx, "", &permalinks_ctx, InsertAnchor::None);
        let res = markdown_to_html(r#"# Hello"#, &context).unwrap();
        assert_eq!(res.0, "<h1 id=\"hello\">Hello</h1>\n");
    }

    #[test]
    fn can_add_id_to_headers_same_slug() {
        let tera_ctx = Tera::default();
        let permalinks_ctx = HashMap::new();
        let config_ctx = Config::default();
        let context = Context::new(&tera_ctx, &config_ctx, "", &permalinks_ctx, InsertAnchor::None);
        let res = markdown_to_html("# Hello\n# Hello", &context).unwrap();
        assert_eq!(res.0, "<h1 id=\"hello\">Hello</h1>\n<h1 id=\"hello-1\">Hello</h1>\n");
    }

    #[test]
    fn can_insert_anchor_left() {
        let permalinks_ctx = HashMap::new();
        let config_ctx = Config::default();
        let context = Context::new(&GUTENBERG_TERA, &config_ctx, "", &permalinks_ctx, InsertAnchor::Left);
        let res = markdown_to_html("# Hello", &context).unwrap();
        assert_eq!(
            res.0,
            "<h1 id=\"hello\"><a class=\"gutenberg-anchor\" href=\"#hello\" aria-label=\"Anchor link for: hello\">🔗</a>\nHello</h1>\n"
        );
    }

    #[test]
    fn can_insert_anchor_right() {
        let permalinks_ctx = HashMap::new();
        let config_ctx = Config::default();
        let context = Context::new(&GUTENBERG_TERA, &config_ctx, "", &permalinks_ctx, InsertAnchor::Right);
        let res = markdown_to_html("# Hello", &context).unwrap();
        assert_eq!(
            res.0,
            "<h1 id=\"hello\">Hello<a class=\"gutenberg-anchor\" href=\"#hello\" aria-label=\"Anchor link for: hello\">🔗</a>\n</h1>\n"
        );
    }

    // See https://github.com/Keats/gutenberg/issues/42
    #[test]
    fn can_insert_anchor_with_exclamation_mark() {
        let permalinks_ctx = HashMap::new();
        let config_ctx = Config::default();
        let context = Context::new(&GUTENBERG_TERA, &config_ctx, "", &permalinks_ctx, InsertAnchor::Left);
        let res = markdown_to_html("# Hello!", &context).unwrap();
        assert_eq!(
            res.0,
            "<h1 id=\"hello\"><a class=\"gutenberg-anchor\" href=\"#hello\" aria-label=\"Anchor link for: hello\">🔗</a>\nHello!</h1>\n"
        );
    }

    // See https://github.com/Keats/gutenberg/issues/53
    #[test]
    fn can_insert_anchor_with_link() {
        let permalinks_ctx = HashMap::new();
        let config_ctx = Config::default();
        let context = Context::new(&GUTENBERG_TERA, &config_ctx, "", &permalinks_ctx, InsertAnchor::Left);
        let res = markdown_to_html("## [](#xresources)Xresources", &context).unwrap();
        assert_eq!(
            res.0,
            "<h2 id=\"xresources\"><a class=\"gutenberg-anchor\" href=\"#xresources\" aria-label=\"Anchor link for: xresources\">🔗</a>\nXresources</h2>\n"
        );
    }

    #[test]
    fn can_insert_anchor_with_other_special_chars() {
        let permalinks_ctx = HashMap::new();
        let config_ctx = Config::default();
        let context = Context::new(&GUTENBERG_TERA, &config_ctx, "", &permalinks_ctx, InsertAnchor::Left);
        let res = markdown_to_html("# Hello*_()", &context).unwrap();
        assert_eq!(
            res.0,
            "<h1 id=\"hello\"><a class=\"gutenberg-anchor\" href=\"#hello\" aria-label=\"Anchor link for: hello\">🔗</a>\nHello*_()</h1>\n"
        );
    }

    #[test]
    fn can_make_toc() {
        let permalinks_ctx = HashMap::new();
        let config_ctx = Config::default();
        let context = Context::new(
            &GUTENBERG_TERA,
            &config_ctx,
            "https://mysite.com/something",
            &permalinks_ctx,
            InsertAnchor::Left
        );

        let res = markdown_to_html(r#"
# Header 1

## Header 2

## Another Header 2

### Last one
        "#, &context).unwrap();

        let toc = res.1;
        assert_eq!(toc.len(), 1);
        assert_eq!(toc[0].children.len(), 2);
        assert_eq!(toc[0].children[1].children.len(), 1);

    }
}
