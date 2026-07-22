use std::borrow::Cow;
use std::path::{Path, PathBuf};

use errors::{Result, bail};
use utils::fs::is_path_in_directory;

/// This is used by a few Tera functions to search for files on the filesystem.
/// This does try to find the file in 5 different spots:
/// 1. base_path + path
/// 2. base_path + static + path
/// 3. base_path + content + path
/// 4. base_path + {output dir} + path
/// 5. base_path + themes + {current_theme} + static + path
///
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
    let output_search_path = base_path.join(output_path);
    let mut search_paths = vec![
        (base_path.to_path_buf(), base_path.to_path_buf()),
        (base_path.join("static"), base_path.to_path_buf()),
        (base_path.join("content"), base_path.to_path_buf()),
        (output_search_path.clone(), output_search_path),
    ];
    if let Some(t) = theme {
        search_paths
            .push((base_path.join("themes").join(t).join("static"), base_path.to_path_buf()));
    }
    let actual_path = if path.starts_with("@/") {
        Cow::Owned(path.replace("@/", "content/"))
    } else {
        Cow::Borrowed(path.trim_start_matches('/'))
    };

    for (dir, allowed_root) in search_paths {
        let file_path = dir.join(&*actual_path);
        if file_path.exists() {
            if !is_path_in_directory(&allowed_root, &file_path)? {
                bail!("{:?} is not inside the base site directory {:?}", path, base_path);
            }
            return Ok(Some((file_path, actual_path.into_owned())));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::search_for_file;
    use std::fs;

    #[test]
    fn rejects_escape_through_alternate_search_path() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().join("site");
        fs::create_dir_all(base.join("static")).unwrap();
        fs::write(temp.path().join("outside.txt"), "outside").unwrap();

        let result = search_for_file(&base, "../../outside.txt", &None, &base.join("public"));

        assert!(result.unwrap_err().to_string().contains("is not inside"));
    }

    #[test]
    fn finds_file_in_static_directory() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().join("site");
        let expected = base.join("static").join("asset.txt");
        fs::create_dir_all(expected.parent().unwrap()).unwrap();
        fs::write(&expected, "asset").unwrap();

        let (resolved, unified) =
            search_for_file(&base, "asset.txt", &None, &base.join("public")).unwrap().unwrap();

        assert_eq!(resolved, expected);
        assert_eq!(unified, "asset.txt");
    }

    #[test]
    fn finds_file_in_external_output_directory() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().join("site");
        let output = temp.path().join("output");
        let expected = output.join("asset.txt");
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(&output).unwrap();
        fs::write(&expected, "asset").unwrap();

        let (resolved, unified) =
            search_for_file(&base, "asset.txt", &None, &output).unwrap().unwrap();

        assert_eq!(resolved, expected);
        assert_eq!(unified, "asset.txt");
    }
}
