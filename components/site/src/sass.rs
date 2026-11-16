use fs_err as fs;
use std::path::{Path, PathBuf};

use globset::Glob;
use sasso::{FsImporter, Options, OutputStyle, Syntax};
use walkdir::{DirEntry, WalkDir};

use crate::anyhow;
use errors::{Result, bail};
use utils::fs::{create_directory, create_file};

pub fn compile_sass(base_path: &Path, output_path: &Path) -> Result<()> {
    create_directory(output_path)?;

    let sass_path = {
        let mut sass_path = PathBuf::from(base_path);
        sass_path.push("sass");
        sass_path
    };

    let files = get_non_partial_scss(&sass_path);
    let mut compiled_paths = Vec::new();

    for file in files {
        let css = compile_file(&file).map_err(|e| anyhow!(e))?;

        let path_inside_sass = file.strip_prefix(&sass_path).unwrap();
        let parent_inside_sass = path_inside_sass.parent();
        let css_output_path = output_path.join(path_inside_sass).with_extension("css");

        if parent_inside_sass.is_some() {
            fs::create_dir_all(css_output_path.parent().unwrap())?;
        }

        create_file(&css_output_path, &css)?;
        compiled_paths.push((path_inside_sass.to_owned(), css_output_path));
    }

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

fn is_partial_scss(entry: &DirEntry) -> bool {
    entry.file_name().to_str().map(|s| s.starts_with('_')).unwrap_or(false)
}

fn get_non_partial_scss(sass_path: &Path) -> Vec<PathBuf> {
    let glob = Glob::new("*.{sass,scss}").expect("Invalid glob for sass").compile_matcher();

    WalkDir::new(sass_path)
        .into_iter()
        .filter_entry(|e| !is_partial_scss(e))
        .filter_map(|e| e.ok())
        .map(|e| e.into_path())
        .filter(|e| glob.is_match(e))
        .collect::<Vec<_>>()
}

fn compile_file(file: &Path) -> Result<String> {
    let source = fs::read_to_string(file)?;
    let syntax = match file.extension().and_then(|e| e.to_str()) {
        Some("sass") => Syntax::Sass,
        _ => Syntax::Scss,
    };
    let importer = FsImporter::new(vec![]);
    let url = file.to_string_lossy();
    let options = Options::new()
        .with_style(OutputStyle::Compressed)
        .with_syntax(syntax)
        .with_importer(&importer)
        .with_url(&url);
    sasso::compile(&source, &options).map_err(|e| anyhow!("{e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_non_partial_scss() {
        use std::env;

        let mut path =
            env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
        path.push("test_site");
        path.push("sass");

        let result = get_non_partial_scss(&path);

        assert!(!result.is_empty());
        assert!(result.iter().filter_map(|path| path.file_name()).any(|file| file == "scss.scss"))
    }

    #[test]
    fn test_get_non_partial_scss_underscores() {
        use std::env;

        let mut path =
            env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
        path.push("test_site");
        path.push("_dir_with_underscores");
        path.push("..");
        path.push("sass");

        let result = get_non_partial_scss(&path);

        assert!(!result.is_empty());
        assert!(result.iter().filter_map(|path| path.file_name()).any(|file| file == "scss.scss"))
    }
}
