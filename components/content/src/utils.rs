use std::collections::HashMap;
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

/// Serializes assets source path for assets colocated with a section or a page
pub fn serialize_assets(
    assets: &Vec<PathBuf>,
    parent_path: Option<&Path>,
    colocated_path: Option<&String>,
) -> Vec<String> {
    assets
        .iter()
        .filter_map(|asset| asset.strip_prefix(parent_path.unwrap()).ok())
        .map(|asset_relative_path| {
            asset_relative_path
                .components()
                .map(|component| component.as_os_str().to_string_lossy().to_string())
                .collect::<Vec<String>>()
                .join("/")
        })
        .map(|asset_relative_path_as_string| {
            format!(
                "/{}{}",
                colocated_path.expect("Should have colocated path for assets"),
                asset_relative_path_as_string
            )
        })
        .collect()
}

/// Create assets permalinks based on the permalin of the section or the page they are colocated with
pub fn get_assets_permalinks(
    serialized_assets: &Vec<String>,
    parent_permalink: &str,
    colocated_path: Option<&String>,
) -> HashMap<String, String> {
    let mut permalinks = HashMap::new();
    if !serialized_assets.is_empty() {
        let colocated_path = colocated_path.expect("Should have a colocated path for assets");
        for asset in serialized_assets {
            let asset_file_path = asset.strip_prefix("/").unwrap_or(asset);
            let page_relative_asset_path = asset_file_path
                .strip_prefix(colocated_path)
                .expect("Should be able to stripe colocated path from asset path");
            let asset_permalink = format!("{}{}", parent_permalink, page_relative_asset_path);
            permalinks.insert(asset_file_path.to_string(), asset_permalink.to_string());
        }
    }
    permalinks
}

/// Get word count and estimated reading time
pub fn get_reading_analytics(content: &str) -> (usize, usize) {
    // code fences "toggle" the state from non-code to code and back, so anything inbetween the
    // first fence and the next can be ignored
    let split = content.split("```");
    let word_count = split.step_by(2).map(|section| section.unicode_words().count()).sum();

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
    fn can_serialize_assets() {
        let parent_path = Path::new("/tmp/test");
        let page_folder_path = parent_path.join("content").join("posts").join("my-article");
        let assets = vec![
            page_folder_path.join("example.js"),
            page_folder_path.join("graph.jpg"),
            page_folder_path.join("fail.png"),
            page_folder_path.join("extensionless"),
            page_folder_path.join("subdir").join("example.js"),
            page_folder_path.join("FFF.txt"),
            page_folder_path.join("GRAPH.txt"),
            page_folder_path.join("subdir").join("GGG.txt"),
        ];
        let colocated_path = "posts/my-article/".to_string();
        let expected_serialized_assets = vec![
            "/posts/my-article/example.js",
            "/posts/my-article/graph.jpg",
            "/posts/my-article/fail.png",
            "/posts/my-article/extensionless",
            "/posts/my-article/subdir/example.js",
            "/posts/my-article/FFF.txt",
            "/posts/my-article/GRAPH.txt",
            "/posts/my-article/subdir/GGG.txt",
        ];

        let serialized_assets =
            serialize_assets(&assets, Some(&page_folder_path), Some(&colocated_path));

        assert_eq!(
            serialized_assets, expected_serialized_assets,
            "Serialized assets (left) are different from expected (right)",
        );
    }

    #[test]
    fn can_serialize_empty_assets() {
        let parent_path = Path::new("/tmp/test");
        let page_folder_path = parent_path.join("content").join("posts").join("my-article");
        let assets: Vec<PathBuf> = vec![];

        let serialized_assets = serialize_assets(&assets, Some(&page_folder_path), None);

        assert!(serialized_assets.is_empty());
    }

    #[test]
    fn can_get_assets_permalinks() {
        let serialized_assets = vec![
            "/posts/my-article/example.js".to_string(),
            "/posts/my-article/graph.jpg".to_string(),
            "/posts/my-article/fail.png".to_string(),
            "/posts/my-article/extensionless".to_string(),
            "/posts/my-article/subdir/example.js".to_string(),
            "/posts/my-article/FFF.txt".to_string(),
            "/posts/my-article/GRAPH.txt".to_string(),
            "/posts/my-article/subdir/GGG.txt".to_string(),
        ];
        let parent_permalink = "https://remplace-par-ton-url.fr/posts/my-super-article/";
        let colocated_path = "posts/my-article/".to_string();
        let mut expected_assets_permalinks = HashMap::<String, String>::new();
        expected_assets_permalinks.insert(
            "posts/my-article/example.js".to_string(),
            format!("{}{}", parent_permalink, "example.js"),
        );
        expected_assets_permalinks.insert(
            "posts/my-article/graph.jpg".to_string(),
            format!("{}{}", parent_permalink, "graph.jpg"),
        );
        expected_assets_permalinks.insert(
            "posts/my-article/fail.png".to_string(),
            format!("{}{}", parent_permalink, "fail.png"),
        );
        expected_assets_permalinks.insert(
            "posts/my-article/extensionless".to_string(),
            format!("{}{}", parent_permalink, "extensionless"),
        );
        expected_assets_permalinks.insert(
            "posts/my-article/subdir/example.js".to_string(),
            format!("{}{}", parent_permalink, "subdir/example.js"),
        );
        expected_assets_permalinks.insert(
            "posts/my-article/FFF.txt".to_string(),
            format!("{}{}", parent_permalink, "FFF.txt"),
        );
        expected_assets_permalinks.insert(
            "posts/my-article/GRAPH.txt".to_string(),
            format!("{}{}", parent_permalink, "GRAPH.txt"),
        );
        expected_assets_permalinks.insert(
            "posts/my-article/subdir/GGG.txt".to_string(),
            format!("{}{}", parent_permalink, "subdir/GGG.txt"),
        );

        let assets_permalinks =
            get_assets_permalinks(&serialized_assets, &parent_permalink, Some(&colocated_path));

        assert_eq!(
            assets_permalinks, expected_assets_permalinks,
            "Assets permalinks (left) are different from expected (right)",
        );
    }

    #[test]
    fn can_get_empty_assets_permalinks() {
        let serialized_assets: Vec<String> = vec![];
        let parent_permalink = "https://remplace-par-ton-url.fr/posts/my-super-article/";

        let assets_permalinks = get_assets_permalinks(&serialized_assets, &parent_permalink, None);

        assert!(assets_permalinks.is_empty());
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

    #[test]
    fn reading_analytics_no_code() {
        let (word_count, reading_time) =
            get_reading_analytics("hello world ``` code goes here ``` goodbye world");
        assert_eq!(word_count, 4);
        assert_eq!(reading_time, 1);

        let (word_count, reading_time) = get_reading_analytics(
            "hello world ``` code goes here ``` goodbye world ``` dangling fence",
        );
        assert_eq!(word_count, 4);
        assert_eq!(reading_time, 1);
    }
}
