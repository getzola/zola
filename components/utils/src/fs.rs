use filetime::{FileTime, set_file_mtime};
use fs_err as fs;
use globset::GlobSet;
use rayon::prelude::*;
use same_file::is_same_file;
use std::io::prelude::*;
use std::path::Path;
use std::time::SystemTime;
use walkdir::WalkDir;

use errors::{Context, Result};

pub fn is_path_in_directory(parent: &Path, path: &Path) -> Result<bool> {
    let canonical_path = fs::canonicalize(path)?;
    let canonical_parent = fs::canonicalize(parent)?;

    Ok(canonical_path.starts_with(canonical_parent))
}

/// Creates the parent of a directory, if needed.
fn create_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        create_directory(parent)?;
    }
    Ok(())
}

/// Create a file with the content given
/// `content`` can be `&str`, `String`, or `&String` (and probably others)
pub fn create_file(path: &Path, content: impl AsRef<str>) -> Result<()> {
    create_parent(path)?;
    let mut file = fs::File::create(path)
        .with_context(|| format!("Failed to create file {}", path.display()))?;
    file.write_all(content.as_ref().as_bytes())?;
    Ok(())
}

/// Very similar to `create_dir` from the std except it checks if the folder
/// exists before creating it
pub fn create_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create folder {}", path.display()))?;
    }
    Ok(())
}

/// Return the content of a file, with error handling added
pub fn read_file(path: &Path) -> Result<String> {
    let mut content = String::new();
    fs::File::open(path)
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
    let relative_path = src.strip_prefix(base_path)?;
    let target_path = dest.join(relative_path);

    copy_file_if_needed(src, &target_path, hard_link)
}

/// No copy occurs if all of the following conditions are satisfied:
/// 1. A file with the same name already exists in the dest path.
/// 2. Its modification timestamp is identical to that of the src file.
/// 3. Its filesize is identical to that of the src file.
pub fn copy_file_if_needed(src: &Path, dest: &Path, hard_link: bool) -> Result<()> {
    if hard_link {
        if is_same_file(src, dest).unwrap_or(false) {
            return Ok(());
        }
        create_parent(dest)?;
        if dest.exists() {
            fs::remove_file(dest).with_context(|| format!("Error removing file: {:?}", dest))?;
        }
        fs::hard_link(src, dest)
            .with_context(|| format!("Error hard linking file, src: {:?}, dst: {:?}", src, dest))?;
    } else {
        let src_metadata = fs::metadata(src)
            .with_context(|| format!("Failed to get metadata of {}", src.display()))?;
        let src_mtime = FileTime::from_last_modification_time(&src_metadata);

        match fs::metadata(dest) {
            Ok(target_metadata) if target_metadata.is_file() => {
                let target_mtime = FileTime::from_last_modification_time(&target_metadata);
                if !(src_mtime == target_mtime && src_metadata.len() == target_metadata.len()) {
                    create_parent(dest)?;
                    fs::copy(src, dest).with_context(|| {
                        format!("Was not able to copy file {} to {}", src.display(), dest.display())
                    })?;
                    set_file_mtime(dest, src_mtime)?;
                }
            }
            _ => {
                create_parent(dest)?;
                fs::copy(src, dest).with_context(|| {
                    format!("Was not able to copy file {} to {}", src.display(), dest.display())
                })?;
                set_file_mtime(dest, src_mtime)?;
            }
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
    let mut dirs = Vec::new();
    let mut files = Vec::new();

    for entry in
        WalkDir::new(src).follow_links(true).into_iter().filter_map(std::result::Result::ok)
    {
        let relative_path = entry.path().strip_prefix(src).unwrap();

        if let Some(gs) = ignore_globset
            && gs.is_match(relative_path)
        {
            continue;
        }

        let target_path = dest.join(relative_path);

        if entry.path().is_dir() {
            dirs.push(target_path);
        } else {
            files.push((entry.into_path(), target_path));
        }
    }

    for dir in dirs {
        if !dir.exists() {
            create_directory(&dir)?;
        }
    }

    files.into_par_iter().try_for_each(|(src_file, target_file)| {
        copy_file_if_needed(&src_file, &target_file, hard_link).with_context(|| {
            format!(
                "Was not able to copy {} to {} (hard_link={})",
                src_file.display(),
                dest.display(),
                hard_link
            )
        })
    })?;

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
            // helix
            x if x.ends_with("bck") => true,
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
            return fs::remove_dir_all(output_path).context("Couldn't delete output directory");
        }

        for entry in fs::read_dir(output_path)
            .context(format!("Couldn't read output directory `{}`", output_path.display()))?
        {
            let entry = entry.context("Couldn't read entry in output directory")?.path();

            // Skip dotfiles if the preserve_dotfiles_in_output configuration option is set
            if is_dotfile(&entry) {
                continue;
            }

            if entry.is_dir() {
                fs::remove_dir_all(entry)
                    .context("Couldn't delete folder while cleaning the output directory")?;
            } else {
                fs::remove_file(entry)
                    .context("Couldn't delete file while cleaning the output directory")?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use fs_err as fs;
    use std::io::Write;
    use std::path::PathBuf;
    use std::str::FromStr;

    use filetime;
    use tempfile::tempdir_in;

    use super::{copy_directory, copy_file, copy_file_if_needed};
    use globset::GlobSetBuilder;
    use same_file::is_same_file;

    #[test]
    fn test_copy_file_timestamp_preserved() {
        let base_path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
        let src_dir =
            tempdir_in(&base_path).expect("failed to create a temporary source directory.");
        let dest_dir =
            tempdir_in(&base_path).expect("failed to create a temporary destination directory.");
        let src_file_path = src_dir.path().join("test.txt");
        let dest_file_path = dest_dir.path().join(src_file_path.strip_prefix(&base_path).unwrap());
        fs::File::create(&src_file_path).unwrap();
        copy_file(&src_file_path, dest_dir.path(), &base_path, false).unwrap();

        assert_eq!(
            fs::metadata(&src_file_path).and_then(|m| m.modified()).unwrap(),
            fs::metadata(&dest_file_path).and_then(|m| m.modified()).unwrap()
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
            let mut src_file = fs::File::create(&src_file_path).unwrap();
            src_file.write_all(b"file1").unwrap();
        }
        copy_file(&src_file_path, dest_dir.path(), &base_path, false).unwrap();
        {
            let mut dest_file = fs::File::create(&dest_file_path).unwrap();
            dest_file.write_all(b"file2").unwrap();
        }

        // Check copy does not occur when moditication timestamps and filesizes are same.
        filetime::set_file_mtime(&src_file_path, filetime::FileTime::from_unix_time(0, 0)).unwrap();
        filetime::set_file_mtime(&dest_file_path, filetime::FileTime::from_unix_time(0, 0))
            .unwrap();
        copy_file(&src_file_path, dest_dir.path(), &base_path, false).unwrap();
        assert_eq!(fs::read_to_string(&src_file_path).unwrap(), "file1");
        assert_eq!(fs::read_to_string(&dest_file_path).unwrap(), "file2");

        // Copy occurs if the timestamps are different while the filesizes are same.
        filetime::set_file_mtime(&dest_file_path, filetime::FileTime::from_unix_time(42, 42))
            .unwrap();
        copy_file(&src_file_path, dest_dir.path(), &base_path, false).unwrap();
        assert_eq!(fs::read_to_string(&src_file_path).unwrap(), "file1");
        assert_eq!(fs::read_to_string(&dest_file_path).unwrap(), "file1");

        // Copy occurs if the timestamps are same while the filesizes are different.
        {
            let mut dest_file = fs::File::create(&dest_file_path).unwrap();
            dest_file.write_all(b"This file has different file size to the source file!").unwrap();
        }
        filetime::set_file_mtime(&dest_file_path, filetime::FileTime::from_unix_time(0, 0))
            .unwrap();
        copy_file(&src_file_path, dest_dir.path(), &base_path, false).unwrap();
        assert_eq!(fs::read_to_string(&src_file_path).unwrap(), "file1");
        assert_eq!(fs::read_to_string(&dest_file_path).unwrap(), "file1");
    }

    #[test]
    fn test_copy_file_hard_link() {
        let base_path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
        let src_dir =
            tempdir_in(&base_path).expect("failed to create a temporary source directory.");
        let dest_dir =
            tempdir_in(&base_path).expect("failed to create a temporary destination directory.");
        let src_file_path = src_dir.path().join("test.txt");
        let dest_file_path = dest_dir.path().join("test.txt");

        fs::write(&src_file_path, b"hello hard link").unwrap();

        // 1. Initial hard link creation
        copy_file_if_needed(&src_file_path, &dest_file_path, true).unwrap();
        assert!(is_same_file(&src_file_path, &dest_file_path).unwrap());

        // 2. Redundant call should succeed and preserve hard link without errors
        copy_file_if_needed(&src_file_path, &dest_file_path, true).unwrap();
        assert!(is_same_file(&src_file_path, &dest_file_path).unwrap());

        // 3. If dest is overwritten with a separate file, hard link replaces it
        fs::remove_file(&dest_file_path).unwrap();
        fs::write(&dest_file_path, b"different file").unwrap();
        assert!(!is_same_file(&src_file_path, &dest_file_path).unwrap());

        copy_file_if_needed(&src_file_path, &dest_file_path, true).unwrap();
        assert!(is_same_file(&src_file_path, &dest_file_path).unwrap());
        assert_eq!(fs::read_to_string(&dest_file_path).unwrap(), "hello hard link");
    }

    #[test]
    fn test_copy_directory() {
        let base_path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
        let src_dir =
            tempdir_in(&base_path).expect("failed to create a temporary source directory.");
        let dest_dir =
            tempdir_in(&base_path).expect("failed to create a temporary destination directory.");

        // Create structure:
        // src/a.txt
        // src/sub/b.txt
        // src/ignored.tmp
        fs::write(src_dir.path().join("a.txt"), b"aaa").unwrap();
        fs::create_dir_all(src_dir.path().join("sub")).unwrap();
        fs::write(src_dir.path().join("sub").join("b.txt"), b"bbb").unwrap();
        fs::write(src_dir.path().join("ignored.tmp"), b"ignore me").unwrap();

        let mut builder = GlobSetBuilder::new();
        builder.add(globset::Glob::new("*.tmp").unwrap());
        let globset = builder.build().unwrap();

        copy_directory(src_dir.path(), dest_dir.path(), false, Some(&globset)).unwrap();

        assert_eq!(fs::read_to_string(dest_dir.path().join("a.txt")).unwrap(), "aaa");
        assert_eq!(fs::read_to_string(dest_dir.path().join("sub").join("b.txt")).unwrap(), "bbb");
        assert!(!dest_dir.path().join("ignored.tmp").exists());

        // Test with hard links
        let dest_link_dir =
            tempdir_in(&base_path).expect("failed to create a temporary destination directory.");
        copy_directory(src_dir.path(), dest_link_dir.path(), true, None).unwrap();

        assert!(
            is_same_file(src_dir.path().join("a.txt"), dest_link_dir.path().join("a.txt")).unwrap()
        );
        assert!(
            is_same_file(
                src_dir.path().join("sub").join("b.txt"),
                dest_link_dir.path().join("sub").join("b.txt")
            )
            .unwrap()
        );
        assert!(dest_link_dir.path().join("ignored.tmp").exists());
    }
}
