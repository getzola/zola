use fs_err as fs;
use std::path::Path;

use errors::{Result, bail};
use utils::fs::create_file;

use crate::prompt::{ask_bool, ask_url};

const CONFIG: &str = r#"
# The URL the site will be built for
base_url = "%BASE_URL%"

# Whether to automatically compile all Sass files in the sass directory
compile_sass = %COMPILE_SASS%

# Whether to build a search index to be used later on by a JavaScript library
build_search_index = %SEARCH%

[markdown]

[markdown.highlighting]
theme = "catppuccin-mocha"

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
                bail!("Could not read `{}` because of error: {}", path.to_string_lossy(), e);
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
fn strip_unc(path: &Path) -> String {
    let path_to_refine = path.to_str().unwrap();
    path_to_refine.trim_start_matches(LOCAL_UNC).to_string()
}

pub fn create_new_project(name: &str, force: bool) -> Result<()> {
    let path = Path::new(name);

    // Better error message than the rust default
    if path.exists() && !is_directory_quasi_empty(path)? && !force {
        if name == "." {
            bail!("The current directory is not an empty folder (hidden files are ignored).");
        } else {
            bail!("`{}` is not an empty folder (hidden files are ignored).", path.to_string_lossy())
        }
    }

    console::info("Welcome to Zola!");
    console::info("Please answer a few questions to get started quickly.");
    console::info("Any choices made can be changed by modifying the `zola.toml` file later.");

    let base_url = ask_url("> What is the URL of your site?", "https://example.com")?;
    let compile_sass = ask_bool("> Do you want to enable Sass compilation?", true)?;
    let search = ask_bool("> Do you want to build a search index of the content?", false)?;

    let config = CONFIG
        .trim_start()
        .replace("%BASE_URL%", &base_url)
        .replace("%COMPILE_SASS%", &format!("{compile_sass}"))
        .replace("%SEARCH%", &format!("{search}"));

    populate(path, compile_sass, &config)?;

    println!();
    console::success(&format!(
        "Done! Your site was created in {}",
        strip_unc(&fs::canonicalize(path).unwrap())
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
        fs::create_dir(path)?;
    }
    create_file(&path.join("zola.toml"), config)?;
    fs::create_dir(path.join("content"))?;
    fs::create_dir(path.join("templates"))?;
    fs::create_dir(path.join("static"))?;
    fs::create_dir(path.join("themes"))?;
    if compile_sass {
        fs::create_dir(path.join("sass"))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use fs_err as fs;
    use std::env::temp_dir;
    use std::path::Path;

    #[test]
    fn init_empty_directory() {
        let mut dir = temp_dir();
        dir.push("test_empty_dir");
        if dir.exists() {
            fs::remove_dir_all(&dir).expect("Could not free test directory");
        }
        fs::create_dir(&dir).expect("Could not create test directory");
        let allowed = is_directory_quasi_empty(&dir)
            .expect("An error happened reading the directory's contents");
        fs::remove_dir(&dir).unwrap();
        assert!(allowed);
    }

    #[test]
    fn init_non_empty_directory() {
        let mut dir = temp_dir();
        dir.push("test_non_empty_dir");
        if dir.exists() {
            fs::remove_dir_all(&dir).expect("Could not free test directory");
        }
        fs::create_dir(&dir).expect("Could not create test directory");
        let mut content = dir.clone();
        content.push("content");
        fs::create_dir(&content).unwrap();
        let allowed = is_directory_quasi_empty(&dir)
            .expect("An error happened reading the directory's contents");
        fs::remove_dir(&content).unwrap();
        fs::remove_dir(&dir).unwrap();
        assert!(!allowed);
    }

    #[test]
    fn init_quasi_empty_directory() {
        let mut dir = temp_dir();
        dir.push("test_quasi_empty_dir");
        if dir.exists() {
            fs::remove_dir_all(&dir).expect("Could not free test directory");
        }
        fs::create_dir(&dir).expect("Could not create test directory");
        let mut git = dir.clone();
        git.push(".git");
        fs::create_dir(&git).unwrap();
        let allowed = is_directory_quasi_empty(&dir)
            .expect("An error happened reading the directory's contents");
        fs::remove_dir(&git).unwrap();
        fs::remove_dir(&dir).unwrap();
        assert!(allowed);
    }

    #[test]
    fn populate_existing_directory() {
        let mut dir = temp_dir();
        dir.push("test_existing_dir");
        if dir.exists() {
            fs::remove_dir_all(&dir).expect("Could not free test directory");
        }
        fs::create_dir(&dir).expect("Could not create test directory");
        populate(&dir, true, "").expect("Could not populate zola directories");

        assert!(dir.join("zola.toml").exists());
        assert!(dir.join("content").exists());
        assert!(dir.join("templates").exists());
        assert!(dir.join("static").exists());
        assert!(dir.join("themes").exists());
        assert!(dir.join("sass").exists());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn populate_non_existing_directory() {
        let mut dir = temp_dir();
        dir.push("test_non_existing_dir");
        if dir.exists() {
            fs::remove_dir_all(&dir).expect("Could not free test directory");
        }
        populate(&dir, true, "").expect("Could not populate zola directories");

        assert!(dir.exists());
        assert!(dir.join("zola.toml").exists());
        assert!(dir.join("content").exists());
        assert!(dir.join("templates").exists());
        assert!(dir.join("static").exists());
        assert!(dir.join("themes").exists());
        assert!(dir.join("sass").exists());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn populate_without_sass() {
        let mut dir = temp_dir();
        dir.push("test_wihout_sass_dir");
        if dir.exists() {
            fs::remove_dir_all(&dir).expect("Could not free test directory");
        }
        fs::create_dir(&dir).expect("Could not create test directory");
        populate(&dir, false, "").expect("Could not populate zola directories");

        assert!(!dir.join("sass").exists());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn strip_unc_test() {
        let mut dir = temp_dir();
        dir.push("new_project1");
        if dir.exists() {
            fs::remove_dir_all(&dir).expect("Could not free test directory");
        }
        fs::create_dir(&dir).expect("Could not create test directory");
        if cfg!(target_os = "windows") {
            let stripped_path = strip_unc(&fs::canonicalize(Path::new(&dir)).unwrap());
            assert!(same_file::is_same_file(Path::new(&stripped_path), &dir).unwrap());
            assert!(!stripped_path.starts_with(LOCAL_UNC), "The path was not stripped.");
        } else {
            assert_eq!(
                strip_unc(&fs::canonicalize(Path::new(&dir)).unwrap()),
                fs::canonicalize(Path::new(&dir)).unwrap().to_str().unwrap().to_string()
            );
        }

        fs::remove_dir_all(&dir).unwrap();
    }

    // If the following test fails it means that the canonicalize function is fixed and strip_unc
    // function/workaround is not anymore required.
    // See issue https://github.com/rust-lang/rust/issues/42869 as a reference.
    #[test]
    #[cfg(target_os = "windows")]
    fn strip_unc_required_test() {
        let mut dir = temp_dir();
        dir.push("new_project2");
        if dir.exists() {
            fs::remove_dir_all(&dir).expect("Could not free test directory");
        }
        fs::create_dir(&dir).expect("Could not create test directory");

        let canonicalized_path = fs::canonicalize(Path::new(&dir)).unwrap();
        assert!(same_file::is_same_file(Path::new(&canonicalized_path), &dir).unwrap());
        assert!(canonicalized_path.to_str().unwrap().starts_with(LOCAL_UNC));

        fs::remove_dir_all(&dir).unwrap();
    }
}
