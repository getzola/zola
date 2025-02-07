use std::path::{Path, PathBuf};

use errors::{bail, Result};

/// Takes a full path to a file and returns only the components after the first `content` directory
/// Will not return the filename as last component
pub fn find_content_components<P: AsRef<Path>>(path: P) -> Vec<String> {
    let path = path.as_ref();
    let mut is_in_content = false;
    let mut components = vec![];

    for section in path.parent().unwrap().components() {
        let component = section.as_os_str().to_string_lossy();

        if is_in_content {
            components.push(component.to_string());
            continue;
        }

        if component == "content" {
            is_in_content = true;
        }
    }

    components
}

/// Struct that contains all the information about the actual file
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FileInfo {
    /// The full path to the .md file
    pub path: PathBuf,
    /// The on-disk filename, will differ from the `name` when there is a language code in it
    pub filename: String,
    /// The name of the .md file without the extension, always `_index` for sections
    /// Doesn't contain the language if there was one in the filename
    pub name: String,
    /// The .md path, starting from the content directory, with `/` slashes
    pub relative: String,
    /// The path from the content directory to the colocated directory. Ends with a `/` when set.
    /// Only filled if it is a colocated directory, None otherwise.
    pub colocated_path: Option<String>,
    /// Path of the directory containing the .md file
    pub parent: PathBuf,
    /// Path of the grand parent directory for that file. Only used in sections to find subsections.
    pub grand_parent: Option<PathBuf>,
    /// The folder names to this section file, starting from the `content` directory
    /// For example a file at content/kb/solutions/blabla.md will have 2 components:
    /// `kb` and `solutions`
    pub components: Vec<String>,
    /// This is `parent` + `name`, used to find content referring to the same content but in
    /// various languages.
    pub canonical: PathBuf,
}

impl FileInfo {
    pub fn new_page(path: &Path, base_path: &Path) -> FileInfo {
        let file_path = path.to_path_buf();
        let mut parent = file_path.parent().expect("Get parent of page").to_path_buf();
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let canonical = parent.join(&name);
        let mut components =
            find_content_components(file_path.strip_prefix(base_path).unwrap_or(&file_path));
        let relative = if !components.is_empty() {
            format!("{}/{}.md", components.join("/"), name)
        } else {
            format!("{}.md", name)
        };
        let mut colocated_path = None;

        // If we have a folder with an asset, don't consider it as a component
        // Splitting on `.` as we might have a language so it isn't *only* index but also index.fr
        // etc
        if !components.is_empty() && name.split('.').collect::<Vec<_>>()[0] == "index" {
            colocated_path = Some({
                let mut val = components.join("/");
                val.push('/');
                val
            });

            components.pop();
            // also set parent_path to grandparent instead
            parent = parent.parent().unwrap().to_path_buf();
        }

        FileInfo {
            filename: file_path.file_name().unwrap().to_string_lossy().to_string(),
            path: file_path,
            // We don't care about grand parent for pages
            grand_parent: None,
            canonical,
            parent,
            name,
            components,
            relative,
            colocated_path,
        }
    }

    pub fn new_section(path: &Path, base_path: &Path) -> FileInfo {
        let file_path = path.to_path_buf();
        let parent = path.parent().expect("Get parent of section").to_path_buf();
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let components =
            find_content_components(file_path.strip_prefix(base_path).unwrap_or(&file_path));
        let relative = if !components.is_empty() {
            format!("{}/{}.md", components.join("/"), name)
        } else {
            format!("{}.md", name)
        };
        let grand_parent = parent.parent().map(|p| p.to_path_buf());
        let mut colocated_path = None;

        // If we have a folder with an asset, don't consider it as a component
        // Splitting on `.` as we might have a language so it isn't *only* index but also index.fr
        // etc
        if !components.is_empty() && name.split('.').collect::<Vec<_>>()[0] == "_index" {
            colocated_path = Some({
                let mut val = components.join("/");
                val.push('/');
                val
            });
        }

        FileInfo {
            filename: file_path.file_name().unwrap().to_string_lossy().to_string(),
            path: file_path,
            canonical: parent.join(&name),
            parent,
            grand_parent,
            name,
            components,
            relative,
            colocated_path: colocated_path,
        }
    }

    /// Look for a language in the filename.
    /// If a language has been found, update the name of the file in this struct to
    /// remove it and return the language code
    pub fn find_language(
        &mut self,
        default_language: &str,
        other_languages: &[&str],
    ) -> Result<String> {
        // No languages? Nothing to do
        if other_languages.is_empty() {
            return Ok(default_language.to_owned());
        }

        if !self.name.contains('.') {
            return Ok(default_language.to_owned());
        }

        // Go with the assumption that no one is using `.` in filenames when using i18n
        // We can document that
        let mut parts: Vec<String> = self.name.splitn(2, '.').map(|s| s.to_string()).collect();

        // If language code is same as default language, go for default
        if default_language == parts[1].as_str() {
            return Ok(default_language.to_owned());
        }

        // The language code is not present in the config: typo or the user forgot to add it to the
        // config
        if !other_languages.contains(&parts[1].as_ref()) {
            bail!("File {:?} has a language code of {} which isn't present in the config.toml `languages`", self.path, parts[1]);
        }

        self.name = parts.swap_remove(0);
        self.canonical = self.path.parent().expect("Get parent of page path").join(&self.name);
        let lang = parts.swap_remove(0);

        Ok(lang)
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{find_content_components, FileInfo};

    #[test]
    fn can_find_content_components() {
        let res =
            find_content_components("/home/vincent/code/site/content/posts/tutorials/python.md");
        assert_eq!(res, ["posts".to_string(), "tutorials".to_string()]);
    }

    #[test]
    fn can_find_components_in_page_with_assets() {
        let file = FileInfo::new_page(
            Path::new("/home/vincent/code/site/content/posts/tutorials/python/index.md"),
            &PathBuf::new(),
        );
        assert_eq!(file.components, ["posts".to_string(), "tutorials".to_string()]);
        assert_eq!(file.colocated_path, Some("posts/tutorials/python/".to_string()));
    }

    #[test]
    fn doesnt_fail_with_multiple_content_directories_in_path() {
        let file = FileInfo::new_page(
            Path::new("/home/vincent/code/content/site/content/posts/tutorials/python/index.md"),
            &PathBuf::from("/home/vincent/code/content/site"),
        );
        assert_eq!(file.components, ["posts".to_string(), "tutorials".to_string()]);
    }

    #[test]
    fn can_find_valid_language_in_page() {
        let mut file = FileInfo::new_page(
            Path::new("/home/vincent/code/site/content/posts/tutorials/python.fr.md"),
            &PathBuf::new(),
        );
        let res = file.find_language("en", &["fr"]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), "fr");
    }

    #[test]
    fn can_find_valid_language_with_default_locale() {
        let mut file = FileInfo::new_page(
            Path::new("/home/vincent/code/site/content/posts/tutorials/python.en.md"),
            &PathBuf::new(),
        );
        let res = file.find_language("en", &["fr"]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), "en");
    }

    #[test]
    fn can_find_valid_language_in_page_with_assets() {
        let mut file = FileInfo::new_page(
            Path::new("/home/vincent/code/site/content/posts/tutorials/python/index.fr.md"),
            &PathBuf::new(),
        );
        assert_eq!(file.components, ["posts".to_string(), "tutorials".to_string()]);
        assert_eq!(file.colocated_path, Some("posts/tutorials/python/".to_string()));
        let res = file.find_language("en", &["fr"]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), "fr");
    }

    #[test]
    fn do_nothing_on_unknown_language_in_page_with_i18n_off() {
        let mut file = FileInfo::new_page(
            Path::new("/home/vincent/code/site/content/posts/tutorials/python.fr.md"),
            &PathBuf::new(),
        );
        let res = file.find_language("en", &[]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), "en");
    }

    #[test]
    fn errors_on_unknown_language_in_page_with_i18n_on() {
        let mut file = FileInfo::new_page(
            Path::new("/home/vincent/code/site/content/posts/tutorials/python.fr.md"),
            &PathBuf::new(),
        );
        let res = file.find_language("en", &["it"]);
        assert!(res.is_err());
    }

    #[test]
    fn can_find_valid_language_in_section() {
        let mut file = FileInfo::new_section(
            Path::new("/home/vincent/code/site/content/posts/tutorials/_index.fr.md"),
            &PathBuf::new(),
        );
        let res = file.find_language("en", &["fr"]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), "fr");
    }

    /// Regression test for https://github.com/getzola/zola/issues/854
    #[test]
    fn correct_canonical_for_index() {
        let file = FileInfo::new_page(
            Path::new("/home/vincent/code/site/content/posts/tutorials/python/index.md"),
            &PathBuf::new(),
        );
        assert_eq!(
            file.canonical,
            Path::new("/home/vincent/code/site/content/posts/tutorials/python/index")
        );
    }

    /// Regression test for https://github.com/getzola/zola/issues/854
    #[test]
    fn correct_canonical_after_find_language() {
        let mut file = FileInfo::new_page(
            Path::new("/home/vincent/code/site/content/posts/tutorials/python/index.fr.md"),
            &PathBuf::new(),
        );
        let res = file.find_language("en", &["fr"]);
        assert!(res.is_ok());
        assert_eq!(
            file.canonical,
            Path::new("/home/vincent/code/site/content/posts/tutorials/python/index")
        );
    }

    #[test]
    fn correct_colocated_path() {
        struct Test<'a> {
            file_info: FileInfo,
            expected_colocated_path: &'a str,
        }

        // A colocated path:
        // - MUST NOT start with a '/'
        // - MUST end with a '/'
        // Breaking those assumptions may have uncontrolled side effects in some other code, including but not limited to assets permalinks generation.
        let tests = vec![
            Test {
                file_info: FileInfo::new_page(
                    Path::new("/home/vincent/code/site/content/posts/tutorials/python/index.md"),
                    &PathBuf::new(),
                ),
                expected_colocated_path: "posts/tutorials/python/",
            },
            Test {
                file_info: FileInfo::new_section(
                    Path::new("/home/vincent/code/site/content/posts/tutorials/_index.fr.md"),
                    &PathBuf::new(),
                ),
                expected_colocated_path: "posts/tutorials/",
            },
        ];

        for test in tests {
            assert!(test.file_info.colocated_path.is_some());
            assert_eq!(
                test.file_info.colocated_path.as_ref().unwrap(),
                test.expected_colocated_path
            )
        }
    }
}
