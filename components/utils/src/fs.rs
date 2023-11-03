use libs::filetime::{set_file_mtime, FileTime};
use libs::globset::GlobSet;
use libs::walkdir::WalkDir;
use std::fs::{copy, create_dir_all, metadata, remove_dir_all, remove_file, File};
use std::io::prelude::*;
use std::path::Path;
use std::time::SystemTime;

use errors::{Context, Result};

pub fn is_path_in_directory(parent: &Path, path: &Path) -> Result<bool> {
    let canonical_path = path
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize {}", path.display()))?;
    let canonical_parent = parent
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize {}", parent.display()))?;

    Ok(canonical_path.starts_with(canonical_parent))
}

/// Create a file with the content given
pub fn create_file(path: &Path, content: &str) -> Result<()> {
    let mut file =
        File::create(path).with_context(|| format!("Failed to create file {}", path.display()))?;
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
            .with_context(|| format!("Failed to create folder {}", path.display()))?;
    }
    Ok(())
}

/// Return the content of a file, with error handling added
pub fn read_file(path: &Path) -> Result<String> {
    let mut content = String::new();
    File::open(path)
        .with_context(|| format!("Failed to open file {}", path.display()))?
        .read_to_string(&mut content)?;

    // Remove utf-8 BOM if any.
    if content.starts_with('\u{feff}') {
        content.drain(..3);
    }

    Ok(content)
}

/// Copy a file but takes into account where to start the copy as
/// there might be folders we need to create on the way.
pub fn copy_file(src: &Path, dest: &Path, base_path: &Path, hard_link: bool) -> Result<()> {
    let relative_path = src.strip_prefix(base_path).unwrap();
    let target_path = dest.join(relative_path);

    if let Some(parent_directory) = target_path.parent() {
        create_dir_all(parent_directory).with_context(|| {
            format!("Failed to create directory {}", parent_directory.display())
        })?;
    }

    copy_file_if_needed(src, &target_path, hard_link)
}

/// No copy occurs if all of the following conditions are satisfied:
/// 1. A file with the same name already exists in the dest path.
/// 2. Its modification timestamp is identical to that of the src file.
/// 3. Its filesize is identical to that of the src file.
pub fn copy_file_if_needed(src: &Path, dest: &Path, hard_link: bool) -> Result<()> {
    if let Some(parent_directory) = dest.parent() {
        create_dir_all(parent_directory).with_context(|| {
            format!("Failed to create directory {}", parent_directory.display())
        })?;
    }

    if hard_link {
        if dest.exists() {
            std::fs::remove_file(dest)
                .with_context(|| format!("Error removing file: {:?}", dest))?;
        }
        std::fs::hard_link(src, dest)
            .with_context(|| format!("Error hard linking file, src: {:?}, dst: {:?}", src, dest))?;
    } else {
        let src_metadata = metadata(src)
            .with_context(|| format!("Failed to get metadata of {}", src.display()))?;
        let src_mtime = FileTime::from_last_modification_time(&src_metadata);
        if Path::new(&dest).is_file() {
            let target_metadata = metadata(dest)?;
            let target_mtime = FileTime::from_last_modification_time(&target_metadata);
            if !(src_mtime == target_mtime && src_metadata.len() == target_metadata.len()) {
                copy(src, dest).with_context(|| {
                    format!("Was not able to copy file {} to {}", src.display(), dest.display())
                })?;
                set_file_mtime(dest, src_mtime)?;
            }
        } else {
            copy(src, dest).with_context(|| {
                format!("Was not able to copy directory {} to {}", src.display(), dest.display())
            })?;
            set_file_mtime(dest, src_mtime)?;
        }
    }
    Ok(())
}

pub fn copy_directory(
    src: &Path,
    dest: &Path,
    hard_link: bool,
    ignore_globset: Option<&GlobSet>,
) -> Result<()> {
    for entry in
        WalkDir::new(src).follow_links(true).into_iter().filter_map(std::result::Result::ok)
    {
        let relative_path = entry.path().strip_prefix(src).unwrap();

        if let Some(gs) = ignore_globset {
            if gs.is_match(relative_path) {
                continue;
            }
        }

        let target_path = dest.join(relative_path);

        if entry.path().is_dir() {
            if !target_path.exists() {
                create_directory(&target_path)?;
            }
        } else {
            copy_file(entry.path(), dest, src, hard_link).with_context(|| {
                format!(
                    "Was not able to copy {} to {} (hard_link={})",
                    entry.path().display(),
                    dest.display(),
                    hard_link
                )
            })?;
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

/// Checks if the file or folder for the given path is a dotfile, meaning starts with '.'
pub fn is_dotfile<P>(path: P) -> bool
where
    P: AsRef<Path>,
{
    path.as_ref().file_name().and_then(|s| s.to_str()).map(|s| s.starts_with('.')).unwrap_or(false)
}

/// Returns whether the path we received corresponds to a temp file created
/// by an editor or the OS
pub fn is_temp_file(path: &Path) -> bool {
    let ext = path.extension();
    match ext {
        Some(ex) => match ex.to_str().unwrap() {
            "swp" | "swx" | "tmp" | ".DS_STORE" | ".DS_Store" => true,
            // kate
            "kate-swp" => true,
            // jetbrains IDE
            x if x.ends_with("jb_old___") => true,
            x if x.ends_with("jb_tmp___") => true,
            x if x.ends_with("jb_bak___") => true,
            // vim & jetbrains
            x if x.ends_with('~') => true,
            _ => {
                if let Some(filename) = path.file_stem() {
                    // emacs
                    let name = filename.to_str().unwrap();
                    name.starts_with('#') || name.starts_with(".#")
                } else {
                    false
                }
            }
        },
        None => true,
    }
}

/// Deletes the `output_path` directory if it exists and `preserve_dotfiles_in_output` is set to false,
/// or if set to true: its contents except for the dotfiles at the root level.
pub fn clean_site_output_folder(
    output_path: &Path,
    preserve_dotfiles_in_output: bool,
) -> Result<()> {
    if output_path.exists() {
        if !preserve_dotfiles_in_output {
            return remove_dir_all(output_path).context("Couldn't delete output directory");
        }

        for entry in output_path
            .read_dir()
            .context(format!("Couldn't read output directory `{}`", output_path.display()))?
        {
            let entry = entry.context("Couldn't read entry in output directory")?.path();

            // Skip dotfiles if the preserve_dotfiles_in_output configuration option is set
            if is_dotfile(&entry) {
                continue;
            }

            if entry.is_dir() {
                remove_dir_all(entry)
                    .context("Couldn't delete folder while cleaning the output directory")?;
            } else {
                remove_file(entry)
                    .context("Couldn't delete file while cleaning the output directory")?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::{metadata, read_to_string, File};
    use std::io::Write;
    use std::path::PathBuf;
    use std::str::FromStr;

    use libs::filetime;
    use tempfile::tempdir_in;

    use super::copy_file;

    #[test]
    fn test_copy_file_timestamp_preserved() {
        let base_path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
        let src_dir =
            tempdir_in(&base_path).expect("failed to create a temporary source directory.");
        let dest_dir =
            tempdir_in(&base_path).expect("failed to create a temporary destination directory.");
        let src_file_path = src_dir.path().join("test.txt");
        let dest_file_path = dest_dir.path().join(src_file_path.strip_prefix(&base_path).unwrap());
        File::create(&src_file_path).unwrap();
        copy_file(&src_file_path, dest_dir.path(), &base_path, false).unwrap();

        assert_eq!(
            metadata(&src_file_path).and_then(|m| m.modified()).unwrap(),
            metadata(&dest_file_path).and_then(|m| m.modified()).unwrap()
        );
    }

    #[test]
    fn test_copy_file_already_exists() {
        let base_path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
        let src_dir =
            tempdir_in(&base_path).expect("failed to create a temporary source directory.");
        let dest_dir =
            tempdir_in(&base_path).expect("failed to create a temporary destination directory.");
        let src_file_path = src_dir.path().join("test.txt");
        let dest_file_path = dest_dir.path().join(src_file_path.strip_prefix(&base_path).unwrap());
        {
            let mut src_file = File::create(&src_file_path).unwrap();
            src_file.write_all(b"file1").unwrap();
        }
        copy_file(&src_file_path, dest_dir.path(), &base_path, false).unwrap();
        {
            let mut dest_file = File::create(&dest_file_path).unwrap();
            dest_file.write_all(b"file2").unwrap();
        }

        // Check copy does not occur when moditication timestamps and filesizes are same.
        filetime::set_file_mtime(&src_file_path, filetime::FileTime::from_unix_time(0, 0)).unwrap();
        filetime::set_file_mtime(&dest_file_path, filetime::FileTime::from_unix_time(0, 0))
            .unwrap();
        copy_file(&src_file_path, dest_dir.path(), &base_path, false).unwrap();
        assert_eq!(read_to_string(&src_file_path).unwrap(), "file1");
        assert_eq!(read_to_string(&dest_file_path).unwrap(), "file2");

        // Copy occurs if the timestamps are different while the filesizes are same.
        filetime::set_file_mtime(&dest_file_path, filetime::FileTime::from_unix_time(42, 42))
            .unwrap();
        copy_file(&src_file_path, dest_dir.path(), &base_path, false).unwrap();
        assert_eq!(read_to_string(&src_file_path).unwrap(), "file1");
        assert_eq!(read_to_string(&dest_file_path).unwrap(), "file1");

        // Copy occurs if the timestamps are same while the filesizes are different.
        {
            let mut dest_file = File::create(&dest_file_path).unwrap();
            dest_file.write_all(b"This file has different file size to the source file!").unwrap();
        }
        filetime::set_file_mtime(&dest_file_path, filetime::FileTime::from_unix_time(0, 0))
            .unwrap();
        copy_file(&src_file_path, dest_dir.path(), &base_path, false).unwrap();
        assert_eq!(read_to_string(&src_file_path).unwrap(), "file1");
        assert_eq!(read_to_string(&dest_file_path).unwrap(), "file1");
    }
}
