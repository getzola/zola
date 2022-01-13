mod file_info;
mod page;
mod section;
mod ser;

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

pub use self::file_info::FileInfo;
pub use self::page::Page;
pub use self::section::Section;
pub use self::ser::{SerializingPage, SerializingSection};

use config::Config;
use rendering::Heading;

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
pub fn find_related_assets(path: &Path, config: &Config, recursive: bool) -> Vec<PathBuf> {
    let mut assets = vec![];

    let mut builder = WalkDir::new(path);
    if !recursive {
        builder = builder.max_depth(1);
    }
    for entry in builder.into_iter().filter_map(std::result::Result::ok) {
        let entry_path = entry.path();
        if entry_path.is_file() {
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
        assets = assets
            .into_iter()
            .filter(|p| match p.strip_prefix(path) {
                Err(_) => false,
                Ok(file) => !globset.is_match(file),
            })
            .collect();
    }

    assets
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

        let assets = find_related_assets(path, &Config::default(), true);
        assert_eq!(assets.len(), 5);
        assert_eq!(assets.iter().filter(|p| p.extension().unwrap_or("".as_ref()) != "md").count(), 5);

        for asset in vec!["example.js", "graph.jpg", "fail.png", "subdir/example.js", "extensionless"] {
            assert!(assets
                .iter()
                .find(|p| p.strip_prefix(path).unwrap() == Path::new(asset))
                .is_some())
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
        let assets = find_related_assets(path, &Config::default(), false);
        assert_eq!(assets.len(), 4);
        assert_eq!(assets.iter().filter(|p| p.extension().unwrap_or("".as_ref()) != "md").count(), 4);

        for asset in vec!["example.js", "graph.jpg", "fail.png", "extensionless"] {
            assert!(assets
                .iter()
                .find(|p| p.strip_prefix(path).unwrap() == Path::new(asset))
                .is_some())
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
}
