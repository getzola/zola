use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;

use errors::Result;

/// Get word count and estimated reading time
pub fn get_reading_analytics(content: &str) -> (usize, usize) {
    let word_count: usize = content.unicode_words().count();

    // https://help.medium.com/hc/en-us/articles/214991667-Read-time
    // 275 seems a bit too high though
    (word_count, (word_count / 200))
}

/// Resolves an internal link (of the `./posts/something.md#hey` sort) to its absolute link
pub fn resolve_internal_link(link: &str, permalinks: &HashMap<String, String>) -> Result<String> {
    // First we remove the ./ since that's gutenberg specific
    let clean_link = link.replacen("./", "", 1);
    // Then we remove any potential anchor
    // parts[0] will be the file path and parts[1] the anchor if present
    let parts = clean_link.split('#').collect::<Vec<_>>();
    match permalinks.get(parts[0]) {
        Some(p) => {
            if parts.len() > 1 {
                Ok(format!("{}#{}", p, parts[1]))
            } else {
                Ok(p.to_string())
            }
        },
        None => bail!(format!("Relative link {} not found.", link)),
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{resolve_internal_link, get_reading_analytics};

    #[test]
    fn can_resolve_valid_internal_link() {
        let mut permalinks = HashMap::new();
        permalinks.insert("pages/about.md".to_string(), "https://vincent.is/about".to_string());
        let res = resolve_internal_link("./pages/about.md", &permalinks).unwrap();
        assert_eq!(res, "https://vincent.is/about");
    }

    #[test]
    fn can_resolve_valid_root_internal_link() {
        let mut permalinks = HashMap::new();
        permalinks.insert("about.md".to_string(), "https://vincent.is/about".to_string());
        let res = resolve_internal_link("./about.md", &permalinks).unwrap();
        assert_eq!(res, "https://vincent.is/about");
    }

    #[test]
    fn can_resolve_internal_links_with_anchors() {
        let mut permalinks = HashMap::new();
        permalinks.insert("pages/about.md".to_string(), "https://vincent.is/about".to_string());
        let res = resolve_internal_link("./pages/about.md#hello", &permalinks).unwrap();
        assert_eq!(res, "https://vincent.is/about#hello");
    }

    #[test]
    fn errors_resolve_inexistant_internal_link() {
        let res = resolve_internal_link("./pages/about.md#hello", &HashMap::new());
        assert!(res.is_err());
    }

    #[test]
    fn reading_analytics_short_text() {
        let (word_count, reading_time) = get_reading_analytics("Hello World");
        assert_eq!(word_count, 2);
        assert_eq!(reading_time, 0);
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
