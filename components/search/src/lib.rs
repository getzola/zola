mod elasticlunr;
mod fuse;

use content::Library;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use time::OffsetDateTime;

pub use elasticlunr::{ELASTICLUNR_JS, build_index as build_elasticlunr};
pub use fuse::build_index as build_fuse;

static AMMONIA: LazyLock<ammonia::Builder<'static>> = LazyLock::new(|| {
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
    library
        .pages
        .values()
        .filter(|p| {
            p.lang == lang
                && p.meta.in_search_index
                && p.meta.render
                && !p.hidden
                && p.meta.redirect_to.is_none()
        })
        .map(|p| IndexItem {
            url: &p.permalink,
            title: &p.meta.title,
            datetime: &p.meta.datetime,
            description: &p.meta.description,
            content: &p.content,
            path: &p.path,
        })
        .collect()
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
