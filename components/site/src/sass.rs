use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use glob::glob;
use sass_rs::{compile_file, Options, OutputStyle};

use errors::{bail, Result};
use utils::fs::{create_file, ensure_directory_exists};

pub fn compile_sass(base_path: &Path, output_path: &Path) -> Result<()> {
    ensure_directory_exists(&output_path)?;

    let sass_path = {
        let mut sass_path = PathBuf::from(base_path);
        sass_path.push("sass");
        sass_path
    };

    let mut options = Options::default();
    options.output_style = OutputStyle::Compressed;
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
    let glob_string = format!("{}/**/*.{}", sass_path.display(), extension);
    let files = glob(&glob_string)
        .expect("Invalid glob for sass")
        .filter_map(|e| e.ok())
        .filter(|entry| {
            !entry
                .as_path()
                .iter()
                .last()
                .map(|c| c.to_string_lossy().starts_with('_'))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    let mut compiled_paths = Vec::new();
    for file in files {
        let css = compile_file(&file, options.clone())?;

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
