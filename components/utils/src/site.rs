use percent_encoding::percent_decode;
use std::collections::HashMap;
use std::hash::BuildHasher;
use unicode_segmentation::UnicodeSegmentation;

use errors::{bail, Result};

/// Get word count and estimated reading time
pub fn get_reading_analytics(content: &str) -> (usize, usize) {
    let word_count: usize = content.unicode_words().count();

    // https://help.medium.com/hc/en-us/articles/214991667-Read-time
    // 275 seems a bit too high though
    (word_count, ((word_count + 199) / 200))
}

#[derive(Debug, PartialEq, Clone)]
pub struct ResolvedInternalLink {
    pub permalink: String,
    // The 2 fields below are only set when there is an anchor
    // as we will need that to check if it exists after the markdown rendering is done
    pub md_path: Option<String>,
    pub anchor: Option<String>,
}

/// Resolves an internal link (of the `@/posts/something.md#hey` sort) to its absolute link and
/// returns the path + anchor as well
pub fn resolve_internal_link<S: BuildHasher>(
    link: &str,
    permalinks: &HashMap<String, String, S>,
) -> Result<ResolvedInternalLink> {
    // First we remove the ./ since that's zola specific
    let clean_link = link.replacen("@/", "", 1);
    // Then we remove any potential anchor
    // parts[0] will be the file path and parts[1] the anchor if present
    let parts = clean_link.split('#').collect::<Vec<_>>();
    // If we have slugification turned off, we might end up with some escaped characters so we need
    // to decode them first
    let decoded = &*percent_decode(parts[0].as_bytes()).decode_utf8_lossy();
    match permalinks.get(decoded) {
        Some(p) => {
            if parts.len() > 1 {
                Ok(ResolvedInternalLink {
                    permalink: format!("{}#{}", p, parts[1]),
                    md_path: Some(decoded.to_string()),
                    anchor: Some(parts[1].to_string()),
                })
            } else {
                Ok(ResolvedInternalLink { permalink: p.to_string(), md_path: None, anchor: None })
            }
        }
        None => bail!(format!("Relative link {} not found.", link)),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{get_reading_analytics, resolve_internal_link};

    #[test]
    fn can_resolve_valid_internal_link() {
        let mut permalinks = HashMap::new();
        permalinks.insert("pages/about.md".to_string(), "https://vincent.is/about".to_string());
        let res = resolve_internal_link("@/pages/about.md", &permalinks).unwrap();
        assert_eq!(res.permalink, "https://vincent.is/about");
    }

    #[test]
    fn can_resolve_valid_root_internal_link() {
        let mut permalinks = HashMap::new();
        permalinks.insert("about.md".to_string(), "https://vincent.is/about".to_string());
        let res = resolve_internal_link("@/about.md", &permalinks).unwrap();
        assert_eq!(res.permalink, "https://vincent.is/about");
    }

    #[test]
    fn can_resolve_internal_links_with_anchors() {
        let mut permalinks = HashMap::new();
        permalinks.insert("pages/about.md".to_string(), "https://vincent.is/about".to_string());
        let res = resolve_internal_link("@/pages/about.md#hello", &permalinks).unwrap();
        assert_eq!(res.permalink, "https://vincent.is/about#hello");
        assert_eq!(res.md_path, Some("pages/about.md".to_string()));
        assert_eq!(res.anchor, Some("hello".to_string()));
    }

    #[test]
    fn can_resolve_escaped_internal_links() {
        let mut permalinks = HashMap::new();
        permalinks.insert(
            "pages/about space.md".to_string(),
            "https://vincent.is/about%20space/".to_string(),
        );
        let res = resolve_internal_link("@/pages/about%20space.md#hello", &permalinks).unwrap();
        assert_eq!(res.permalink, "https://vincent.is/about%20space/#hello");
        assert_eq!(res.md_path, Some("pages/about space.md".to_string()));
        assert_eq!(res.anchor, Some("hello".to_string()));
    }

    #[test]
    fn errors_resolve_inexistant_internal_link() {
        let res = resolve_internal_link("@/pages/about.md#hello", &HashMap::new());
        assert!(res.is_err());
    }

    #[test]
    fn reading_analytics_empty_text() {
        let (word_count, reading_time) = get_reading_analytics("  ");
        assert_eq!(word_count, 0);
        assert_eq!(reading_time, 0);
    }

    #[test]
    fn reading_analytics_short_text() {
        let (word_count, reading_time) = get_reading_analytics("Hello World");
        assert_eq!(word_count, 2);
        assert_eq!(reading_time, 1);
    }

    #[test]
    fn reading_analytics_long_text() {
        let mut content = String::new();
        for _ in 0..1000 {
            content.push_str(" Hello world");
        }
        let (word_count, reading_time) = get_reading_analytics(&content);
        assert_eq!(word_count, 2000);
        assert_eq!(reading_time, 10);
    }
}
