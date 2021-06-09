use std::borrow::Cow;
use std::path::{Path, PathBuf};

/// This is used by a few Tera functions to search for files on the filesystem.
/// This does try to find the file in 3 different spots:
/// 1. base_path + path
/// 2. base_path + static + path
/// 3. base_path + content + path
/// A path starting with @/ will replace it with `content/` and a path starting with `/` will have
/// it removed.
/// It also returns the unified path so it can be used as unique hash for a given file.
pub fn search_for_file(base_path: &Path, path: &str) -> Option<(PathBuf, String)> {
    let search_paths = [base_path.join("static"), base_path.join("content")];
    let actual_path = if path.starts_with("@/") {
        Cow::Owned(path.replace("@/", "content/"))
    } else {
        Cow::Borrowed(path.trim_start_matches('/'))
    };
    let mut file_path = base_path.join(&*actual_path);
    let mut file_exists = file_path.exists();

    if !file_exists {
        // we need to search in both search folders now
        for dir in &search_paths {
            let p = dir.join(&*actual_path);
            if p.exists() {
                file_path = p;
                file_exists = true;
                break;
            }
        }
    }

    if file_exists {
        Some((file_path, actual_path.into_owned()))
    } else {
        None
    }
}
