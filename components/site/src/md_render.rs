//! This is here to avoid content depending on the markdown subcrate

use std::borrow::Cow;
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
    permalinks: &AHashMap<String, String>,
    colocated_assets: &AHashMap<String, (String, String)>,
    wikilinks: &AHashMap<String, String>,
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
        wikilinks,
        lang: &page.lang,
        current_permalink: &page.permalink,
        current_path: &page.file.relative,
        insert_anchor,
    };
    let res = markdown::render_content(&input, &context)
        .with_context(|| format!("Failed to render content of {}", page.file.path.display()))?;

    page.summary = res.summary;
    page.content = res.body;
    page.raw_content.clear();
    page.toc = res.toc;
    page.internal_links = res.internal_links;
    page.external_links = res.external_links;
    Ok(())
}

pub fn render_section(
    section: &mut Section,
    renderer: Renderer,
    permalinks: &AHashMap<String, String>,
    colocated_assets: &AHashMap<String, (String, String)>,
    wikilinks: &AHashMap<String, String>,
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
        wikilinks,
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
    section.raw_content.clear();
    section.toc = res.toc;
    section.external_links = res.external_links;
    if let Some(ref redirect_to) = section.meta.redirect_to
        && is_external_link(redirect_to)
    {
        section.external_links.push(redirect_to.to_owned());
    }
    section.internal_links = res.internal_links;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
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
            &AHashMap::default(),
            &AHashMap::default(),
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
            &AHashMap::default(),
            &AHashMap::default(),
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
            &AHashMap::default(),
            &AHashMap::default(),
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
            &AHashMap::default(),
            &AHashMap::default(),
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
}
