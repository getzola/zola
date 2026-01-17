mod elasticlunr;
mod fuse;

use content::Library;
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use time::OffsetDateTime;

pub use elasticlunr::{ELASTICLUNR_JS, build_index as build_elasticlunr};
pub use fuse::build_index as build_fuse;

static AMMONIA: Lazy<ammonia::Builder<'static>> = Lazy::new(|| {
    let mut clean_content = HashSet::new();
    clean_content.insert("script");
    clean_content.insert("style");
    clean_content.insert("pre");
    let mut builder = ammonia::Builder::new();
    builder
        .tags(HashSet::new())
        .tag_attributes(HashMap::new())
        .generic_attributes(HashSet::new())
        .link_rel(None)
        .allowed_classes(HashMap::new())
        .clean_content_tags(clean_content);
    builder
});

/// Uses ammonia to clean the body, and truncates it to `truncate_content_length`.
/// Removes extra whitespace and blank lines.
pub fn clean_and_truncate_body(truncate_content_length: Option<usize>, body: &str) -> String {
    let mut clean = AMMONIA
        .clean(body)
        .to_string()
        .lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    if let Some(new_len) = truncate_content_length {
        clean.truncate(clean.char_indices().nth(new_len).map(|(i, _)| i).unwrap_or(clean.len()))
    }
    clean
}

/// A single page or section that should be included in the search index
/// for a specific language.
struct IndexItem<'a> {
    url: &'a str,
    title: &'a Option<String>,
    description: &'a Option<String>,
    content: &'a String,
    datetime: &'a Option<OffsetDateTime>,
    path: &'a String,
}

/// Collect all pages and sections which should be included in the search index
/// of a given language.
fn collect_index_items<'a>(lang: &str, library: &'a Library) -> Vec<IndexItem<'a>> {
    let mut items: Vec<IndexItem> = Vec::new();
    for (_, section) in &library.sections {
        if section.lang != lang {
            continue;
        }
        if !section.meta.in_search_index {
            continue;
        }

        if section.meta.redirect_to.is_none() {
            items.push(IndexItem {
                url: &section.permalink,
                title: &section.meta.title,
                datetime: &None,
                description: &section.meta.description,
                content: &section.content,
                path: &section.path,
            });
        }

        for page in &section.pages {
            let page = &library.pages[page];
            if page.meta.in_search_index {
                items.push(IndexItem {
                    url: &page.permalink,
                    title: &page.meta.title,
                    datetime: &page.meta.datetime,
                    description: &page.meta.description,
                    content: &page.content,
                    path: &page.path,
                })
            }
        }
    }
    items
}

#[cfg(test)]
#[test]
fn clean_and_truncate_body_test() {
    assert_eq!(clean_and_truncate_body(None, "hello world"), "hello world");
    assert_eq!(
        clean_and_truncate_body(None, "hello <script>alert('xss')</script> world"),
        "hello world"
    );
    assert_eq!(clean_and_truncate_body(Some(100), "hello"), "hello");
    assert_eq!(clean_and_truncate_body(Some(2), "hello"), "he");
    assert_eq!(clean_and_truncate_body(Some(6), "hello \u{202E} world"), "hello ");
    assert_eq!(clean_and_truncate_body(Some(7), "hello \u{202E} world"), "hello \u{202e}");
    assert_eq!(clean_and_truncate_body(None, "hello        world"), "hello world");
    assert_eq!(clean_and_truncate_body(None, "hello    \n    world"), "hello\nworld");
    assert_eq!(clean_and_truncate_body(None, "\n hello  \n \n \n   world\n   "), "hello\nworld");
}
