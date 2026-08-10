//! Minimal HTML → Markdown, used only as a fallback when a fetcher returns raw
//! HTML instead of markdown (Firecrawl normally returns markdown natively via
//! `formats:["markdown"]`, so this rarely runs).
//!
//! ponytail: hand-rolled, tag-table converter. Ceiling = nested tables, `<pre>`
//! whitespace, `<script>`/`<style>` leakage, entities beyond the common set.
//! Upgrade to a real HTML parser (`scraper` + `html2text`) if a migrated page's
//! body looks mangled; for the common marketing-page shape this is enough.

use std::sync::OnceLock;

use regex::Regex;

static TAG_RE: OnceLock<Regex> = OnceLock::new();
fn tag_re() -> &'static Regex {
    TAG_RE.get_or_init(|| Regex::new(r"<[^>]+>").unwrap())
}

/// First `<title>…</title>` text, trimmed; empty when absent.
pub fn extract_title(html: &str) -> String {
    let lower = html.to_ascii_lowercase();
    let Some(open) = lower.find("<title>") else {
        return String::new();
    };
    let start = open + "<title>".len();
    let Some(end) = lower[start..].find("</title>") else {
        return String::new();
    };
    decode(html[start..start + end].trim())
}

/// Convert a slice of HTML body to readable markdown-ish text.
pub fn html_to_markdown(html: &str) -> String {
    // drop script/style blocks wholesale
    let stripped = strip_blocks(html, "script");
    let stripped = strip_blocks(&stripped, "style");

    let mut out = String::with_capacity(stripped.len());
    let mut last = '\n';
    for token in tokenize(&stripped) {
        match token {
            Tok::Text(t) => {
                // collapse whitespace runs (incl. newlines) to single spaces but
                // do NOT trim per-token — inline tags like <strong> rely on the
                // surrounding text keeping its spaces ("Para one <strong>bold").
                let collapsed = collapse_ws(&decode(t));
                if !collapsed.is_empty() {
                    out.push_str(&collapsed);
                    last = collapsed.chars().next().unwrap();
                }
            }
            Tok::Tag(name, _attrs) => {
                let sep = block_separator(&name, last);
                out.push_str(sep);
                last = sep.chars().next().unwrap_or(last);
            }
        }
    }
    // collapse 3+ newlines to a paragraph break, trim trailing space
    let collapse = Regex::new(r"\n{3,}").unwrap();
    let out = collapse.replace_all(&out, "\n\n");
    out.trim().to_string()
}

#[derive(Debug)]
enum Tok<'a> {
    Text(&'a str),
    Tag(String, String),
}

fn tokenize(s: &str) -> Vec<Tok<'_>> {
    let tag = tag_re();
    let mut toks = Vec::new();
    let mut last = 0;
    for m in tag.find_iter(s) {
        if m.start() > last {
            toks.push(Tok::Text(&s[last..m.start()]));
        }
        let raw = m.as_str();
        let inner = raw
            .trim_start_matches('<')
            .trim_end_matches('>')
            .trim_end_matches('/');
        let (name, attrs) = match inner.find(char::is_whitespace) {
            Some(i) => (inner[..i].to_ascii_lowercase(), inner[i..].to_string()),
            None => (inner.to_ascii_lowercase(), String::new()),
        };
        toks.push(Tok::Tag(name, attrs));
        last = m.end();
    }
    if last < s.len() {
        toks.push(Tok::Text(&s[last..]));
    }
    toks
}

/// Prefix/suffix whitespace a start-of-block tag contributes.
fn block_separator(tag: &str, last: char) -> &'static str {
    match tag {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "li" | "ul" | "ol"
        | "div" | "section" | "article" | "header" | "footer" | "br" => {
            if last == '\n' {
                "\n"
            } else {
                "\n\n"
            }
        }
        _ => "",
    }
}

fn strip_blocks(html: &str, name: &str) -> String {
    let open = format!("<{name}");
    let close = format!("</{name}>");
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(s) = rest.to_ascii_lowercase().find(&open) {
        out.push_str(&rest[..s]);
        let after = match rest[s..].to_ascii_lowercase().find(&close) {
            Some(e) => s + e + close.len(),
            None => rest.len(), // unterminated — drop to end
        };
        rest = &rest[after..];
    }
    out.push_str(rest);
    out
}

fn collapse_ws(s: &str) -> String {
    let ws = Regex::new(r"\s+").unwrap();
    ws.replace_all(s, " ").into_owned()
}

fn decode(s: &str) -> String {
    s.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&mdash;", "—")
        .replace("&ndash;", "–")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_extracted() {
        let h = "<html><head><title>  Hello &amp; Bye  </title></head></html>";
        assert_eq!(extract_title(h), "Hello & Bye");
        assert_eq!(extract_title("<html></html>"), "");
    }

    #[test]
    fn basic_conversion() {
        let h = "<h1>Title</h1><p>Para one <strong>bold</strong>.</p><p>Para two.</p>";
        let md = html_to_markdown(h);
        assert!(md.contains("Title"));
        assert!(md.contains("Para one bold."));
        assert!(md.contains("Para two."));
        assert!(!md.contains("<"));
    }

    #[test]
    fn script_and_style_dropped() {
        let h = "<style>.x{color:red}</style><p>keep</p><script>evil()</script>";
        let md = html_to_markdown(h);
        assert!(md.contains("keep"));
        assert!(!md.contains("evil"));
        assert!(!md.contains("color:red"));
    }

    #[test]
    fn empty_input_is_empty() {
        assert_eq!(html_to_markdown(""), "");
        assert_eq!(html_to_markdown("   "), "");
    }
}
