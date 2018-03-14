use std::io::prelude::*;
use std::fs::{File, create_dir_all, read_dir, copy};
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use errors::{Result, ResultExt};


/// Create a file with the content given
pub fn create_file(path: &Path, content: &str) -> Result<()> {
    let mut file = File::create(&path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// Create a directory at the given path if it doesn't exist already
pub fn ensure_directory_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        create_directory(path)?;
    }
    Ok(())
}

/// Very similar to `create_dir` from the std except it checks if the folder
/// exists before creating it
pub fn create_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        create_dir_all(path)
            .chain_err(|| format!("Was not able to create folder {}", path.display()))?;
    }
    Ok(())
}

/// Return the content of a file, with error handling added
pub fn read_file(path: &Path) -> Result<String> {
    let mut content = String::new();
    File::open(path)
        .chain_err(|| format!("Failed to open '{:?}'", path.display()))?
        .read_to_string(&mut content)?;

    Ok(content)
}

/// Looks into the current folder for the path and see if there's anything that is not a .md
/// file. Those will be copied next to the rendered .html file
pub fn find_related_assets(path: &Path) -> Vec<PathBuf> {
    let mut assets = vec![];

    for entry in read_dir(path).unwrap().filter_map(|e| e.ok()) {
        let entry_path = entry.path();
        if entry_path.is_file() {
            match entry_path.extension() {
                Some(e) => match e.to_str() {
                    Some("md") => continue,
                    _ => assets.push(entry_path.to_path_buf()),
                },
                None => continue,
            }
        }
    }

    assets
}

/// Copy a file but takes into account where to start the copy as
/// there might be folders we need to create on the way
pub fn copy_file(src: &Path, dest: &PathBuf, base_path: &PathBuf) -> Result<()> {
    let relative_path = src.strip_prefix(base_path).unwrap();
    let target_path = dest.join(relative_path);

    if let Some(parent_directory) = target_path.parent() {
        create_dir_all(parent_directory)?;
    }

    copy(src, target_path)?;
    Ok(())
}

pub fn copy_directory(src: &PathBuf, dest: &PathBuf) -> Result<()> {
    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let relative_path = entry.path().strip_prefix(src).unwrap();
        let target_path = dest.join(relative_path);

        if entry.path().is_dir() {
            if !target_path.exists() {
                create_directory(&target_path)?;
            }
        } else {
            copy_file(entry.path(), dest, src)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use tempdir::TempDir;

    use super::{find_related_assets};

    #[test]
    fn can_find_related_assets() {
        let tmp_dir = TempDir::new("example").expect("create temp dir");
        File::create(tmp_dir.path().join("index.md")).unwrap();
        File::create(tmp_dir.path().join("example.js")).unwrap();
        File::create(tmp_dir.path().join("graph.jpg")).unwrap();
        File::create(tmp_dir.path().join("fail.png")).unwrap();

        let assets = find_related_assets(tmp_dir.path());
        assert_eq!(assets.len(), 3);
        assert_eq!(assets.iter().filter(|p| p.extension().unwrap() != "md").count(), 3);
        assert_eq!(assets.iter().filter(|p| p.file_name().unwrap() == "example.js").count(), 1);
        assert_eq!(assets.iter().filter(|p| p.file_name().unwrap() == "graph.jpg").count(), 1);
        assert_eq!(assets.iter().filter(|p| p.file_name().unwrap() == "fail.png").count(), 1);
    }
}
