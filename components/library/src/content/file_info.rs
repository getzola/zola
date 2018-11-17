use std::path::{Path, PathBuf};

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
#[derive(Debug, Clone, PartialEq)]
pub struct FileInfo {
    /// The full path to the .md file
    pub path: PathBuf,
    /// The name of the .md file without the extension, always `_index` for sections
    pub name: String,
    /// The .md path, starting from the content directory, with `/` slashes
    pub relative: String,
    /// Path of the directory containing the .md file
    pub parent: PathBuf,
    /// Path of the grand parent directory for that file. Only used in sections to find subsections.
    pub grand_parent: Option<PathBuf>,
    /// The folder names to this section file, starting from the `content` directory
    /// For example a file at content/kb/solutions/blabla.md will have 2 components:
    /// `kb` and `solutions`
    pub components: Vec<String>,
}

impl FileInfo {
    pub fn new_page(path: &Path) -> FileInfo {
        let file_path = path.to_path_buf();
        let mut parent = file_path.parent().unwrap().to_path_buf();
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let mut components = find_content_components(&file_path);
        let relative = if !components.is_empty() {
            format!("{}/{}.md", components.join("/"), name)
        } else {
            format!("{}.md", name)
        };

        // If we have a folder with an asset, don't consider it as a component
        if !components.is_empty() && name == "index" {
            components.pop();
            // also set parent_path to grandparent instead
            parent = parent.parent().unwrap().to_path_buf();
        }

        FileInfo {
            path: file_path,
            // We don't care about grand parent for pages
            grand_parent: None,
            parent,
            name,
            components,
            relative,
        }
    }

    pub fn new_section(path: &Path) -> FileInfo {
        let parent = path.parent().unwrap().to_path_buf();
        let components = find_content_components(path);
        let relative = if components.is_empty() {
            // the index one
            "_index.md".to_string()
        } else {
            format!("{}/_index.md", components.join("/"))
        };
        let grand_parent = parent.parent().map(|p| p.to_path_buf());

        FileInfo {
            path: path.to_path_buf(),
            parent,
            grand_parent,
            name: "_index".to_string(),
            components,
            relative,
        }
    }
}

#[doc(hidden)]
impl Default for FileInfo {
    fn default() -> FileInfo {
        FileInfo {
            path: PathBuf::new(),
            parent: PathBuf::new(),
            grand_parent: None,
            name: String::new(),
            components: vec![],
            relative: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::find_content_components;

    #[test]
    fn can_find_content_components() {
        let res =
            find_content_components("/home/vincent/code/site/content/posts/tutorials/python.md");
        assert_eq!(res, ["posts".to_string(), "tutorials".to_string()]);
    }
}
