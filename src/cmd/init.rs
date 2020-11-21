use std::fs::{canonicalize, create_dir};
use std::path::Path;
use std::path::PathBuf;

use errors::{bail, Result};
use utils::fs::create_file;

use crate::console;
use crate::prompt::{ask_bool, ask_url};

const CONFIG: &str = r#"
# The URL the site will be built for
base_url = "%BASE_URL%"

# Whether to automatically compile all Sass files in the sass directory
compile_sass = %COMPILE_SASS%

# Whether to build a search index to be used later on by a JavaScript library
build_search_index = %SEARCH%

[markdown]
# Whether to do syntax highlighting
# Theme can be customised by setting the `highlight_theme` variable to a theme supported by Zola
highlight_code = %HIGHLIGHT%

[extra]
# Put all your custom variables here
"#;

// canonicalize(path) function on windows system returns a path with UNC.
// Example: \\?\C:\Users\VssAdministrator\AppData\Local\Temp\new_project
// More details on Universal Naming Convention (UNC):
// https://en.wikipedia.org/wiki/Path_(computing)#Uniform_Naming_Convention
// So the following const will be used to remove the network part of the UNC to display users a more common
// path on windows systems.
// This is a workaround until this issue https://github.com/rust-lang/rust/issues/42869 was fixed.
const LOCAL_UNC: &str = "\\\\?\\";

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

// Remove the unc part of a windows path
fn strip_unc(path: &PathBuf) -> String {
    let path_to_refine = path.to_str().unwrap();
    path_to_refine.trim_start_matches(LOCAL_UNC).to_string()
}

pub fn create_new_project(name: &str, force: bool) -> Result<()> {
    let path = Path::new(name);

    // Better error message than the rust default
    if path.exists() && !is_directory_quasi_empty(&path)? && !force {
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

    populate(&path, compile_sass, &config)?;

    println!();
    console::success(&format!(
        "Done! Your site was created in {}",
        strip_unc(&canonicalize(path).unwrap())
    ));
    println!();
    console::info(
        "Get started by moving into the directory and using the built-in server: `zola serve`",
    );
    println!("Visit https://www.getzola.org for the full documentation.");
    Ok(())
}

fn populate(path: &Path, compile_sass: bool, config: &str) -> Result<()> {
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

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::fs::{create_dir, remove_dir, remove_dir_all};
    use std::path::Path;

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

    #[test]
    fn populate_existing_directory() {
        let mut dir = temp_dir();
        dir.push("test_existing_dir");
        if dir.exists() {
            remove_dir_all(&dir).expect("Could not free test directory");
        }
        create_dir(&dir).expect("Could not create test directory");
        populate(&dir, true, "").expect("Could not populate zola directories");

        assert_eq!(true, dir.join("config.toml").exists());
        assert_eq!(true, dir.join("content").exists());
        assert_eq!(true, dir.join("templates").exists());
        assert_eq!(true, dir.join("static").exists());
        assert_eq!(true, dir.join("themes").exists());
        assert_eq!(true, dir.join("sass").exists());

        remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn populate_non_existing_directory() {
        let mut dir = temp_dir();
        dir.push("test_non_existing_dir");
        if dir.exists() {
            remove_dir_all(&dir).expect("Could not free test directory");
        }
        populate(&dir, true, "").expect("Could not populate zola directories");

        assert_eq!(true, dir.exists());
        assert_eq!(true, dir.join("config.toml").exists());
        assert_eq!(true, dir.join("content").exists());
        assert_eq!(true, dir.join("templates").exists());
        assert_eq!(true, dir.join("static").exists());
        assert_eq!(true, dir.join("themes").exists());
        assert_eq!(true, dir.join("sass").exists());

        remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn populate_without_sass() {
        let mut dir = temp_dir();
        dir.push("test_wihout_sass_dir");
        if dir.exists() {
            remove_dir_all(&dir).expect("Could not free test directory");
        }
        create_dir(&dir).expect("Could not create test directory");
        populate(&dir, false, "").expect("Could not populate zola directories");

        assert_eq!(false, dir.join("sass").exists());

        remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn strip_unc_test() {
        let mut dir = temp_dir();
        dir.push("new_project");
        if dir.exists() {
            remove_dir_all(&dir).expect("Could not free test directory");
        }
        create_dir(&dir).expect("Could not create test directory");
        if cfg!(target_os = "windows") {
            assert_eq!(
                strip_unc(&canonicalize(Path::new(&dir)).unwrap()),
                "C:\\Users\\VssAdministrator\\AppData\\Local\\Temp\\new_project"
            )
        } else {
            assert_eq!(
                strip_unc(&canonicalize(Path::new(&dir)).unwrap()),
                canonicalize(Path::new(&dir)).unwrap().to_str().unwrap().to_string()
            );
        }

        remove_dir_all(&dir).unwrap();
    }

    // If the following test fails it means that the canonicalize function is fixed and strip_unc
    // function/workaround is not anymore required.
    // See issue https://github.com/rust-lang/rust/issues/42869 as a reference.
    #[test]
    #[cfg(target_os = "windows")]
    fn strip_unc_required_test() {
        let mut dir = temp_dir();
        dir.push("new_project");
        if dir.exists() {
            remove_dir_all(&dir).expect("Could not free test directory");
        }
        create_dir(&dir).expect("Could not create test directory");
        assert_eq!(
            canonicalize(Path::new(&dir)).unwrap().to_str().unwrap(),
            "\\\\?\\C:\\Users\\VssAdministrator\\AppData\\Local\\Temp\\new_project"
        );

        remove_dir_all(&dir).unwrap();
    }
}
