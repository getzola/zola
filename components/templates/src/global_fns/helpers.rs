use std::borrow::Cow;
use std::path::{Path, PathBuf};

use errors::{bail, Result};
use utils::fs::is_path_in_directory;

/// This is used by a few Tera functions to search for files on the filesystem.
/// This does try to find the file in 5 different spots:
/// 1. base_path + path
/// 2. base_path + static + path
/// 3. base_path + content + path
/// 4. base_path + {output dir} + path
/// 5. base_path + themes + {current_theme} + static + path
/// A path starting with @/ will replace it with `content/` and a path starting with `/` will have
/// it removed.
/// It also returns the unified path so it can be used as unique hash for a given file.
/// It will error if the file is not contained in the Zola directory.
pub fn search_for_file(
    base_path: &Path,
    path: &str,
    theme: &Option<String>,
    output_path: &Path,
) -> Result<Option<(PathBuf, String)>> {
    let mut search_paths =
        vec![base_path.join("static"), base_path.join("content"), base_path.join(output_path)];
    if let Some(t) = theme {
        search_paths.push(base_path.join("themes").join(t).join("static"));
    }
    let actual_path = if path.starts_with("@/") {
        Cow::Owned(path.replace("@/", "content/"))
    } else {
        Cow::Borrowed(path.trim_start_matches('/'))
    };

    let mut file_path = base_path.join(&*actual_path);
    let mut file_exists = file_path.exists();

    if file_exists && !is_path_in_directory(base_path, &file_path)? {
        bail!("{:?} is not inside the base site directory {:?}", path, base_path);
    }

    if !file_exists {
        // we need to search in all search folders now
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
        Ok(Some((file_path, actual_path.into_owned())))
    } else {
        Ok(None)
    }
}
