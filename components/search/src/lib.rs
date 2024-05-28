mod elasticlunr;
mod fuse;

use libs::ammonia;
use libs::once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

pub use elasticlunr::{build_index as build_elasticlunr, ELASTICLUNR_JS};
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

/// uses ammonia to clean the body, and truncates it to `truncate_content_length`
pub fn clean_and_truncate_body(truncate_content_length: Option<usize>, body: &str) -> String {
    let mut clean = AMMONIA.clean(body).to_string();
    if let Some(new_len) = truncate_content_length {
        clean.truncate(clean.char_indices().nth(new_len).map(|(i, _)| i).unwrap_or(clean.len()))
    }
    clean
}

#[cfg(test)]
#[test]
fn clean_and_truncate_body_test() {
    assert_eq!(clean_and_truncate_body(None, "hello world"), "hello world");
    assert_eq!(
        clean_and_truncate_body(None, "hello <script>alert('xss')</script> world"),
        "hello  world"
    );
    assert_eq!(clean_and_truncate_body(Some(100), "hello"), "hello");
    assert_eq!(clean_and_truncate_body(Some(2), "hello"), "he");
    assert_eq!(clean_and_truncate_body(Some(6), "hello \u{202E} world"), "hello ");
    assert_eq!(clean_and_truncate_body(Some(7), "hello \u{202E} world"), "hello \u{202e}");
}
