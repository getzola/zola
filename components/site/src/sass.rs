use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use libs::walkdir::{WalkDir, DirEntry};
use libs::globset::{Glob};
use libs::sass_rs::{compile_file, Options, OutputStyle};

use crate::anyhow;
use errors::{bail, Result};
use utils::fs::{create_file, ensure_directory_exists};

pub fn compile_sass(base_path: &Path, output_path: &Path) -> Result<()> {
    ensure_directory_exists(output_path)?;

    let sass_path = {
        let mut sass_path = PathBuf::from(base_path);
        sass_path.push("sass");
        sass_path
    };

    let mut options = Options { output_style: OutputStyle::Compressed, ..Default::default() };
    let mut compiled_paths = compile_sass_glob(&sass_path, output_path, "scss", &options)?;

    options.indented_syntax = true;
    compiled_paths.extend(compile_sass_glob(&sass_path, output_path, "sass", &options)?);

    compiled_paths.sort();
    for window in compiled_paths.windows(2) {
        if window[0].1 == window[1].1 {
            bail!(
                "SASS path conflict: \"{}\" and \"{}\" both compile to \"{}\"",
                window[0].0.display(),
                window[1].0.display(),
                window[0].1.display(),
            );
        }
    }

    Ok(())
}

fn compile_sass_glob(
    sass_path: &Path,
    output_path: &Path,
    extension: &str,
    options: &Options,
) -> Result<Vec<(PathBuf, PathBuf)>> {
    let files = get_non_partial_scss(sass_path, extension);

    let mut compiled_paths = Vec::new();
    for file in files {
        let css = compile_file(&file, options.clone()).map_err(|e| anyhow!(e))?;

        let path_inside_sass = file.strip_prefix(&sass_path).unwrap();
        let parent_inside_sass = path_inside_sass.parent();
        let css_output_path = output_path.join(path_inside_sass).with_extension("css");

        if parent_inside_sass.is_some() {
            create_dir_all(&css_output_path.parent().unwrap())?;
        }

        create_file(&css_output_path, &css)?;
        compiled_paths.push((path_inside_sass.to_owned(), css_output_path));
    }

    Ok(compiled_paths)
}

fn is_partial_scss(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with("_"))
         .unwrap_or(false)
}

fn get_non_partial_scss(sass_path: &Path, extension: &str) -> Vec<PathBuf> {
    let glob_string = format!("*.{}", extension);
    let glob = Glob::new(glob_string.as_str()).expect("Invalid glob for sass").compile_matcher();
    
    WalkDir::new(sass_path)
        .into_iter()
        .filter_entry(|e| !is_partial_scss(e))
        .filter_map(|e| e.ok())
        .map(|e| e.into_path())
        .filter(|e| glob.is_match(e))
        .collect::<Vec<_>>()
}

#[test]
fn test_get_non_partial_scss() {
    use std::env;

    let mut path = env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    path.push("test_site");
    path.push("sass");

    let result = get_non_partial_scss(&path, "scss");

    assert!(!result.is_empty());
    assert!(result.iter().filter_map(|path| path.file_name()).any(|file| file == "scss.scss"))
}
#[test]
fn test_get_non_partial_scss_underscores() {
    use std::env;

    let mut path = env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    path.push("test_site");
    path.push("_dir_with_underscores");
    path.push("..");
    path.push("sass");

    let result = get_non_partial_scss(&path, "scss");

    assert!(!result.is_empty());
    assert!(result.iter().filter_map(|path| path.file_name()).any(|file| file == "scss.scss"))
}
