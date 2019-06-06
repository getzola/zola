use std::fs::{copy, create_dir_all, read_dir, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;

use errors::{Error, Result};

pub fn is_path_in_directory(parent: &Path, path: &Path) -> Result<bool> {
    let canonical_path = path
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize {}: {}", path.display(), e))?;
    let canonical_parent = parent
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize {}: {}", parent.display(), e))?;

    Ok(canonical_path.starts_with(canonical_parent))
}

/// Create a file with the content given
pub fn create_file(path: &Path, content: &str) -> Result<()> {
    let mut file =
        File::create(&path).map_err(|e| Error::chain(format!("Failed to create {:?}", path), e))?;
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
        create_dir_all(path).map_err(|e| {
            Error::chain(format!("Was not able to create folder {}", path.display()), e)
        })?;
    }
    Ok(())
}

/// Return the content of a file, with error handling added
pub fn read_file(path: &Path) -> Result<String> {
    let mut content = String::new();
    File::open(path)
        .map_err(|e| Error::chain(format!("Failed to open '{:?}'", path.display()), e))?
        .read_to_string(&mut content)?;

    // Remove utf-8 BOM if any.
    if content.starts_with("\u{feff}") {
        content.drain(..3);
    }

    Ok(content)
}

/// Return the content of a file, with error handling added.
/// The default error message is overwritten by the message given.
/// That means it is allocation 2 strings, oh well
pub fn read_file_with_error(path: &Path, message: &str) -> Result<String> {
    let res = read_file(&path);
    if res.is_ok() {
        return res;
    }
    let mut err = Error::msg(message);
    err.source = res.unwrap_err().source;
    Err(err)
}

/// Looks into the current folder for the path and see if there's anything that is not a .md
/// file. Those will be copied next to the rendered .html file
pub fn find_related_assets(path: &Path) -> Vec<PathBuf> {
    let mut assets = vec![];

    for entry in read_dir(path).unwrap().filter_map(std::result::Result::ok) {
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
    for entry in WalkDir::new(src).into_iter().filter_map(std::result::Result::ok) {
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

pub fn get_file_time(path: &Path) -> Option<SystemTime> {
    path.metadata().ok().and_then(|meta| {
        Some(match (meta.created().ok(), meta.modified().ok()) {
            (Some(tc), Some(tm)) => tc.max(tm),
            (Some(tc), None) => tc,
            (None, Some(tm)) => tm,
            (None, None) => return None,
        })
    })
}

/// Compares source and target files' timestamps and returns true if the source file
/// has been created _or_ updated after the target file has
pub fn file_stale<PS, PT>(p_source: PS, p_target: PT) -> bool
where
    PS: AsRef<Path>,
    PT: AsRef<Path>,
{
    let p_source = p_source.as_ref();
    let p_target = p_target.as_ref();

    if !p_target.exists() {
        return true;
    }

    let time_source = get_file_time(p_source);
    let time_target = get_file_time(p_target);

    time_source.and_then(|ts| time_target.map(|tt| ts > tt)).unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use tempfile::tempdir;

    use super::find_related_assets;

    #[test]
    fn can_find_related_assets() {
        let tmp_dir = tempdir().expect("create temp dir");
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
