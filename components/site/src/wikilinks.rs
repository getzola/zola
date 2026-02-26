use std::path::Path;

use ahash::AHashMap as HashMap;

/// Build a lookup map from permalinks for wikilink resolution.
///
/// For each entry in `permalinks` (relative_path -> permalink), we insert 2 things pointing to the full relative path.:
/// 1. Full path without extension (eg `docs/overview`)
/// 2. Bare stem (eg `overview`) if different from full path
///
/// If a stem is the same as the full path, the stem is ignored
/// If a stem collides (multiple pages share it, eg _index in Zola), it won't be inserted and users
/// can't refer to that stem in links.
pub fn build_wikilinks(permalinks: &HashMap<String, String>) -> HashMap<String, String> {
    let mut wikilinks = HashMap::new();
    let mut stems: HashMap<String, Vec<&str>> = HashMap::new();

    for relative_path in permalinks.keys() {
        let without_ext = relative_path.trim_end_matches(".md");
        wikilinks.insert(without_ext.to_owned(), relative_path.clone());

        let stem =
            Path::new(without_ext).file_name().unwrap_or_default().to_string_lossy().into_owned();
        if stem != without_ext {
            stems.entry(stem).or_default().push(relative_path);
        }
    }

    for (stem, md_paths) in &stems {
        // Don't overwrite a full-path entry with a bare stem
        if wikilinks.contains_key(stem) {
            continue;
        }
        if md_paths.len() == 1 {
            wikilinks.insert(stem.clone(), md_paths[0].to_owned());
        } else {
            log::warn!("Multiple files with the name `{stem}`, use the full path to link to them");
        }
    }

    wikilinks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_wikilinks_lookups() {
        let permalinks = HashMap::from_iter([
            ("blog/overview.md".to_string(), "/blog/overview/".to_string()),
            ("docs/overview.md".to_string(), "/docs/overview/".to_string()),
            ("about.md".to_string(), "/about/".to_string()),
            ("blog/_index.md".to_string(), "/blog/".to_string()),
            ("_index.md".to_string(), "/".to_string()),
            ("guides/quickstart.md".to_string(), "/guides/quickstart/".to_string()),
        ]);
        let wl = build_wikilinks(&permalinks);

        // Full paths always resolve
        assert_eq!(wl.get("blog/overview"), Some(&"blog/overview.md".to_string()));
        assert_eq!(wl.get("docs/overview"), Some(&"docs/overview.md".to_string()));
        assert_eq!(wl.get("about"), Some(&"about.md".to_string()));
        assert_eq!(wl.get("blog/_index"), Some(&"blog/_index.md".to_string()));
        assert_eq!(wl.get("_index"), Some(&"_index.md".to_string()));
        assert_eq!(wl.get("guides/quickstart"), Some(&"guides/quickstart.md".to_string()));
        assert_eq!(wl.get("quickstart"), Some(&"guides/quickstart.md".to_string()));
        assert_eq!(wl.get("overview"), None);
        // not blog/_index.md, relative path has precedence over stem
        assert_eq!(wl.get("_index"), Some(&"_index.md".to_string()));
        // Relative path and stem being equal should only be inserted once
        assert_eq!(wl.values().filter(|v| *v == "about.md").count(), 1);
    }
}
