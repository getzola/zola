use std::borrow::Cow::Owned;
use std::collections::HashMap;

use pulldown_cmark as cmark;
use self::cmark::{Parser, Event, Tag, Options, OPTION_ENABLE_TABLES, OPTION_ENABLE_FOOTNOTES};
use regex::Regex;
use slug::slugify;
use syntect::dumps::from_binary;
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use syntect::html::{start_coloured_html_snippet, styles_to_coloured_html, IncludeBackground};
use tera::{Tera, Context};

use config::Config;
use errors::{Result, ResultExt};


// We need to put those in a struct to impl Send and sync
pub struct Setup {
    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
}

unsafe impl Send for Setup {}
unsafe impl Sync for Setup {}

lazy_static!{
    static ref SHORTCODE_RE: Regex = Regex::new(r#"\{(?:%|\{)\s+([[:alnum:]]+?)\(([[:alnum:]]+?="?.+?"?)\)\s+(?:%|\})\}"#).unwrap();
    pub static ref SETUP: Setup = Setup {
        syntax_set: {
            let mut ps: SyntaxSet = from_binary(include_bytes!("../sublime_syntaxes/newlines.packdump"));
            ps.link_syntaxes();
            ps
        },
        theme_set: from_binary(include_bytes!("../sublime_themes/all.themedump"))
    };
}

/// A shortcode that has a body
/// Called by having some content like {% ... %} body {% end %}
/// We need the struct to hold the data while we're processing the markdown
#[derive(Debug)]
struct ShortCode {
    name: String,
    args: HashMap<String, String>,
    body: String,
}

impl ShortCode {
    pub fn new(name: &str, args: HashMap<String, String>) -> ShortCode {
        ShortCode {
            name: name.to_string(),
            args: args,
            body: String::new(),
        }
    }

    pub fn append(&mut self, text: &str) {
        self.body.push_str(text)
    }

    pub fn render(&self, tera: &Tera) -> Result<String> {
        let mut context = Context::new();
        for (key, value) in &self.args {
            context.add(key, value);
        }
        context.add("body", &self.body);
        let tpl_name = format!("shortcodes/{}.html", self.name);
        tera.render(&tpl_name, &context)
            .chain_err(|| format!("Failed to render {} shortcode", self.name))
    }
}

/// Parse a shortcode without a body
fn parse_shortcode(input: &str) -> (String, HashMap<String, String>) {
    let mut args = HashMap::new();
    let caps = SHORTCODE_RE.captures(input).unwrap();
    // caps[0] is the full match
    let name = &caps[1];
    let arg_list = &caps[2];
    for arg in arg_list.split(',') {
        let bits = arg.split('=').collect::<Vec<_>>();
        args.insert(bits[0].trim().to_string(), bits[1].replace("\"", ""));
    }

    (name.to_string(), args)
}

/// Renders a shortcode or return an error
fn render_simple_shortcode(tera: &Tera, name: &str, args: &HashMap<String, String>) -> Result<String> {
    let mut context = Context::new();
    for (key, value) in args.iter() {
        context.add(key, value);
    }
    let tpl_name = format!("shortcodes/{}.html", name);

    tera.render(&tpl_name, &context).chain_err(|| format!("Failed to render {} shortcode", name))
}

pub fn markdown_to_html(content: &str, permalinks: &HashMap<String, String>, tera: &Tera, config: &Config) -> Result<String> {
    // We try to be smart about highlighting code as it can be time-consuming
    // If the global config disables it, then we do nothing. However,
    // if we see a code block in the content, we assume that this page needs
    // to be highlighted. It could potentially have false positive if the content
    // has ``` in it but that seems kind of unlikely
    let should_highlight = if config.highlight_code.unwrap() {
        content.contains("```")
    } else {
        false
    };
    let highlight_theme = config.highlight_theme.clone().unwrap();
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
    // the rendered html
    let mut html = String::new();
    let mut anchors: Vec<String> = vec![];

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
                    match render_simple_shortcode(tera, &name, &args) {
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
                            match shortcode.render(tera) {
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
                    let anchor_link = if config.insert_anchor_links.unwrap() {
                        let mut context = Context::new();
                        context.add("id", &id);
                        tera.render("anchor-link.html", &context).unwrap()
                    } else {
                        String::new()
                    };
                    header_already_inserted = true;
                    return Event::Html(Owned(format!(r#"id="{}">{}{}"#, id, anchor_link, text)));
                }

                // Business as usual
                Event::Text(text)
            },
            Event::Start(Tag::CodeBlock(ref info)) => {
                in_code_block = true;
                if !should_highlight {
                    return Event::Html(Owned("<pre><code>".to_owned()));
                }
                let theme = &SETUP.theme_set.themes[&highlight_theme];
                let syntax = info
                    .split(' ')
                    .next()
                    .and_then(|lang| SETUP.syntax_set.find_syntax_by_token(lang))
                    .unwrap_or_else(|| SETUP.syntax_set.find_syntax_plain_text());
                highlighter = Some(HighlightLines::new(syntax, theme));
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
                if link.starts_with("./") {
                    // First we remove the ./ since that's gutenberg specific
                    let clean_link = link.replacen("./", "", 1);
                    // Then we remove any potential anchor
                    // parts[0] will be the file path and parts[1] the anchor if present
                    let parts = clean_link.split('#').collect::<Vec<_>>();
                    match permalinks.get(parts[0]) {
                        Some(p) => {
                            let url = if parts.len() > 1 {
                                format!("{}#{}", p, parts[1])
                            } else {
                                p.to_string()
                            };
                            return Event::Start(Tag::Link(Owned(url), title.clone()));
                        },
                        None => {
                            error = Some(format!("Relative link {} not found.", link).into());
                            return Event::Html(Owned("".to_string()));
                        }
                    };
                }

                Event::Start(Tag::Link(link.clone(), title.clone()))
            },
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
        None => Ok(html.replace("<p></p>", "")),
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use site::GUTENBERG_TERA;
    use tera::Tera;

    use config::Config;
    use super::{markdown_to_html, parse_shortcode};

    #[test]
    fn test_parse_simple_shortcode_one_arg() {
        let (name, args) = parse_shortcode(r#"{{ youtube(id="w7Ft2ymGmfc") }}"#);
        assert_eq!(name, "youtube");
        assert_eq!(args["id"], "w7Ft2ymGmfc");
    }

    #[test]
    fn test_parse_simple_shortcode_several_arg() {
        let (name, args) = parse_shortcode(r#"{{ youtube(id="w7Ft2ymGmfc", autoplay=true) }}"#);
        assert_eq!(name, "youtube");
        assert_eq!(args["id"], "w7Ft2ymGmfc");
        assert_eq!(args["autoplay"], "true");
    }

    #[test]
    fn test_parse_block_shortcode_several_arg() {
        let (name, args) = parse_shortcode(r#"{% youtube(id="w7Ft2ymGmfc", autoplay=true) %}"#);
        assert_eq!(name, "youtube");
        assert_eq!(args["id"], "w7Ft2ymGmfc");
        assert_eq!(args["autoplay"], "true");
    }

    #[test]
    fn test_markdown_to_html_simple() {
        let res = markdown_to_html("hello", &HashMap::new(), &Tera::default(), &Config::default()).unwrap();
        assert_eq!(res, "<p>hello</p>\n");
    }

    #[test]
    fn test_markdown_to_html_code_block_highlighting_off() {
        let mut config = Config::default();
        config.highlight_code = Some(false);
        let res = markdown_to_html("```\n$ gutenberg server\n```", &HashMap::new(), &Tera::default(), &config).unwrap();
        assert_eq!(
            res,
            "<pre><code>$ gutenberg server\n</code></pre>\n"
        );
    }

    #[test]
    fn test_markdown_to_html_code_block_no_lang() {
        let res = markdown_to_html("```\n$ gutenberg server\n$ ping\n```", &HashMap::new(), &Tera::default(), &Config::default()).unwrap();
        assert_eq!(
            res,
            "<pre style=\"background-color:#2b303b\">\n<span style=\"background-color:#2b303b;color:#c0c5ce;\">$ gutenberg server\n</span><span style=\"background-color:#2b303b;color:#c0c5ce;\">$ ping\n</span></pre>"
        );
    }

    #[test]
    fn test_markdown_to_html_code_block_with_lang() {
        let res = markdown_to_html("```python\nlist.append(1)\n```", &HashMap::new(), &Tera::default(), &Config::default()).unwrap();
        assert_eq!(
            res,
            "<pre style=\"background-color:#2b303b\">\n<span style=\"background-color:#2b303b;color:#c0c5ce;\">list</span><span style=\"background-color:#2b303b;color:#c0c5ce;\">.</span><span style=\"background-color:#2b303b;color:#bf616a;\">append</span><span style=\"background-color:#2b303b;color:#c0c5ce;\">(</span><span style=\"background-color:#2b303b;color:#d08770;\">1</span><span style=\"background-color:#2b303b;color:#c0c5ce;\">)</span><span style=\"background-color:#2b303b;color:#c0c5ce;\">\n</span></pre>"
        );
    }

    #[test]
    fn test_markdown_to_html_code_block_with_unknown_lang() {
        let res = markdown_to_html("```yolo\nlist.append(1)\n```", &HashMap::new(), &Tera::default(), &Config::default()).unwrap();
        // defaults to plain text
        assert_eq!(
            res,
            "<pre style=\"background-color:#2b303b\">\n<span style=\"background-color:#2b303b;color:#c0c5ce;\">list.append(1)\n</span></pre>"
        );
    }

    #[test]
    fn test_markdown_to_html_with_shortcode() {
        let res = markdown_to_html(r#"
Hello

{{ youtube(id="ub36ffWAqgQ") }}
        "#, &HashMap::new(), &GUTENBERG_TERA, &Config::default()).unwrap();
        assert!(res.contains("<p>Hello</p>\n<div >"));
        assert!(res.contains(r#"<iframe src="https://www.youtube.com/embed/ub36ffWAqgQ""#));
    }

    #[test]
    fn test_markdown_to_html_with_several_shortcode_in_row() {
        let res = markdown_to_html(r#"
Hello

{{ youtube(id="ub36ffWAqgQ") }}

{{ youtube(id="ub36ffWAqgQ", autoplay=true) }}

{{ vimeo(id="210073083") }}

{{ gist(url="https://gist.github.com/Keats/32d26f699dcc13ebd41b") }}

        "#, &HashMap::new(), &GUTENBERG_TERA, &Config::default()).unwrap();
        assert!(res.contains("<p>Hello</p>\n<div >"));
        assert!(res.contains(r#"<iframe src="https://www.youtube.com/embed/ub36ffWAqgQ""#));
        assert!(res.contains(r#"<iframe src="https://www.youtube.com/embed/ub36ffWAqgQ?autoplay=1""#));
        assert!(res.contains(r#"//player.vimeo.com/video/210073083""#));
    }

    #[test]
    fn test_markdown_to_html_shortcode_in_code_block() {
        let res = markdown_to_html(r#"```{{ youtube(id="w7Ft2ymGmfc") }}```"#, &HashMap::new(), &GUTENBERG_TERA, &Config::default()).unwrap();
        assert_eq!(res, "<p><code>{{ youtube(id=&quot;w7Ft2ymGmfc&quot;) }}</code></p>\n");
    }

    #[test]
    fn test_markdown_to_html_shortcode_with_body() {
        let mut tera = Tera::default();
        tera.extend(&GUTENBERG_TERA).unwrap();
        tera.add_raw_template("shortcodes/quote.html", "<blockquote>{{ body }} - {{ author}}</blockquote>").unwrap();
        let res = markdown_to_html(r#"
Hello
{% quote(author="Keats") %}
A quote
{% end %}
        "#, &HashMap::new(), &tera, &Config::default()).unwrap();
        assert_eq!(res, "<p>Hello\n</p><blockquote>A quote - Keats</blockquote>");
    }

    #[test]
    fn test_markdown_to_html_unknown_shortcode() {
        let res = markdown_to_html("{{ hello(flash=true) }}", &HashMap::new(), &Tera::default(), &Config::default());
        assert!(res.is_err());
    }

    #[test]
    fn test_markdown_to_html_relative_link_exists() {
        let mut permalinks = HashMap::new();
        permalinks.insert("pages/about.md".to_string(), "https://vincent.is/about".to_string());
        let res = markdown_to_html(
            r#"[rel link](./pages/about.md), [abs link](https://vincent.is/about)"#,
            &permalinks,
            &GUTENBERG_TERA,
            &Config::default()
        ).unwrap();

        assert!(
            res.contains(r#"<p><a href="https://vincent.is/about">rel link</a>, <a href="https://vincent.is/about">abs link</a></p>"#)
        );
    }

    #[test]
    fn test_markdown_to_html_relative_links_with_anchors() {
        let mut permalinks = HashMap::new();
        permalinks.insert("pages/about.md".to_string(), "https://vincent.is/about".to_string());
        let res = markdown_to_html(
            r#"[rel link](./pages/about.md#cv)"#,
            &permalinks,
            &GUTENBERG_TERA,
            &Config::default()
        ).unwrap();

        assert!(
            res.contains(r#"<p><a href="https://vincent.is/about#cv">rel link</a></p>"#)
        );
    }

    #[test]
    fn test_markdown_to_html_relative_link_inexistant() {
        let res = markdown_to_html("[rel link](./pages/about.md)", &HashMap::new(), &Tera::default(), &Config::default());
        assert!(res.is_err());
    }

    #[test]
    fn test_markdown_to_html_add_id_to_headers() {
        let res = markdown_to_html(r#"# Hello"#, &HashMap::new(), &GUTENBERG_TERA, &Config::default()).unwrap();
        assert_eq!(res, "<h1 id=\"hello\">Hello</h1>\n");
    }

    #[test]
    fn test_markdown_to_html_add_id_to_headers_same_slug() {
        let res = markdown_to_html("# Hello\n# Hello", &HashMap::new(), &GUTENBERG_TERA, &Config::default()).unwrap();
        assert_eq!(res, "<h1 id=\"hello\">Hello</h1>\n<h1 id=\"hello-1\">Hello</h1>\n");
    }

    #[test]
    fn test_markdown_to_html_insert_anchor() {
        let mut config = Config::default();
        config.insert_anchor_links = Some(true);
        let res = markdown_to_html("# Hello", &HashMap::new(), &GUTENBERG_TERA, &config).unwrap();
        assert_eq!(
            res,
            "<h1 id=\"hello\"><a class=\"anchor\" href=\"#hello\" aria-label=\"Anchor link for: hello\">🔗</a>\nHello</h1>\n"
        );
    }

    // See https://github.com/Keats/gutenberg/issues/42
    #[test]
    fn test_markdown_to_html_insert_anchor_with_exclamation_mark() {
        let mut config = Config::default();
        config.insert_anchor_links = Some(true);
        let res = markdown_to_html("# Hello!", &HashMap::new(), &GUTENBERG_TERA, &config).unwrap();
        assert_eq!(
            res,
            "<h1 id=\"hello\"><a class=\"anchor\" href=\"#hello\" aria-label=\"Anchor link for: hello\">🔗</a>\nHello!</h1>\n"
        );
    }

    #[test]
    fn test_markdown_to_html_insert_anchor_with_other_special_chars() {
        let mut config = Config::default();
        config.insert_anchor_links = Some(true);
        let res = markdown_to_html("# Hello*_()", &HashMap::new(), &GUTENBERG_TERA, &config).unwrap();
        assert_eq!(
            res,
            "<h1 id=\"hello\"><a class=\"anchor\" href=\"#hello\" aria-label=\"Anchor link for: hello\">🔗</a>\nHello*_()</h1>\n"
        );
    }
}
