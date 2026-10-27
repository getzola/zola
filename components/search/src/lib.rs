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

#[inline]
fn push_word_bounded(
    out: &mut String,
    word: &str,
    char_count: &mut usize,
    max_chars: usize,
) -> bool {
    let remaining = max_chars - *char_count;
    if word.is_ascii() {
        if word.len() <= remaining {
            out.push_str(word);
            *char_count += word.len();
        } else {
            out.push_str(&word[..remaining]);
            *char_count += remaining;
        }
    } else {
        let mut word_chars = 0;
        let mut cutoff_byte = None;
        for (idx, _) in word.char_indices() {
            if word_chars == remaining {
                cutoff_byte = Some(idx);
                break;
            }
            word_chars += 1;
        }
        if let Some(cutoff) = cutoff_byte {
            out.push_str(&word[..cutoff]);
            *char_count += remaining;
        } else {
            out.push_str(word);
            *char_count += word_chars;
        }
    }
    *char_count >= max_chars
}

/// Uses ammonia to clean the body, and truncates it to `truncate_content_length`.
/// Removes extra whitespace and blank lines.
pub fn clean_and_truncate_body(truncate_content_length: Option<usize>, body: &str) -> String {
    let cleaned = AMMONIA.clean(body).to_string();

    match truncate_content_length {
        Some(0) => String::new(),
        Some(max_chars) => {
            let mut out = String::with_capacity(cleaned.len().min(max_chars));
            let mut char_count = 0;
            let mut first_line = true;

            'outer: for line in cleaned.lines() {
                let mut words = line.split_whitespace();
                let Some(first_word) = words.next() else { continue };

                if !first_line {
                    out.push('\n');
                    char_count += 1;
                    if char_count >= max_chars {
                        break 'outer;
                    }
                }
                first_line = false;

                if push_word_bounded(&mut out, first_word, &mut char_count, max_chars) {
                    break 'outer;
                }

                for word in words {
                    out.push(' ');
                    char_count += 1;
                    if char_count >= max_chars {
                        break 'outer;
                    }

                    if push_word_bounded(&mut out, word, &mut char_count, max_chars) {
                        break 'outer;
                    }
                }
            }
            out
        }
        None => {
            let mut out = String::with_capacity(cleaned.len());
            let mut first_line = true;

            for line in cleaned.lines() {
                let mut words = line.split_whitespace();
                let Some(first_word) = words.next() else { continue };

                if !first_line {
                    out.push('\n');
                }
                first_line = false;

                out.push_str(first_word);
                for word in words {
                    out.push(' ');
                    out.push_str(word);
                }
            }
            out
        }
    }
}

/// A single page or section that should be included in the search index
/// for a specific language.
#[derive(Clone, Copy)]
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
    let mut items: HashMap<&str, IndexItem> = HashMap::new();
    for (_, section) in &library.sections {
        if section.lang != lang {
            continue;
        }
        if !section.meta.in_search_index {
            continue;
        }

        if section.meta.redirect_to.is_none() && !section.hidden {
            items.insert(
                &section.permalink,
                IndexItem {
                    url: &section.permalink,
                    title: &section.meta.title,
                    datetime: &None,
                    description: &section.meta.description,
                    content: &section.content,
                    path: &section.path,
                },
            );
        }

        for page in &section.pages {
            let page = &library.pages[page];
            if page.meta.in_search_index {
                items.insert(
                    &page.permalink,
                    IndexItem {
                        url: &page.permalink,
                        title: &page.meta.title,
                        datetime: &page.meta.datetime,
                        description: &page.meta.description,
                        content: &page.content,
                        path: &page.path,
                    },
                );
            }
        }
    }
    items.into_values().collect()
}

#[cfg(test)]
#[test]
fn clean_and_truncate_body_test() {
    assert_eq!(clean_and_truncate_body(None, "hello world"), "hello world");
    assert_eq!(
        clean_and_truncate_body(None, "hello <script>alert('xss')</script> world"),
        "hello world"
    );
    assert_eq!(clean_and_truncate_body(Some(0), "hello world"), "");
    assert_eq!(clean_and_truncate_body(Some(100), "hello"), "hello");
    assert_eq!(clean_and_truncate_body(Some(2), "hello"), "he");
    assert_eq!(clean_and_truncate_body(Some(6), "hello \u{202E} world"), "hello ");
    assert_eq!(clean_and_truncate_body(Some(7), "hello \u{202E} world"), "hello \u{202e}");
    assert_eq!(clean_and_truncate_body(None, "hello        world"), "hello world");
    assert_eq!(clean_and_truncate_body(None, "hello    \n    world"), "hello\nworld");
    assert_eq!(clean_and_truncate_body(None, "\n hello  \n \n \n   world\n   "), "hello\nworld");
    assert_eq!(clean_and_truncate_body(Some(5), "hello\nworld"), "hello");
    assert_eq!(clean_and_truncate_body(Some(6), "hello\nworld"), "hello\n");
    assert_eq!(clean_and_truncate_body(Some(7), "hello\nworld"), "hello\nw");
    assert_eq!(clean_and_truncate_body(Some(2), "你好世界"), "你好");
    assert_eq!(clean_and_truncate_body(Some(3), "你好世界"), "你好世");
    assert_eq!(clean_and_truncate_body(Some(5), "你好 世界 测试"), "你好 世界");
    assert_eq!(clean_and_truncate_body(None, ""), "");
    assert_eq!(clean_and_truncate_body(Some(5), ""), "");
    assert_eq!(clean_and_truncate_body(None, "   \n\t\n   "), "");
}
