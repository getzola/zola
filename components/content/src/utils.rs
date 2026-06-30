use std::path::{Path, PathBuf};

use unicode_segmentation::UnicodeSegmentation;
use walkdir::WalkDir;

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

    assets.sort_by_cached_key(|a| a.to_str().unwrap().to_ascii_lowercase());

    assets
}

/// Get word count and estimated reading time.
///
/// Rates are silent-reading speeds of young adults for non-fiction text,
/// sourced from each language's ophthalmology, education, or media research
/// society journals.
///
/// `zh`/`ja` use character counts (cpm); other languages use word counts via
/// `unicode_words()`. Unlisted languages fall back to 200 wpm.
///
/// Per-language reading rates:
///
/// - en (238 wpm) — Brysbaert (2019)
///   "How many words do we read per minute? A review and meta-analysis of reading rate"
///   J. of Memory and Language.
///   https://doi.org/10.1016/j.jml.2019.104047
///
/// - ko (202 wpm) — 송지호・김재형・형성민 (2016)
///   「한국어 읽기 속도 측정 애플리케이션의 유효성 및 정상인의 읽기 속도에 대한 사전 연구」
///   대한안과학회지 57(4), 642-649.
///   https://www.kci.go.kr/kciportal/ci/sereArticleSearch/ciSereArtiView.kci?sereArticleSearchBean.artiId=ART002099342
///
/// - zh (439 cpm) — 王影超・李赛男・宋子明・闫国利 (2024)
///   《不同阅读方式对汉语句子阅读中词频效应的影响》
///   心理与行为研究 22(2), 183-188.
///   https://psybeh.tjnu.edu.cn/CN/10.12139/j.1672-0628.2024.02.005
///
/// - ja (653 cpm) — 小林潤平・川嶋稔夫 (2018)
///   「日本語文章の読み速度の個人差をもたらす眼球運動」
///   映像情報メディア学会誌.
///   https://doi.org/10.3169/itej.72.J154
pub fn get_reading_analytics(content: &str, lang: &str) -> (usize, usize) {
    // code fences "toggle" the state from non-code to code and back, so anything inbetween the
    // first fence and the next can be ignored
    let stripped = content.split("```").step_by(2);
    let primary = lang.split('-').next().unwrap_or(lang);

    let (count, per_minute): (usize, usize) = match primary {
        "zh" => (stripped.map(count_cjk_chars).sum(), 439),
        "ja" => (stripped.map(count_cjk_chars).sum(), 653),
        _ => {
            let count = stripped.map(|s| s.unicode_words().count()).sum();
            let wpm = match primary {
                "en" => 238,
                "ko" => 202,
                _ => 200,
            };
            (count, wpm)
        }
    };

    (count, count.div_ceil(per_minute))
}

fn count_cjk_chars(s: &str) -> usize {
    s.chars().filter(|c| c.is_alphabetic()).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    use config::Config;
    use fs_err as fs;
    use tempfile::tempdir;

    #[test]
    fn can_find_related_assets_recursive() {
        let tmp_dir = tempdir().expect("create temp dir");
        let path = tmp_dir.path();
        fs::File::create(path.join("index.md")).unwrap();
        fs::File::create(path.join("example.js")).unwrap();
        fs::File::create(path.join("graph.jpg")).unwrap();
        fs::File::create(path.join("fail.png")).unwrap();
        fs::File::create(path.join("extensionless")).unwrap();
        fs::create_dir(path.join("subdir")).expect("create subdir temp dir");
        fs::File::create(path.join("subdir").join("index.md")).unwrap();
        fs::File::create(path.join("subdir").join("example.js")).unwrap();
        fs::File::create(path.join("FFF.txt")).unwrap();
        fs::File::create(path.join("GRAPH.txt")).unwrap();
        fs::File::create(path.join("subdir").join("GGG.txt")).unwrap();

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
        fs::File::create(path.join("index.md")).unwrap();
        fs::File::create(path.join("example.js")).unwrap();
        fs::File::create(path.join("graph.jpg")).unwrap();
        fs::File::create(path.join("fail.png")).unwrap();
        fs::File::create(path.join("extensionless")).unwrap();
        fs::create_dir(path.join("subdir")).expect("create subdir temp dir");
        fs::File::create(path.join("subdir").join("index.md")).unwrap();
        fs::File::create(path.join("subdir").join("example.js")).unwrap();
        fs::File::create(path.join("FFF.txt")).unwrap();
        fs::File::create(path.join("GRAPH.txt")).unwrap();
        fs::File::create(path.join("subdir").join("GGG.txt")).unwrap();

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
        let (word_count, reading_time) = get_reading_analytics("  ", "en");
        assert_eq!(word_count, 0);
        assert_eq!(reading_time, 0);
    }

    #[test]
    fn reading_analytics_short_text() {
        let (word_count, reading_time) = get_reading_analytics("Hello World", "en");
        assert_eq!(word_count, 2);
        assert_eq!(reading_time, 1);
    }

    #[test]
    fn reading_analytics_long_text() {
        let mut content = String::new();
        for _ in 0..1000 {
            content.push_str(" Hello world");
        }
        let (word_count, reading_time) = get_reading_analytics(&content, "en");
        assert_eq!(word_count, 2000);
        assert_eq!(reading_time, 9);
    }

    #[test]
    fn reading_analytics_no_code() {
        let (word_count, reading_time) =
            get_reading_analytics("hello world ``` code goes here ``` goodbye world", "en");
        assert_eq!(word_count, 4);
        assert_eq!(reading_time, 1);

        let (word_count, reading_time) = get_reading_analytics(
            "hello world ``` code goes here ``` goodbye world ``` dangling fence",
            "en",
        );
        assert_eq!(word_count, 4);
        assert_eq!(reading_time, 1);
    }

    #[test]
    fn reading_analytics_chinese_counts_chars() {
        let (count, time) = get_reading_analytics("你好世界，这是中文测试。", "zh");
        assert_eq!(count, 10);
        assert_eq!(time, 1);
    }

    #[test]
    fn reading_analytics_japanese_counts_chars() {
        let (count, time) = get_reading_analytics("こんにちは、世界。", "ja");
        assert_eq!(count, 7);
        assert_eq!(time, 1);
    }

    #[test]
    fn reading_analytics_zh_variant_subtag() {
        let (count, _) = get_reading_analytics("你好世界", "zh-Hant-TW");
        assert_eq!(count, 4);
    }

    #[test]
    fn reading_analytics_unknown_lang_falls_back() {
        let mut content = String::new();
        for _ in 0..1000 {
            content.push_str(" Hello world");
        }
        let (word_count, reading_time) = get_reading_analytics(&content, "xx");
        assert_eq!(word_count, 2000);
        assert_eq!(reading_time, 10);
    }
}
