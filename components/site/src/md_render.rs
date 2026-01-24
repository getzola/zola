//! This is here to avoid content depending on the markdown subcrate
use std::collections::HashMap;

use tera::Tera;

use config::Config;
use content::{Page, Section};
use errors::{Context, Result};
use markdown::MarkdownContext;
use utils::net::is_external_link;
use utils::types::InsertAnchor;

/// We need access to all pages url to render links relative to content
/// so that can't happen at the same time as parsing
pub fn render_page(
    page: &mut Page,
    permalinks: &HashMap<String, String>,
    tera: &Tera,
    config: &Config,
    insert_anchor: InsertAnchor,
) -> Result<()> {
    let context = MarkdownContext {
        tera,
        config,
        permalinks,
        lang: &page.lang,
        current_permalink: &page.permalink,
        current_path: &page.file.relative,
        insert_anchor,
    };
    let res = markdown::render_content(&page.raw_content, &context)
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
    permalinks: &HashMap<String, String>,
    tera: &Tera,
    config: &Config,
) -> Result<()> {
    let context = MarkdownContext {
        tera,
        config,
        permalinks,
        lang: &section.lang,
        current_permalink: &section.permalink,
        current_path: &section.file.relative,
        insert_anchor: section
            .meta
            .insert_anchor_links
            .unwrap_or(config.markdown.insert_anchor_links),
    };
    let res = markdown::render_content(&section.raw_content, &context)
        .with_context(|| format!("Failed to render content of {}", section.file.path.display()))?;

    section.content = res.body;
    section.raw_content.clear();
    section.toc = res.toc;
    section.external_links = res.external_links;
    if let Some(ref redirect_to) = section.meta.redirect_to {
        if is_external_link(redirect_to) {
            section.external_links.push(redirect_to.to_owned());
        }
    }
    section.internal_links = res.internal_links;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::Path;
    use std::path::PathBuf;

    use config::Config;
    use content::Page;
    use templates::ZOLA_TERA;
    use utils::types::InsertAnchor;

    use super::render_page;

    #[test]
    fn can_specify_summary() {
        let config = Config::default_for_test();
        let content = r#"
+++
+++
Hello world
<!-- more -->"#;
        let res = Page::parse(Path::new("hello.md"), content, &config, &PathBuf::new());
        assert!(res.is_ok());
        let mut page = res.unwrap();
        render_page(&mut page, &HashMap::default(), &ZOLA_TERA, &config, InsertAnchor::None)
            .unwrap();
        assert_eq!(page.summary, Some("<p>Hello world</p>".to_string()));
    }

    #[test]
    fn strips_footnotes_in_summary() {
        let mut config = Config::default_for_test();
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
        render_page(&mut page, &HashMap::default(), &ZOLA_TERA, &config, InsertAnchor::None)
            .unwrap();
        assert_eq!(
            page.summary,
            Some("<p>This page use <sup>1.5</sup> and has footnotes, here\'s one. </p>\n<p>Here's another. </p>".to_string())
        );
        assert_eq!(
            page.content,
            r##"<p>This page use <sup>1.5</sup> and has footnotes, here's one. <sup class="footnote-reference"><a href="#1">1</a></sup></p>
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
"##
        );

        let res = Page::parse(Path::new("hello.md"), content, &config, &PathBuf::new());
        assert!(res.is_ok());
        config.markdown.bottom_footnotes = true;
        let mut page = res.unwrap();
        render_page(&mut page, &HashMap::default(), &ZOLA_TERA, &config, InsertAnchor::None)
            .unwrap();
        assert_eq!(
            page.summary,
            Some("<p>This page use <sup>1.5</sup> and has footnotes, here's one. </p>\n<p>Here's another. </p>".to_string())
        );
        assert_eq!(
            page.content,
            r##"<p>This page use <sup>1.5</sup> and has footnotes, here's one. <sup class="footnote-reference" id="fr-1-1"><a href="#fn-1">1</a></sup></p>
<p>Here's another. <sup class="footnote-reference" id="fr-2-1"><a href="#fn-2">2</a></sup></p>
<span id="continue-reading"></span>
<p>And here's another. <sup class="footnote-reference" id="fr-3-1"><a href="#fn-3">3</a></sup></p>
<section class="footnotes">
<ol class="footnotes-list">
<li id="fn-1">
<p>This is the first footnote. <a href="#fr-1-1">↩</a></p>
</li>
<li id="fn-2">
<p>This is the second footnote. <a href="#fr-2-1">↩</a></p>
</li>
<li id="fn-3">
<p>This is the third footnote. <a href="#fr-3-1">↩</a></p>
</li>
</ol>
</section>
"##
        );
    }
}
