use std::fs::{canonicalize, create_dir};
use std::path::Path;

use errors::Result;
use utils::fs::create_file;

use console;
use prompt::{ask_bool, ask_url};

const CONFIG: &str = r#"
# The URL the site will be built for
base_url = "%BASE_URL%"

# Whether to automatically compile all Sass files in the sass directory
compile_sass = %COMPILE_SASS%

# Whether to do syntax highlighting
# Theme can be customised by setting the `highlight_theme` variable to a theme supported by Zola
highlight_code = %HIGHLIGHT%

# Whether to build a search index to be used later on by a JavaScript library
build_search_index = %SEARCH%

[extra]
# Put all your custom variables here
"#;

// Given a path, return true if it is a directory and it doesn't have any
// non-hidden files, otherwise return false (path is assumed to exist)
pub fn is_directory_quasi_empty(path: &Path) -> Result<bool> {
    if path.is_dir() {
        let mut entries = match path.read_dir() {
            Ok(entries) => entries,
            Err(e) => {
                bail!(
                    "Could not read `{}` because of error: {}",
                    path.to_string_lossy().to_string(),
                    e
                );
            }
        };
        // If any entry raises an error or isn't hidden (i.e. starts with `.`), we raise an error
        if entries.any(|x| match x {
            Ok(file) => !file
                .file_name()
                .to_str()
                .expect("Could not convert filename to &str")
                .starts_with('.'),
            Err(_) => true,
        }) {
            return Ok(false);
        }
        return Ok(true);
    }

    Ok(false)
}

pub fn create_new_project(name: &str) -> Result<()> {
    let path = Path::new(name);
    // Better error message than the rust default
    if path.exists() && !is_directory_quasi_empty(&path)? {
        if name == "." {
            bail!("The current directory is not an empty folder (hidden files are ignored).");
        } else {
            bail!(
                "`{}` is not an empty folder (hidden files are ignored).",
                path.to_string_lossy().to_string()
            )
        }
    }

    console::info("Welcome to Zola!");
    console::info("Please answer a few questions to get started quickly.");
    console::info("Any choices made can be changed by modifying the `config.toml` file later.");

    let base_url = ask_url("> What is the URL of your site?", "https://example.com")?;
    let compile_sass = ask_bool("> Do you want to enable Sass compilation?", true)?;
    let highlight = ask_bool("> Do you want to enable syntax highlighting?", false)?;
    let search = ask_bool("> Do you want to build a search index of the content?", false)?;

    let config = CONFIG
        .trim_start()
        .replace("%BASE_URL%", &base_url)
        .replace("%COMPILE_SASS%", &format!("{}", compile_sass))
        .replace("%SEARCH%", &format!("{}", search))
        .replace("%HIGHLIGHT%", &format!("{}", highlight));

    if !path.exists() {
        create_dir(path)?;
    }
    create_file(&path.join("config.toml"), &config)?;

    create_dir(path.join("content"))?;
    create_dir(path.join("templates"))?;
    create_dir(path.join("static"))?;
    create_dir(path.join("themes"))?;
    if compile_sass {
        create_dir(path.join("sass"))?;
    }

    println!();
    console::success(&format!("Done! Your site was created in {:?}", canonicalize(path).unwrap()));
    println!();
    console::info(
        "Get started by moving into the directory and using the built-in server: `zola serve`",
    );
    println!("Visit https://www.getzola.org for the full documentation.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::fs::{create_dir, remove_dir, remove_dir_all};

    #[test]
    fn init_empty_directory() {
        let mut dir = temp_dir();
        dir.push("test_empty_dir");
        if dir.exists() {
            remove_dir_all(&dir).expect("Could not free test directory");
        }
        create_dir(&dir).expect("Could not create test directory");
        let allowed = is_directory_quasi_empty(&dir)
            .expect("An error happened reading the directory's contents");
        remove_dir(&dir).unwrap();
        assert_eq!(true, allowed);
    }

    #[test]
    fn init_non_empty_directory() {
        let mut dir = temp_dir();
        dir.push("test_non_empty_dir");
        if dir.exists() {
            remove_dir_all(&dir).expect("Could not free test directory");
        }
        create_dir(&dir).expect("Could not create test directory");
        let mut content = dir.clone();
        content.push("content");
        create_dir(&content).unwrap();
        let allowed = is_directory_quasi_empty(&dir)
            .expect("An error happened reading the directory's contents");
        remove_dir(&content).unwrap();
        remove_dir(&dir).unwrap();
        assert_eq!(false, allowed);
    }

    #[test]
    fn init_quasi_empty_directory() {
        let mut dir = temp_dir();
        dir.push("test_quasi_empty_dir");
        if dir.exists() {
            remove_dir_all(&dir).expect("Could not free test directory");
        }
        create_dir(&dir).expect("Could not create test directory");
        let mut git = dir.clone();
        git.push(".git");
        create_dir(&git).unwrap();
        let allowed = is_directory_quasi_empty(&dir)
            .expect("An error happened reading the directory's contents");
        remove_dir(&git).unwrap();
        remove_dir(&dir).unwrap();
        assert_eq!(true, allowed);
    }
}
