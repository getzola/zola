use lazy_static::lazy_static;
use regex::Regex;

use std::path::{Path, PathBuf};

lazy_static! {
    // Based on https://regex101.com/r/H2n38Z/1/tests
    // A regex parsing RFC3339 date followed by {_,-}, and some characters (name)
    static ref RFC3339_DATE: Regex = Regex::new(
        r"^(?P<datetime>(\d{4})-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])(T([01][0-9]|2[0-3]):([0-5][0-9]):([0-5][0-9]|60)(\.[0-9]+)?(Z|(\+|-)([01][0-9]|2[0-3]):([0-5][0-9])))?)(_|-)(?P<name>.+$)"
    ).unwrap();
}

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

/// Return a (name, maybe_lang) tuple if name has a possible language in it
pub fn get_language<S: AsRef<str>>(name: S) -> (String, Option<String>) {
    // Go with the assumption that no one is using '.' in filenames (regardless of whether the site
    // is multilingual)
    if !name.as_ref().contains('.') {
        return (name.as_ref().to_string(), None);
    }

    let parts: Vec<String> = name.as_ref().splitn(2, '.').map(|s| s.to_string()).collect();
    (parts[0].clone(), Some(parts[1].clone()))
}

/// Return a (name, date) tuple if name has a date in it
///
/// This only makes sense for pages, not sections.
pub fn get_date<S: AsRef<str>>(name: S) -> (String, Option<String>) {
    if let Some(ref caps) = RFC3339_DATE.captures(name.as_ref()) {
        return (
            caps.name("name").unwrap().as_str().to_string(),
            Some(caps.name("datetime").unwrap().as_str().to_string()),
        );
    }
    (name.as_ref().to_string(), None)
}

/// Strip `{date}{_,-}` prefix and `.{lang}` suffix from name
pub fn clear_filename<S: AsRef<str>>(name: S) -> String {
    let (name, _) = get_language(name);
    let (name, _) = get_date(name);
    name
}

/// Struct that contains all the information about the actual file and data extracted from its name
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FileInfo {
    /// The full path to the .md file
    pub path: PathBuf,
    /// The on-disk filename, will differ from the `name` when there is a language code or date in it
    pub filename: String,
    /// The name of the .md file without the extension, always `_index` for sections
    /// Doesn't contain the language if there was one in the filename.
    /// Doesn't contain the date if it was specified in the filename.
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
    /// This is `parent` + `name`, used to find content referring to the same content but in
    /// various languages.
    pub canonical: PathBuf,
    /// The `lang` part of the {name}.{lang}.md naming scheme. There is no guarantee that this
    /// language is enabled for the site.
    pub maybe_lang: Option<String>,
    /// The `date` part of the {date}{_,-}{name}.md naming scheme. Guranteed to be a RFC3339
    /// date in valid form
    pub date: Option<String>,
}

impl FileInfo {
    pub fn new_page(path: &Path, base_path: &PathBuf) -> FileInfo {
        let file_path = path.to_path_buf();
        let mut parent = file_path.parent().expect("Get parent of page").to_path_buf();
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let mut components =
            find_content_components(&file_path.strip_prefix(base_path).unwrap_or(&file_path));
        assert!(!name.is_empty());
        let relative = if !components.is_empty() {
            format!("{}/{}.md", components.join("/"), name)
        } else {
            format!("{}.md", name)
        };
        let (name, maybe_lang) = get_language(name);
        let (name, mut date) = get_date(&name);
        let canonical = parent.join(&name);

        // If we have a folder with an asset, don't consider it as a component
        // By this point, any language and date, as well as extension has been stripped from name,
        // so checking it is easy.
        if !components.is_empty() && name == "index" {
            if date.is_none() {
                date = get_date(parent.file_name().unwrap().to_str().unwrap()).1;
            }

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
            maybe_lang,
            date,
        }
    }

    pub fn new_section(path: &Path, base_path: &PathBuf) -> FileInfo {
        let file_path = path.to_path_buf();
        let parent = path.parent().expect("Get parent of section").to_path_buf();
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let components =
            find_content_components(&file_path.strip_prefix(base_path).unwrap_or(&file_path));
        assert!(!name.is_empty());
        let relative = if !components.is_empty() {
            format!("{}/{}.md", components.join("/"), name)
        } else {
            format!("{}.md", name)
        };
        let grand_parent = parent.parent().map(|p| p.to_path_buf());

        let (name, maybe_lang) = get_language(name);

        FileInfo {
            filename: file_path.file_name().unwrap().to_string_lossy().to_string(),
            path: file_path,
            canonical: parent.join(&name),
            parent,
            grand_parent,
            name,
            components,
            relative,
            maybe_lang,
            date: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;

    #[test]
    fn can_find_content_components() {
        let res =
            find_content_components("/home/vincent/code/site/content/posts/tutorials/python.md");
        assert_eq!(res, ["posts".to_string(), "tutorials".to_string()]);
    }

    #[test]
    fn can_find_components_in_page_with_assets() {
        let file = FileInfo::new_page(
            &Path::new("/home/vincent/code/site/content/posts/tutorials/python/index.md"),
            &PathBuf::new(),
        );
        assert_eq!(file.components, ["posts".to_string(), "tutorials".to_string()]);
    }

    #[test]
    fn doesnt_fail_with_multiple_content_directories() {
        let file = FileInfo::new_page(
            &Path::new("/home/vincent/code/content/site/content/posts/tutorials/python/index.md"),
            &PathBuf::from("/home/vincent/code/content/site"),
        );
        assert_eq!(file.components, ["posts".to_string(), "tutorials".to_string()]);
    }

    #[test]
    fn can_get_language_in_page() {
        let file = FileInfo::new_page(
            &Path::new("/home/vincent/code/site/content/posts/tutorials/python.fr.md"),
            &PathBuf::new(),
        );
        assert_eq!(file.maybe_lang.unwrap(), "fr");
        assert_eq!(file.name, "python");
    }

    #[test]
    fn can_get_language_in_page_with_assets() {
        let file = FileInfo::new_page(
            &Path::new("/home/vincent/code/site/content/posts/tutorials/python/index.fr.md"),
            &PathBuf::new(),
        );
        assert_eq!(file.components, ["posts".to_string(), "tutorials".to_string()]);
        assert_eq!(file.maybe_lang.unwrap(), "fr");
        assert_eq!(file.name, "index");
    }

    #[test]
    fn doesnt_fail_on_no_language_in_page() {
        let file = FileInfo::new_page(
            &Path::new("/home/vincent/code/site/content/posts/tutorials/python/index.md"),
            &PathBuf::new(),
        );
        assert!(file.maybe_lang.is_none());
        assert_eq!(file.name, "index");
    }

    #[test]
    fn doesnt_fail_on_no_language_in_page_with_assets() {
        let file = FileInfo::new_page(
            &Path::new("/home/vincent/code/content/site/content/posts/tutorials/python/index.md"),
            &PathBuf::from("/home/vincent/code/content/site"),
        );
        assert_eq!(file.components, ["posts".to_string(), "tutorials".to_string()]);
        assert!(file.maybe_lang.is_none());
        assert_eq!(file.name, "index");
    }

    #[test]
    fn can_get_language_in_section() {
        let file = FileInfo::new_section(
            &Path::new("/home/vincent/code/site/content/posts/tutorials/_index.fr.md"),
            &PathBuf::new(),
        );
        assert_eq!(file.maybe_lang.unwrap(), "fr");
        assert_eq!(file.name, "_index");
    }

    #[test]
    fn can_get_short_date_in_page() {
        let file = FileInfo::new_page(&Path::new("2018-10-08_hello.md"), &PathBuf::new());
        assert_eq!(file.date, Some("2018-10-08".to_string()));
        assert_eq!(file.name, "hello");
    }

    #[test]
    fn can_get_full_rfc3339_date_in_page() {
        let file = FileInfo::new_page(&Path::new("2018-10-02T15:00:00Z-hello.md"), &PathBuf::new());
        assert_eq!(file.date, Some("2018-10-02T15:00:00Z".to_string()));
        assert_eq!(file.name, "hello");
    }

    #[test]
    fn cannot_get_date_in_section() {
        let file =
            FileInfo::new_section(&Path::new("2018-10-02T15:00:00Z_index.md"), &PathBuf::new());
        assert!(file.date.is_none());
        assert_eq!(file.name, "2018-10-02T15:00:00Z_index");
    }

    #[test]
    fn can_get_lang_and_short_date_in_page() {
        let file = FileInfo::new_page(&Path::new("2018-10-08-hello.fr.md"), &PathBuf::new());
        assert_eq!(file.name, "hello");
        assert_eq!(file.maybe_lang, Some("fr".to_string()));
        assert_eq!(file.date, Some("2018-10-08".to_string()));
    }

    #[test]
    fn can_get_lang_and_full_rfc3339_date_in_page() {
        let file =
            FileInfo::new_page(&Path::new("2018-10-02T15:00:00Z-hello.fr.md"), &PathBuf::new());
        assert_eq!(file.name, "hello");
        assert_eq!(file.maybe_lang, Some("fr".to_string()));
        assert_eq!(file.date, Some("2018-10-02T15:00:00Z".to_string()));
    }

    /// Regression test for https://github.com/getzola/zola/issues/854
    #[test]
    fn correct_canonical_for_index() {
        let file = FileInfo::new_page(
            &Path::new("/home/vincent/code/site/content/posts/tutorials/python/index.md"),
            &PathBuf::new(),
        );
        assert_eq!(
            file.canonical,
            Path::new("/home/vincent/code/site/content/posts/tutorials/python/index")
        );
    }

    /// Regression test for https://github.com/getzola/zola/issues/854
    #[test]
    fn correct_canonical_with_language() {
        let file = FileInfo::new_page(
            &Path::new("/home/vincent/code/site/content/posts/tutorials/python/index.fr.md"),
            &PathBuf::new(),
        );
        assert_eq!(file.maybe_lang.unwrap(), "fr");
        assert_eq!(
            file.canonical,
            Path::new("/home/vincent/code/site/content/posts/tutorials/python/index")
        );
    }
}
