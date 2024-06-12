use std::path::{Path, PathBuf};

use libs::unicode_segmentation::UnicodeSegmentation;
use libs::walkdir::WalkDir;

use config::Config;
use utils::fs::is_temp_file;
use utils::table_of_contents::Heading;

pub fn has_anchor(headings: &[Heading], anchor: &str) -> bool {
    for heading in headings {
        if heading.id == anchor {
            return true;
        }
        if has_anchor(&heading.children, anchor) {
            return true;
        }
    }

    false
}

/// Looks into the current folder for the path and see if there's anything that is not a .md
/// file. Those will be copied next to the rendered .html file
/// If `recursive` is set to `true`, it will add all subdirectories assets as well. This should
/// only be set when finding page assets currently.
/// TODO: remove this flag once sections with assets behave the same as pages with assets
/// The returned vector with assets is sorted in case-sensitive order (using `to_ascii_lowercase()`)
pub fn find_related_assets(path: &Path, config: &Config, recursive: bool) -> Vec<PathBuf> {
    let mut assets = vec![];

    let mut builder = WalkDir::new(path).follow_links(true);
    if !recursive {
        builder = builder.max_depth(1);
    }
    for entry in builder.into_iter().filter_map(std::result::Result::ok) {
        let entry_path = entry.path();

        if entry_path.is_file() && !is_temp_file(entry_path) {
            match entry_path.extension() {
                Some(e) => match e.to_str() {
                    Some("md") => continue,
                    _ => assets.push(entry_path.to_path_buf()),
                },
                None => assets.push(entry_path.to_path_buf()),
            }
        }
    }

    if let Some(ref globset) = config.ignored_content_globset {
        assets.retain(|p| !globset.is_match(p));
    }

    assets.sort_by(|a, b| {
        a.to_str().unwrap().to_ascii_lowercase().cmp(&b.to_str().unwrap().to_ascii_lowercase())
    });

    assets
}

/// Get word count and estimated reading time
pub fn get_reading_analytics(content: &str) -> (usize, usize) {
    let word_count: usize = content.unicode_words().count();

    // https://help.medium.com/hc/en-us/articles/214991667-Read-time
    // 275 seems a bit too high though
    (word_count, ((word_count + 199) / 200))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir, File};

    use config::Config;
    use tempfile::tempdir;

    #[test]
    fn can_find_related_assets_recursive() {
        let tmp_dir = tempdir().expect("create temp dir");
        let path = tmp_dir.path();
        File::create(path.join("index.md")).unwrap();
        File::create(path.join("example.js")).unwrap();
        File::create(path.join("graph.jpg")).unwrap();
        File::create(path.join("fail.png")).unwrap();
        File::create(path.join("extensionless")).unwrap();
        create_dir(path.join("subdir")).expect("create subdir temp dir");
        File::create(path.join("subdir").join("index.md")).unwrap();
        File::create(path.join("subdir").join("example.js")).unwrap();
        File::create(path.join("FFF.txt")).unwrap();
        File::create(path.join("GRAPH.txt")).unwrap();
        File::create(path.join("subdir").join("GGG.txt")).unwrap();

        let assets = find_related_assets(path, &Config::default(), true);
        assert_eq!(assets.len(), 7);
        assert_eq!(assets.iter().filter(|p| p.extension().unwrap_or_default() != "md").count(), 7);

        // Use case-insensitive ordering for testassets
        let testassets = [
            "example.js",
            "fail.png",
            "FFF.txt",
            "graph.jpg",
            "GRAPH.txt",
            "subdir/example.js",
            "subdir/GGG.txt",
        ];
        for (asset, testasset) in assets.iter().zip(testassets.iter()) {
            assert!(
                asset.strip_prefix(path).unwrap() == Path::new(testasset),
                "Mismatch between asset {} and testasset {}",
                asset.to_str().unwrap(),
                testasset
            );
        }
    }

    #[test]
    fn can_find_related_assets_non_recursive() {
        let tmp_dir = tempdir().expect("create temp dir");
        let path = tmp_dir.path();
        File::create(path.join("index.md")).unwrap();
        File::create(path.join("example.js")).unwrap();
        File::create(path.join("graph.jpg")).unwrap();
        File::create(path.join("fail.png")).unwrap();
        File::create(path.join("extensionless")).unwrap();
        create_dir(path.join("subdir")).expect("create subdir temp dir");
        File::create(path.join("subdir").join("index.md")).unwrap();
        File::create(path.join("subdir").join("example.js")).unwrap();
        File::create(path.join("FFF.txt")).unwrap();
        File::create(path.join("GRAPH.txt")).unwrap();
        File::create(path.join("subdir").join("GGG.txt")).unwrap();

        let assets = find_related_assets(path, &Config::default(), false);
        assert_eq!(assets.len(), 5);
        assert_eq!(assets.iter().filter(|p| p.extension().unwrap_or_default() != "md").count(), 5);

        // Use case-insensitive ordering for testassets
        let testassets = ["example.js", "fail.png", "FFF.txt", "graph.jpg", "GRAPH.txt"];
        for (asset, testasset) in assets.iter().zip(testassets.iter()) {
            assert!(
                asset.strip_prefix(path).unwrap() == Path::new(testasset),
                "Mismatch between asset {} and testasset {}",
                asset.to_str().unwrap(),
                testasset
            );
        }
    }
    #[test]
    fn can_find_anchor_at_root() {
        let input = vec![
            Heading {
                level: 1,
                id: "1".to_string(),
                permalink: String::new(),
                title: String::new(),
                children: vec![],
            },
            Heading {
                level: 2,
                id: "1-1".to_string(),
                permalink: String::new(),
                title: String::new(),
                children: vec![],
            },
            Heading {
                level: 3,
                id: "1-1-1".to_string(),
                permalink: String::new(),
                title: String::new(),
                children: vec![],
            },
            Heading {
                level: 2,
                id: "1-2".to_string(),
                permalink: String::new(),
                title: String::new(),
                children: vec![],
            },
        ];

        assert!(has_anchor(&input, "1-2"));
    }

    #[test]
    fn can_find_anchor_in_children() {
        let input = vec![Heading {
            level: 1,
            id: "1".to_string(),
            permalink: String::new(),
            title: String::new(),
            children: vec![
                Heading {
                    level: 2,
                    id: "1-1".to_string(),
                    permalink: String::new(),
                    title: String::new(),
                    children: vec![],
                },
                Heading {
                    level: 3,
                    id: "1-1-1".to_string(),
                    permalink: String::new(),
                    title: String::new(),
                    children: vec![],
                },
                Heading {
                    level: 2,
                    id: "1-2".to_string(),
                    permalink: String::new(),
                    title: String::new(),
                    children: vec![],
                },
            ],
        }];

        assert!(has_anchor(&input, "1-2"));
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
