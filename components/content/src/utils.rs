use libs::regex;
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

/// Remove Markdown footnotes
///
/// Footnotes source is [^word]
///
/// Footnotes target can be one line
/// or blocks starting by 4 spaces
///
/// Paragraph 1[^1]
/// Paragraph 2[^2]
///
/// [^1]: Footnote 1
/// [^2]: Footnote 2
///     Big footnote
///     Inside
fn remove_footnotes(content: &str) -> String {
    let re_footnote_target = regex::Regex::new(r"\[\^\w\]:").unwrap();
    let re_footnote_src = regex::Regex::new(r"(\[\^\w\])").unwrap();
    let mut content_wo_footnotes = vec![];
    let mut in_footnote = false;
    for line in content.lines() {
        if in_footnote {
            // Footnote can be a single line
            // with or without a 4 spaces block
            // Also catch multiple single line references
            in_footnote = line.starts_with("    ") || re_footnote_target.is_match(line.trim());
        } else {
            in_footnote = re_footnote_target.is_match(line.trim());
        }
        if !in_footnote {
            let clean_line = re_footnote_src.replace(line, "");
            content_wo_footnotes.push(clean_line);
        }
    }

    content_wo_footnotes.join("\n")
}

/// Remove HTML comments
/// <!-- comment -->
fn remove_html_comments(content: &str) -> String {
    let mut content_wo_comments = String::new();
    let mut cur_str = &content[..];
    while let Some(comment_start) = cur_str.find("<!--") {
        let comment_end;
        match cur_str.find("-->") {
            None => comment_end = cur_str.len(),
            Some(_end) => comment_end = _end + "-->".len(),
        }
        if comment_start != 0 {
            content_wo_comments.push_str(&cur_str[..comment_start]);
        }
        cur_str = &cur_str[comment_end..];
    }

    if cur_str.len() != 0 {
        content_wo_comments.push_str(&cur_str);
    }

    content_wo_comments
}

/// Get word count and estimated reading time
pub fn get_reading_analytics(content: &str) -> (usize, usize) {
    let content_wo_footnotes = remove_footnotes(content);
    let content_wo_comments = remove_html_comments(&content_wo_footnotes);

    // code fences "toggle" the state from non-code to code and back, so anything inbetween the
    // first fence and the next can be ignored
    let split = content_wo_comments.split("```");
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

    #[test]
    fn test_remove_html_comments() {
        let html = "String <!--comments\n -->without <!---->comments";
        let result = remove_html_comments(html);
        assert_eq!(result, "String without comments");

        let html = "String <!--comment\n without end tag";
        let result = remove_html_comments(html);
        assert_eq!(result, "String ");
    }

    #[test]
    fn test_remove_footnotes() {
        let html = "Hello
Paragraph 1[^1]
<!-- more -->
Paragraph 2[^2]
Paragraph 3[^3]

[^1]: Footnote 1
[^2]: Footnote 2
    Big footnote
    Inside

[^3]: Footnote 3";

        let result = remove_footnotes(html);
        let expected = "Hello
Paragraph 1
<!-- more -->
Paragraph 2
Paragraph 3

";
        assert_eq!(result, expected);
    }
}
