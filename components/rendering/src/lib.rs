mod context;
mod markdown;
mod shortcode;
mod table_of_contents;

use errors::Result;
use std::str;

pub use context::RenderContext;
use katex;
use markdown::markdown_to_html;
use pcre2::bytes::{Regex, RegexBuilder};
pub use shortcode::render_shortcodes;
pub use table_of_contents::Heading;

pub fn render_content(content: &str, context: &RenderContext) -> Result<markdown::Rendered> {
    let mut preprocessed: String = content.to_string();

    // Don't do shortcodes if there is nothing like a shortcode in the content
    if preprocessed.contains("{{") || preprocessed.contains("{%") {
        preprocessed = render_shortcodes(&preprocessed, context)?;
    }

    preprocessed = render_katex(preprocessed);

    let mut html = markdown_to_html(&preprocessed, context)?;
    html.body = html.body.replace("<!--\\n-->", "\n");
    return Ok(html);
}

fn render_katex(content: String) -> String {
    let mut re_builder = RegexBuilder::new();
    let re_options = re_builder
        .multi_line(true)
        .extended(true)
        .ucp(true) // unicode
        .jit_if_available(true);
    let inline_math_re = re_options.clone()
        .build(
            r"(?<![\\\$])\$ # non-escaped opening dollar and non-double-dollar
            (
              [^\s\$] # immediately followed by a non-whitespace character
              [^\$]*
              (?<![\\\s\$]) # closing dollar is immediately preceeded by a non-whitespace,
                            # non-backslash character
            )
            \$(?![\d\$]) # closing dollar is not immediately followed by a digit or another dollar"
        ).unwrap();
    let display_math_re = re_options
        .build(
            r"(?<!\\)\$\$ # opening double-dollar not preceeded by a backslash
            (?=[^\s]|\h*\n\h*[^\$\s]) # either no whitespace, or a single newline
                                      # followed by a non-empty line
            ([^\$]*[^\s\$]) # any amount of characters not ending in whitespace
            (?:\h*\n\h*)? # a possibly empty line before closing dollars
            \$\$"
        ).unwrap();


    let inline = render_katex_aux(content, inline_math_re, false);
    render_katex_aux(inline, display_math_re, true)
}

fn render_katex_aux(content: String, rex: Regex, display: bool) -> String {
    let cont_bytes = content.as_bytes();
    let k_opts = katex::Opts::builder().display_mode(display).build().unwrap();
    let mut last: usize = 0;
    let mut with_katex = String::with_capacity(content.len());
    for ma in rex.captures_iter(cont_bytes) {
        if let Ok(mb) = ma {
            let replace = mb.get(0).unwrap();
            let tex = mb.get(1).unwrap();
            with_katex.push_str(str::from_utf8(&cont_bytes[last..replace.start()]).unwrap());
            last = replace.end();
            let s = str::from_utf8(&cont_bytes[tex.start()..tex.end()]).unwrap();
            let k_html = katex::render_with_opts(s, k_opts.clone()).unwrap();
            with_katex.push_str(&k_html);
            // println!("{:?}", k_html);
        }
    }
    with_katex.push_str(str::from_utf8(&cont_bytes[last..]).unwrap());
    with_katex
}
