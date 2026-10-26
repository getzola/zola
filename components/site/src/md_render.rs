//! This is here to avoid content depending on the markdown subcrate

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::LazyLock;

use ahash::AHashMap;
use regex::Regex;
use tera::Tera;

use config::Config;
use content::{Page, Section};
use errors::{Context as _, Result};
use markdown::MarkdownContext;
use render::Renderer;
use utils::net::is_external_link;
use utils::types::InsertAnchor;

/// We will replace the starting `{` of a heading id with this string which
/// shouldn't be found in real content (hopefully?).
const SENTINEL: &str = "『』@@ZOLA_HEADING_START@@『』";

/// A regex getting headers with a pulldown-cmark id and that are NOT a tera comment
static HEADING_ATTR_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^( {0,3}#{1,6} .*?)\{#([^\s{}#](?:[^{}\n]*[^#{}\n])?)}([^\n]*)$").unwrap()
});

static HEADING_ID_REPLACEMENT: LazyLock<String> =
    LazyLock::new(|| format!("${{1}}{SENTINEL}#${{2}}}}${{3}}"));

/// What we are looking for to check if we need to render via Tera
const TERA_DELIMITERS: [(&[u8], &[u8]); 3] = [(b"{{", b"}}"), (b"{%", b"%}"), (b"{#", b"#}")];

#[inline]
fn protect_heading_ids(content: &str) -> Cow<'_, str> {
    // If we actually see it, well we don't do anything since it would change the actual content
    // but I really want to see that content now.
    if content.contains(SENTINEL) {
        return Cow::Borrowed(content);
    }

    HEADING_ATTR_RE.replace_all(content, HEADING_ID_REPLACEMENT.as_str())
}

#[inline]
fn restore_heading_ids(rendered: String) -> String {
    // fast path: no sentinel → return unchanged
    if rendered.contains(SENTINEL) { rendered.replace(SENTINEL, "{") } else { rendered }
}

#[inline]
fn needs_templating(s: &str) -> bool {
    let bytes = s.as_bytes();
    TERA_DELIMITERS.iter().any(|(open, close)| {
        memchr::memmem::find(bytes, open)
            .is_some_and(|i| memchr::memmem::find(&bytes[i + open.len()..], close).is_some())
    })
}

/// We need access to all pages url to render links relative to content
/// so that can't happen at the same time as parsing
pub fn render_page(
    page: &mut Page,
    renderer: Renderer,
    permalinks: &HashMap<String, String>,
    colocated_assets: &AHashMap<String, (String, String)>,
    tera: &Tera,
    config: &Config,
    insert_anchor: InsertAnchor,
) -> Result<()> {
    let skip_templating = config
        .skip_content_templating_globset
        .as_ref()
        .is_some_and(|gs| gs.is_match(&page.file.relative));

    let input = if !skip_templating {
        let protected = protect_heading_ids(&page.raw_content);
        if !needs_templating(protected.as_ref()) {
            Cow::Borrowed(&page.raw_content)
        } else {
            let rendered = renderer.render_page_content(&protected, page)?;
            Cow::Owned(match protected {
                Cow::Owned(_) => restore_heading_ids(rendered),
                Cow::Borrowed(_) => rendered,
            })
        }
    } else {
        Cow::Borrowed(&page.raw_content)
    };

    let context = MarkdownContext {
        tera,
        config,
        permalinks,
        colocated_assets,
        lang: &page.lang,
        current_permalink: &page.permalink,
        current_path: &page.file.relative,
        insert_anchor,
    };
    let res = markdown::render_content(&input, &context)
        .with_context(|| format!("Failed to render content of {}", page.file.path.display()))?;

    page.summary = res.summary;
    page.content = res.body;
    page.raw_content = String::new();
    page.toc = res.toc;
    page.internal_links = res.internal_links;
    page.external_links = res.external_links;

    if page.lang != config.default_language
        && let Some(stem) = page.file.relative.strip_suffix(&format!(".{}.md", page.lang))
        && let Some(canonical_permalink) = permalinks.get(&format!("{stem}.md"))
    {
        page.content = rewrite_colocated_asset_links(
            &page.content,
            &page.permalink,
            canonical_permalink,
            &page.colocated_assets,
        );
        if let Some(summary) = page.summary.take() {
            page.summary = Some(rewrite_colocated_asset_links(
                &summary,
                &page.permalink,
                canonical_permalink,
                &page.colocated_assets,
            ));
        }
    }

    Ok(())
}

fn rewrite_colocated_asset_links(
    content: &str,
    current_permalink: &str,
    canonical_permalink: &str,
    colocated_assets: &AHashMap<String, (String, String)>,
) -> String {
    if colocated_assets.is_empty() {
        return content.to_string();
    }

    let mut result = String::with_capacity(content.len());
    let mut pos = 0;
    let bytes = content.as_bytes();

    while pos < bytes.len() {
        // Find next '<'
        let next_lt = match content[pos..].find('<') {
            Some(idx) => pos + idx,
            None => {
                result.push_str(&content[pos..]);
                break;
            }
        };

        // Push text preceding '<'
        result.push_str(&content[pos..next_lt]);
        pos = next_lt;

        // Check for HTML comment <!-- ... -->
        if content[pos..].starts_with("<!--") {
            if let Some(end_idx) = content[pos..].find("-->") {
                let comment_end = pos + end_idx + 3;
                result.push_str(&content[pos..comment_end]);
                pos = comment_end;
                continue;
            } else {
                result.push_str(&content[pos..]);
                break;
            }
        }

        // Check for closing tag </...
        if content[pos..].starts_with("</") {
            if let Some(end_idx) = content[pos..].find('>') {
                let tag_end = pos + end_idx + 1;
                result.push_str(&content[pos..tag_end]);
                pos = tag_end;
                continue;
            } else {
                result.push_str(&content[pos..]);
                break;
            }
        }

        // Find the end of this tag '>' taking quotes into account
        let mut in_quote = None;
        let mut tag_end = None;
        for (i, &b) in bytes[pos..].iter().enumerate() {
            match b {
                b'"' | b'\'' => {
                    if in_quote == Some(b) {
                        in_quote = None;
                    } else if in_quote.is_none() {
                        in_quote = Some(b);
                    }
                }
                b'>' if in_quote.is_none() => {
                    tag_end = Some(pos + i + 1);
                    break;
                }
                _ => {}
            }
        }

        let Some(end_pos) = tag_end else {
            result.push_str(&content[pos..]);
            break;
        };

        let tag_slice = &content[pos..end_pos];

        // Parse tag name
        let tag_name = {
            let s = &tag_slice[1..]; // skip '<'
            let end =
                s.find(|c: char| c.is_whitespace() || c == '>' || c == '/').unwrap_or(s.len());
            &s[..end]
        };

        let is_skip_block = tag_name.eq_ignore_ascii_case("pre")
            || tag_name.eq_ignore_ascii_case("code")
            || tag_name.eq_ignore_ascii_case("script")
            || tag_name.eq_ignore_ascii_case("style");

        let is_self_closing = tag_slice.ends_with("/>");

        // Rewrite opening tag
        let rewritten_tag =
            rewrite_tag(tag_slice, current_permalink, canonical_permalink, colocated_assets);
        result.push_str(&rewritten_tag);
        pos = end_pos;

        // If it's a skip block and not self-closing, skip its body untouched
        if is_skip_block && !is_self_closing {
            if let Some(rel_idx) = find_closing_tag(&content[pos..], tag_name) {
                let c_idx = pos + rel_idx;
                result.push_str(&content[pos..c_idx]);
                pos = c_idx;
            } else {
                result.push_str(&content[pos..]);
                break;
            }
        }
    }

    result
}

fn find_closing_tag(haystack: &str, tag_name: &str) -> Option<usize> {
    let tag_bytes = tag_name.as_bytes();
    let prefix = b"</";
    let len = 2 + tag_bytes.len();
    if haystack.len() < len {
        return None;
    }
    for i in 0..=haystack.len() - len {
        if &haystack.as_bytes()[i..i + 2] == prefix {
            let candidate = &haystack.as_bytes()[i + 2..i + len];
            if candidate.eq_ignore_ascii_case(tag_bytes) {
                let remainder = &haystack.as_bytes()[i + len..];
                if remainder.is_empty() {
                    return Some(i);
                }
                let next = remainder[0];
                if next == b'>' || next.is_ascii_whitespace() || next == b'/' {
                    return Some(i);
                }
            }
        }
    }
    None
}

fn rewrite_tag(
    tag_str: &str,
    current_permalink: &str,
    canonical_permalink: &str,
    colocated_assets: &AHashMap<String, (String, String)>,
) -> String {
    if !tag_str.starts_with('<')
        || tag_str.starts_with("</")
        || tag_str.starts_with("<!")
        || tag_str.starts_with("<?")
    {
        return tag_str.to_string();
    }

    let bytes = tag_str.as_bytes();
    let mut i = 1;
    // Skip tag name
    while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' && bytes[i] != b'/'
    {
        i += 1;
    }

    let mut result = String::with_capacity(tag_str.len());
    result.push_str(&tag_str[..i]);

    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() || bytes[i] == b'/' || bytes[i] == b'>' {
            result.push(bytes[i] as char);
            i += 1;
            continue;
        }

        let attr_name_start = i;
        while i < bytes.len()
            && !bytes[i].is_ascii_whitespace()
            && bytes[i] != b'='
            && bytes[i] != b'>'
            && bytes[i] != b'/'
        {
            i += 1;
        }
        let attr_name = &tag_str[attr_name_start..i];
        result.push_str(attr_name);

        let ws_start = i;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }

        if i < bytes.len() && bytes[i] == b'=' {
            result.push_str(&tag_str[ws_start..=i]);
            i += 1;

            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                result.push(bytes[i] as char);
                i += 1;
            }

            if i >= bytes.len() {
                break;
            }

            let quote = bytes[i];
            if quote == b'"' || quote == b'\'' {
                result.push(quote as char);
                i += 1;
                let val_start = i;
                while i < bytes.len() && bytes[i] != quote {
                    i += 1;
                }
                let val = &tag_str[val_start..i];
                let rewritten = if attr_name.eq_ignore_ascii_case("srcset") {
                    rewrite_srcset(val, current_permalink, canonical_permalink, colocated_assets)
                } else if matches_attr_name(attr_name) {
                    rewrite_url(val, current_permalink, canonical_permalink, colocated_assets)
                } else {
                    None
                };

                if let Some(new_val) = rewritten {
                    result.push_str(&new_val);
                } else {
                    result.push_str(val);
                }

                if i < bytes.len() && bytes[i] == quote {
                    result.push(quote as char);
                    i += 1;
                }
            } else {
                let val_start = i;
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' {
                    i += 1;
                }
                let val = &tag_str[val_start..i];
                let rewritten = if attr_name.eq_ignore_ascii_case("srcset") {
                    rewrite_srcset(val, current_permalink, canonical_permalink, colocated_assets)
                } else if matches_attr_name(attr_name) {
                    rewrite_url(val, current_permalink, canonical_permalink, colocated_assets)
                } else {
                    None
                };

                if let Some(new_val) = rewritten {
                    result.push_str(&new_val);
                } else {
                    result.push_str(val);
                }
            }
        }
    }

    result
}

fn matches_attr_name(name: &str) -> bool {
    name.eq_ignore_ascii_case("src")
        || name.eq_ignore_ascii_case("href")
        || name.eq_ignore_ascii_case("poster")
        || name.eq_ignore_ascii_case("data")
        || name.eq_ignore_ascii_case("data-src")
        || name.eq_ignore_ascii_case("data-href")
}

fn rewrite_url(
    url: &str,
    current_permalink: &str,
    canonical_permalink: &str,
    colocated_assets: &AHashMap<String, (String, String)>,
) -> Option<String> {
    let (path_part, query_and_hash) = match url.find(['?', '#']) {
        Some(idx) => (&url[..idx], &url[idx..]),
        None => (url, ""),
    };

    let sub_path = if let Some(stripped) = path_part.strip_prefix(current_permalink) {
        stripped
    } else if path_part.starts_with("http://")
        || path_part.starts_with("https://")
        || path_part.starts_with("//")
        || path_part.to_ascii_lowercase().starts_with("data:")
        || path_part.to_ascii_lowercase().starts_with("mailto:")
        || path_part.to_ascii_lowercase().starts_with("javascript:")
    {
        return None;
    } else if let Some(stripped) = path_part.strip_prefix("./") {
        stripped
    } else if path_part.starts_with('/') {
        let parsed_current = url::Url::parse(current_permalink).ok()?;
        let current_path = parsed_current.path();
        path_part.strip_prefix(current_path)?
    } else {
        path_part
    };

    let matches = colocated_assets.values().any(|(_owner_md, rel)| {
        if sub_path == rel {
            return true;
        }
        let encoded_rel = rel.replace(' ', "%20");
        if sub_path == encoded_rel {
            return true;
        }
        false
    });

    if matches { Some(format!("{canonical_permalink}{sub_path}{query_and_hash}")) } else { None }
}

struct SrcsetCandidate<'a> {
    raw: &'a str,
    url: &'a str,
    descriptor: Option<&'a str>,
}

fn parse_srcset_candidates(srcset: &str) -> Vec<SrcsetCandidate<'_>> {
    let mut candidates = Vec::new();
    let bytes = srcset.as_bytes();
    let mut pos = 0;

    while pos < bytes.len() {
        // Skip leading ASCII whitespace and commas
        while pos < bytes.len() && (bytes[pos].is_ascii_whitespace() || bytes[pos] == b',') {
            pos += 1;
        }
        if pos >= bytes.len() {
            break;
        }

        let cand_start = pos;

        // Collect non-ASCII-whitespace characters for the candidate URL
        let url_token_start = pos;
        while pos < bytes.len() && !bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        let url_token = &srcset[url_token_start..pos];

        // If the URL token ends with a comma, that comma separates this candidate from the next
        if url_token.ends_with(',') {
            let url = url_token.trim_end_matches(',');
            candidates.push(SrcsetCandidate {
                raw: &srcset[cand_start..cand_start + url.len()],
                url,
                descriptor: None,
            });
            continue;
        }

        let url = url_token;

        // Skip whitespace between URL and optional descriptor
        while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }

        if pos >= bytes.len() || bytes[pos] == b',' {
            candidates.push(SrcsetCandidate {
                raw: &srcset[cand_start..url_token_start + url.len()],
                url,
                descriptor: None,
            });
            if pos < bytes.len() && bytes[pos] == b',' {
                pos += 1;
            }
            continue;
        }

        // Collect descriptor until comma outside parentheses or end of string
        let desc_start = pos;
        let mut paren_depth = 0;
        while pos < bytes.len() {
            if bytes[pos] == b'(' {
                paren_depth += 1;
            } else if bytes[pos] == b')' && paren_depth > 0 {
                paren_depth -= 1;
            } else if bytes[pos] == b',' && paren_depth == 0 {
                break;
            }
            pos += 1;
        }

        let desc_end = pos;
        let descriptor = srcset[desc_start..desc_end].trim();
        let raw = &srcset[cand_start..desc_end];

        candidates.push(SrcsetCandidate {
            raw,
            url,
            descriptor: if descriptor.is_empty() { None } else { Some(descriptor) },
        });

        if pos < bytes.len() && bytes[pos] == b',' {
            pos += 1;
        }
    }

    candidates
}

fn rewrite_srcset(
    srcset: &str,
    current_permalink: &str,
    canonical_permalink: &str,
    colocated_assets: &AHashMap<String, (String, String)>,
) -> Option<String> {
    let candidates = parse_srcset_candidates(srcset);
    if candidates.is_empty() {
        return None;
    }

    let mut rewritten = false;
    let mut parts = Vec::with_capacity(candidates.len());

    for cand in candidates {
        if let Some(new_url) =
            rewrite_url(cand.url, current_permalink, canonical_permalink, colocated_assets)
        {
            rewritten = true;
            if let Some(desc) = cand.descriptor {
                parts.push(format!("{new_url} {desc}"));
            } else {
                parts.push(new_url);
            }
        } else {
            parts.push(cand.raw.to_string());
        }
    }

    if rewritten { Some(parts.join(", ")) } else { None }
}

pub fn render_section(
    section: &mut Section,
    renderer: Renderer,
    permalinks: &HashMap<String, String>,
    colocated_assets: &AHashMap<String, (String, String)>,
    tera: &Tera,
    config: &Config,
) -> Result<()> {
    let skip_templating = config
        .skip_content_templating_globset
        .as_ref()
        .is_some_and(|gs| gs.is_match(&section.file.relative));

    let input = if !skip_templating {
        let protected = protect_heading_ids(&section.raw_content);
        if !needs_templating(protected.as_ref()) {
            Cow::Borrowed(&section.raw_content)
        } else {
            let rendered = renderer.render_section_content(&protected, section)?;
            Cow::Owned(match protected {
                Cow::Owned(_) => restore_heading_ids(rendered),
                Cow::Borrowed(_) => rendered,
            })
        }
    } else {
        Cow::Borrowed(&section.raw_content)
    };

    let context = MarkdownContext {
        tera,
        config,
        permalinks,
        colocated_assets,
        lang: &section.lang,
        current_permalink: &section.permalink,
        current_path: &section.file.relative,
        insert_anchor: section
            .meta
            .insert_anchor_links
            .unwrap_or(config.markdown.insert_anchor_links),
    };
    let res = markdown::render_content(&input, &context)
        .with_context(|| format!("Failed to render content of {}", section.file.path.display()))?;

    section.content = res.body;
    section.raw_content = String::new();
    section.toc = res.toc;
    section.external_links = res.external_links;
    if let Some(ref redirect_to) = section.meta.redirect_to
        && is_external_link(redirect_to)
    {
        section.external_links.push(redirect_to.to_owned());
    }
    section.internal_links = res.internal_links;

    if section.lang != config.default_language
        && let Some(stem) = section.file.relative.strip_suffix(&format!(".{}.md", section.lang))
        && let Some(canonical_permalink) = permalinks.get(&format!("{stem}.md"))
    {
        section.content = rewrite_colocated_asset_links(
            &section.content,
            &section.permalink,
            canonical_permalink,
            &section.colocated_assets,
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::collections::HashMap;
    use std::path::Path;
    use std::path::PathBuf;

    use ahash::AHashMap;

    use config::Config;
    use content::{Library, Page};
    use render::{RenderCache, Renderer};
    use templates::ZOLA_TERA;
    use utils::types::InsertAnchor;

    use super::{protect_heading_ids, render_page, restore_heading_ids};

    fn make_renderer<'a>(
        config: &'a Config,
        library: &'a Library,
        cache: &'a RenderCache,
    ) -> Renderer<'a> {
        Renderer::new(&ZOLA_TERA, config, library, cache)
    }

    #[test]
    fn can_specify_summary() {
        let config = Config::default_for_test();
        let library = Library::default();
        let mut cache = RenderCache::new(&config);
        cache.build(&library, &[], &ZOLA_TERA);
        let content = r#"
+++
+++
Hello world
<!-- more -->"#;
        let res = Page::parse(Path::new("hello.md"), content, &config, &PathBuf::new());
        assert!(res.is_ok());
        let mut page = res.unwrap();
        let renderer = make_renderer(&config, &library, &cache);
        render_page(
            &mut page,
            renderer,
            &HashMap::default(),
            &AHashMap::default(),
            &ZOLA_TERA,
            &config,
            InsertAnchor::None,
        )
        .unwrap();
        assert_eq!(page.summary, Some("<p>Hello world</p>".to_string()));
    }

    #[test]
    fn strips_footnotes_in_summary() {
        let mut config = Config::default_for_test();
        let library = Library::default();
        let mut cache = RenderCache::new(&config);
        cache.build(&library, &[], &ZOLA_TERA);
        let content = r#"
+++
+++
This page use <sup>1.5</sup> and has footnotes, here's one. [^1]

Here's another. [^2]

<!-- more -->

And here's another. [^3]

[^1]: This is the first footnote.

[^2]: This is the second footnote.

[^3]: This is the third footnote."#;
        let res = Page::parse(Path::new("hello.md"), content, &config, &PathBuf::new());
        assert!(res.is_ok());
        let mut page = res.unwrap();
        let renderer = make_renderer(&config, &library, &cache);
        render_page(
            &mut page,
            renderer,
            &HashMap::default(),
            &AHashMap::default(),
            &ZOLA_TERA,
            &config,
            InsertAnchor::None,
        )
        .unwrap();
        insta::assert_snapshot!(page.summary.as_deref().unwrap_or(""), @r###"
        <p>This page use <sup>1.5</sup> and has footnotes, here's one. </p>
        <p>Here's another. </p>
        "###);
        insta::assert_snapshot!(page.content, @r###"
        <p>This page use <sup>1.5</sup> and has footnotes, here's one. <sup class="footnote-reference"><a href="#1">1</a></sup></p>
        <p>Here's another. <sup class="footnote-reference"><a href="#2">2</a></sup></p>
        <span id="continue-reading"></span>
        <p>And here's another. <sup class="footnote-reference"><a href="#3">3</a></sup></p>
        <div class="footnote-definition" id="1"><sup class="footnote-definition-label">1</sup>
        <p>This is the first footnote.</p>
        </div>
        <div class="footnote-definition" id="2"><sup class="footnote-definition-label">2</sup>
        <p>This is the second footnote.</p>
        </div>
        <div class="footnote-definition" id="3"><sup class="footnote-definition-label">3</sup>
        <p>This is the third footnote.</p>
        </div>
        "###);

        let res = Page::parse(Path::new("hello.md"), content, &config, &PathBuf::new());
        assert!(res.is_ok());
        config.markdown.bottom_footnotes = true;
        let mut cache = RenderCache::new(&config);
        cache.build(&library, &[], &ZOLA_TERA);
        let mut page = res.unwrap();
        let renderer = make_renderer(&config, &library, &cache);
        render_page(
            &mut page,
            renderer,
            &HashMap::default(),
            &AHashMap::default(),
            &ZOLA_TERA,
            &config,
            InsertAnchor::None,
        )
        .unwrap();
        insta::assert_snapshot!(page.summary.as_deref().unwrap_or(""), @r###"
        <p>This page use <sup>1.5</sup> and has footnotes, here's one. </p>
        <p>Here's another. </p>
        "###);
        insta::assert_snapshot!(page.content, @r#"
        <p>This page use <sup>1.5</sup> and has footnotes, here's one. <sup class="footnote-reference" id="fr-1-1"><a href="http://a-website.com/hello/#fn-1">[1]</a></sup></p>
        <p>Here's another. <sup class="footnote-reference" id="fr-2-1"><a href="http://a-website.com/hello/#fn-2">[2]</a></sup></p>
        <span id="continue-reading"></span>
        <p>And here's another. <sup class="footnote-reference" id="fr-3-1"><a href="http://a-website.com/hello/#fn-3">[3]</a></sup></p>
        <section class="footnotes">
        <ol class="footnotes-list">
        <li id="fn-1">
        <p>This is the first footnote. <a href="http://a-website.com/hello/#fr-1-1">↩</a></p>
        </li>
        <li id="fn-2">
        <p>This is the second footnote. <a href="http://a-website.com/hello/#fr-2-1">↩</a></p>
        </li>
        <li id="fn-3">
        <p>This is the third footnote. <a href="http://a-website.com/hello/#fr-3-1">↩</a></p>
        </li>
        </ol>
        </section>
        "#);
    }

    #[test]
    fn can_protect_valid_heading_ids() {
        let inputs = vec![
            ("## Mermaid {#mermaid-header}", true),
            ("## Mermaid {#mermaid-header #another}", true),
            ("## Mermaid { #mermaid-header }", false),
            ("## Mermaid {#mermaid-header .some-class}", true),
            ("## Mermaid {#mermaid-header} {#some comments#}", true),
            ("## Mermaid {#some comments#} {#mermaid-header}", true),
            ("## Mermaid {#some comments#}", false),
            ("## Mermaid {# some comments#}", false),
        ];

        for (input, should_be_owned) in inputs {
            println!("{input}");
            let res = protect_heading_ids(input);
            if should_be_owned {
                assert!(matches!(res, Cow::Owned(_)));
            } else {
                assert!(matches!(res, Cow::Borrowed(_)));
            }
            assert_eq!(input, restore_heading_ids(res.to_string()));
        }
    }

    // https://github.com/getzola/zola/issues/3234
    #[test]
    fn can_keep_heading_ids() {
        let config = Config::default_for_test();
        let library = Library::default();
        let mut cache = RenderCache::new(&config);
        cache.build(&library, &[], &ZOLA_TERA);
        let content = r##"
+++
+++
# Mermaid {#mermaid-header}
# Mermaid { #mermaid-header2}
# Mermaid { #mermaid-header3 .class}
# Mermaid {#comments#}
# Mermaid {#comments#} {#mermaid-header4}
# Mermaid {#mermaid-header5}{#comments#}

{{ 0 }}
"##;
        let res = Page::parse(Path::new("hello.md"), content, &config, &PathBuf::new());
        assert!(res.is_ok());
        let mut page = res.unwrap();
        let renderer = make_renderer(&config, &library, &cache);
        render_page(
            &mut page,
            renderer,
            &HashMap::default(),
            &AHashMap::default(),
            &ZOLA_TERA,
            &config,
            InsertAnchor::None,
        )
        .unwrap();
        insta::assert_snapshot!(page.content, @r#"
        <h1 id="mermaid-header">Mermaid</h1>
        <h1 id="mermaid-header2">Mermaid</h1>
        <h1 id="mermaid-header3" class="class">Mermaid</h1>
        <h1 id="mermaid">Mermaid</h1>
        <h1 id="mermaid-header4">Mermaid</h1>
        <h1 id="mermaid-header5">Mermaid</h1>
        <p>0</p>
        "#);
    }

    #[test]
    fn test_rewrite_colocated_asset_links() {
        let mut colocated_assets = AHashMap::new();
        colocated_assets.insert(
            "blog/with-assets/some.js".to_string(),
            ("blog/with-assets/index.md".to_string(), "some.js".to_string()),
        );
        colocated_assets.insert(
            "blog/with-assets/img 2.png".to_string(),
            ("blog/with-assets/index.md".to_string(), "img 2.png".to_string()),
        );

        let current_permalink = "https://example.com/fr/blog/with-assets/";
        let canonical_permalink = "https://example.com/blog/with-assets/";

        // 1. Raw HTML with query string and hash
        let input = r#"<img src="some.js?v=1#sec" alt="test">"#;
        let out = super::rewrite_colocated_asset_links(
            input,
            current_permalink,
            canonical_permalink,
            &colocated_assets,
        );
        assert_eq!(
            out,
            r#"<img src="https://example.com/blog/with-assets/some.js?v=1#sec" alt="test">"#
        );

        // 2. Relative ./ prefix and single quotes
        let input = r#"<a href='./some.js?v=2'>link</a>"#;
        let out = super::rewrite_colocated_asset_links(
            input,
            current_permalink,
            canonical_permalink,
            &colocated_assets,
        );
        assert_eq!(out, r#"<a href='https://example.com/blog/with-assets/some.js?v=2'>link</a>"#);

        // 3. Current permalink prefix replaced with canonical
        let input = r#"<img src="https://example.com/fr/blog/with-assets/some.js?v=1">"#;
        let out = super::rewrite_colocated_asset_links(
            input,
            current_permalink,
            canonical_permalink,
            &colocated_assets,
        );
        assert_eq!(out, r#"<img src="https://example.com/blog/with-assets/some.js?v=1">"#);

        // 4. Srcset rewriting
        let input = r#"<source srcset="some.js?v=1 1x, ./some.js?v=2 2x, other.js 3x">"#;
        let out = super::rewrite_colocated_asset_links(
            input,
            current_permalink,
            canonical_permalink,
            &colocated_assets,
        );
        assert_eq!(
            out,
            r#"<source srcset="https://example.com/blog/with-assets/some.js?v=1 1x, https://example.com/blog/with-assets/some.js?v=2 2x, other.js 3x">"#
        );

        // 5. Preserved untouched in <pre>
        let input = r#"<pre>https://example.com/fr/blog/with-assets/some.js</pre>"#;
        let out = super::rewrite_colocated_asset_links(
            input,
            current_permalink,
            canonical_permalink,
            &colocated_assets,
        );
        assert_eq!(out, input);

        // 6. Preserved untouched in <code>
        let input = r#"<code>some.js</code>"#;
        let out = super::rewrite_colocated_asset_links(
            input,
            current_permalink,
            canonical_permalink,
            &colocated_assets,
        );
        assert_eq!(out, input);

        // 7. Preserved untouched in <pre><code>...</code></pre>
        let input = r#"<pre><code class="language-html">&lt;img src="some.js"&gt; https://example.com/fr/blog/with-assets/some.js</code></pre>"#;
        let out = super::rewrite_colocated_asset_links(
            input,
            current_permalink,
            canonical_permalink,
            &colocated_assets,
        );
        assert_eq!(out, input);

        // 8. Preserved in inline <script> and <style>
        let input =
            r#"<script>var x = "https://example.com/fr/blog/with-assets/some.js";</script>"#;
        let out = super::rewrite_colocated_asset_links(
            input,
            current_permalink,
            canonical_permalink,
            &colocated_assets,
        );
        assert_eq!(out, input);

        // 9. External script tag src is rewritten, body untouched
        let input = r#"<script src="some.js?v=1"></script>"#;
        let out = super::rewrite_colocated_asset_links(
            input,
            current_permalink,
            canonical_permalink,
            &colocated_assets,
        );
        assert_eq!(
            out,
            r#"<script src="https://example.com/blog/with-assets/some.js?v=1"></script>"#
        );

        // 10. Tag containing <code> inside: tag attribute is rewritten, code content is untouched
        let input = r#"<a href="some.js"><code>some.js</code></a>"#;
        let out = super::rewrite_colocated_asset_links(
            input,
            current_permalink,
            canonical_permalink,
            &colocated_assets,
        );
        assert_eq!(
            out,
            r#"<a href="https://example.com/blog/with-assets/some.js"><code>some.js</code></a>"#
        );

        // 11. Unquoted srcset attribute
        let input = r#"<img srcset=some.js alt="unquoted">"#;
        let out = super::rewrite_colocated_asset_links(
            input,
            current_permalink,
            canonical_permalink,
            &colocated_assets,
        );
        assert_eq!(
            out,
            r#"<img srcset=https://example.com/blog/with-assets/some.js alt="unquoted">"#
        );

        // 12. Unquoted srcset with ./ prefix
        let input = r#"<img srcset=./some.js alt="unquoted dot">"#;
        let out = super::rewrite_colocated_asset_links(
            input,
            current_permalink,
            canonical_permalink,
            &colocated_assets,
        );
        assert_eq!(
            out,
            r#"<img srcset=https://example.com/blog/with-assets/some.js alt="unquoted dot">"#
        );

        // 13. Data URL in srcset is preserved intact without extra spaces
        let input = r#"<img srcset="data:image/png;base64,AAAA 1x, some.js 2x">"#;
        let out = super::rewrite_colocated_asset_links(
            input,
            current_permalink,
            canonical_permalink,
            &colocated_assets,
        );
        assert_eq!(
            out,
            r#"<img srcset="data:image/png;base64,AAAA 1x, https://example.com/blog/with-assets/some.js 2x">"#
        );

        // 14. Data URL without descriptor followed by comma
        let input = r#"<img srcset="data:image/png;base64,AAAA, some.js 2x">"#;
        let out = super::rewrite_colocated_asset_links(
            input,
            current_permalink,
            canonical_permalink,
            &colocated_assets,
        );
        assert_eq!(
            out,
            r#"<img srcset="data:image/png;base64,AAAA, https://example.com/blog/with-assets/some.js 2x">"#
        );

        // 15. Srcset without rewritable assets is preserved completely
        let input =
            r#"<img srcset="data:image/png;base64,AAAA 1x, https://cdn.example.com/a.png 2x">"#;
        let out = super::rewrite_colocated_asset_links(
            input,
            current_permalink,
            canonical_permalink,
            &colocated_assets,
        );
        assert_eq!(out, input);
    }
}
