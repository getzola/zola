#![allow(dead_code)]
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

use path_slash::PathExt;
use site::Site;
use tempfile::{tempdir, TempDir};

// 2 helper macros to make all the build testing more bearable
#[macro_export]
macro_rules! file_exists {
    ($root: expr, $path: expr) => {{
        let mut path = $root.clone();
        for component in $path.split("/") {
            path = path.join(component);
        }
        std::path::Path::new(&path).exists()
    }};
}

#[macro_export]
macro_rules! file_contains {
    ($root: expr, $path: expr, $text: expr) => {{
        use std::io::prelude::*;
        let mut path = $root.clone();
        for component in $path.split("/") {
            path = path.join(component);
        }
        let mut file = std::fs::File::open(&path).expect(&format!("Failed to open {:?}", $path));
        let mut s = String::new();
        file.read_to_string(&mut s).unwrap();
        println!("{}", s);
        s.contains($text)
    }};
}

/// We return the tmpdir otherwise it would get out of scope and be deleted
/// The tests can ignore it if they dont need it by prefixing it with a `_`
pub fn build_site(name: &str) -> (Site, TempDir, PathBuf) {
    let mut path = env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    path.push(name);
    let config_file = path.join("config.toml");
    let mut site = Site::new(&path, &config_file).unwrap();
    site.load().unwrap();
    let tmp_dir = tempdir().expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    site.build().expect("Couldn't build the site");
    (site, tmp_dir, public.clone())
}

/// Same as `build_site` but has a hook to setup some config options
pub fn build_site_with_setup<F>(name: &str, mut setup_cb: F) -> (Site, TempDir, PathBuf)
where
    F: FnMut(Site) -> (Site, bool),
{
    let mut path = env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    path.push(name);
    let config_file = path.join("config.toml");
    let site = Site::new(&path, &config_file).unwrap();
    let (mut site, needs_loading) = setup_cb(site);
    if needs_loading {
        site.load().unwrap();
    }
    let tmp_dir = tempdir().expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    site.build().expect("Couldn't build the site");
    (site, tmp_dir, public.clone())
}

/// Finds the unified path (eg. _index.fr.md -> _index.md) and
/// potential language (if not default) associated with a path
/// When the path is not a markdown file (.md), None is returned
/// Strips base_dir from the start of path
fn find_lang_for(entry: &Path, base_dir: &Path) -> Option<(String, Option<String>)> {
    let ext = entry.extension();
    if ext.is_none() {
        // Not a markdown file (no extension), skip
        return None;
    }
    let ext = ext.unwrap();
    if ext != "md" {
        // Not a markdown file, skip
        return None;
    }
    let mut no_ext = entry.to_path_buf();
    let stem = entry.file_stem().unwrap();
    // Remove .md
    no_ext.pop();
    no_ext.push(stem);
    if let Some(lang) = no_ext.extension() {
        let stem = no_ext.file_stem();
        // Remove lang
        let mut unified_path = no_ext.clone();
        unified_path.pop();
        // Readd stem with .md added
        unified_path.push(&format!("{}.md", stem.unwrap().to_str().unwrap()));
        let unified_path_str = match unified_path.strip_prefix(base_dir) {
            Ok(path_without_prefix) => path_without_prefix.to_slash_lossy(),
            _ => unified_path.to_slash_lossy(),
        };
        return Some((unified_path_str, Some(lang.to_str().unwrap().into())));
    } else {
        // No lang, return no_ext directly
        let mut no_ext_string = match no_ext.strip_prefix(base_dir) {
            Ok(path_without_prefix) => path_without_prefix.to_slash_lossy(),
            _ => no_ext.to_slash_lossy(),
        };
        no_ext_string.push_str(".md");
        return Some((no_ext_string, None));
    }
}

/// Recursively process a folder to find translations, returning a list of every language
/// translated for every page found. Translations for the default language are stored as "DEFAULT"
/// TODO: This implementation does not support files with a dot inside (foo.bar.md where bar is
/// not a language), because it requires to know what languages are enabled from config, and it's
/// unclear how to distinguish (and what to do) between disabled language or "legit" dots
pub fn add_translations_from(
    dir: &Path,
    strip: &Path,
    default: &str,
) -> HashMap<String, Vec<String>> {
    let mut expected: HashMap<String, Vec<String>> = HashMap::new();
    for entry in dir.read_dir().expect("Failed to read dir") {
        let entry = entry.expect("Failed to read entry").path();
        if entry.is_dir() {
            // Recurse
            expected.extend(add_translations_from(&entry, strip, default));
        }
        if let Some((unified_path, lang)) = find_lang_for(&entry, strip) {
            if let Some(index) = expected.get_mut(&unified_path) {
                // Insert found lang for rel_path, or DEFAULT otherwise
                index.push(lang.unwrap_or(default.to_string()));
            } else {
                // rel_path is not registered yet, insert it in expected
                expected.insert(unified_path, vec![lang.unwrap_or(default.to_string())]);
            }
        } else {
            // Not a markdown file, skip
            continue;
        }
    }
    return expected;
}

/// Calculate output path for Markdown files
/// respecting page/section `path` fields, but not aliases (yet)
/// Returns a mapping of unified Markdown paths -> translations
pub fn find_expected_translations(
    name: &str,
    default_language: &str,
) -> HashMap<String, Vec<String>> {
    let mut path = env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    path.push(name);
    path.push("content");

    // Find expected translations from content folder
    // We remove BASEDIR/content/ from the keys so they match paths in library
    let mut strip_prefix = path.to_str().unwrap().to_string();
    strip_prefix.push('/');
    add_translations_from(&path, &path, default_language)
}

/// Checks whether a given permalink has a corresponding HTML page in output folder
pub fn ensure_output_exists(outputdir: &Path, baseurl: &str, link: &str) -> bool {
    // Remove the baseurl as well as the remaining /, otherwise path will be interpreted
    // as absolute.
    let trimmed_url = link.trim_start_matches(baseurl).trim_start_matches('/');
    let path = outputdir.join(trimmed_url);
    path.exists()
}

pub struct Translation {
    path: String,
    lang: String,
    permalink: String,
}

pub struct Translations {
    trans: Vec<Translation>,
}

impl Translations {
    pub fn for_path(site: &Site, path: &str) -> Translations {
        let library = site.library.clone();
        let library = library.read().unwrap();
        // WORKAROUND because site.content_path is private
        let unified_path = if let Some(page) =
            library.get_page(site.base_path.join("content").join(path))
        {
            page.file.canonical.clone()
        } else if let Some(section) = library.get_section(site.base_path.join("content").join(path))
        {
            section.file.canonical.clone()
        } else {
            panic!("No such page or section: {}", path);
        };

        let translations = library.translations.get(&unified_path);
        if translations.is_none() {
            println!(
                "Page canonical path {} is not in library translations",
                unified_path.display()
            );
            panic!("Library error");
        }

        let translations = translations
            .unwrap()
            .iter()
            .map(|key| {
                // Are we looking for a section? (no file extension here)
                if unified_path.ends_with("_index") {
                    //library.get_section_by_key(*key).file.relative.to_string()
                    let section = library.get_section_by_key(*key);
                    Translation {
                        lang: section.lang.clone(),
                        permalink: section.permalink.clone(),
                        path: section.file.path.to_str().unwrap().to_string(),
                    }
                } else {
                    let page = library.get_page_by_key(*key);
                    Translation {
                        lang: page.lang.clone(),
                        permalink: page.permalink.clone(),
                        path: page.file.path.to_str().unwrap().to_string(),
                    }
                    //library.get_page_by_key(*key).file.relative.to_string()
                }
            })
            .collect();

        Translations { trans: translations }
    }

    pub fn languages(&self) -> Vec<String> {
        let mut lang: Vec<String> = self.trans.iter().map(|x| x.lang.clone()).collect();
        lang.sort_unstable();
        lang
    }

    pub fn permalinks(&self) -> Vec<String> {
        let mut links: Vec<String> = self.trans.iter().map(|x| x.permalink.clone()).collect();
        links.sort_unstable();
        links
    }

    pub fn paths(&self) -> Vec<String> {
        let mut paths: Vec<String> = self.trans.iter().map(|x| x.path.clone()).collect();
        paths.sort_unstable();
        paths
    }
}

/// Find translations in library for a single path
fn library_translations_lang_for(site: &Site, path: &str) -> Vec<String> {
    let library_translations = Translations::for_path(site, path);
    library_translations.languages()
}

/// This function takes a list of translations generated by find_expected_translations(),
/// a site instance, and a path of a page to check that translations are the same on both sides
pub fn ensure_translations_match(
    translations: &HashMap<String, Vec<String>>,
    site: &Site,
    path: &str,
) -> bool {
    let library_page_translations = library_translations_lang_for(site, path);

    if let Some((unified_path, _lang)) = find_lang_for(&PathBuf::from(path), Path::new("")) {
        if let Some(page_translations) = translations.get(&unified_path) {
            // We order both claimed translations so we can compare them
            // library_page_translations is already ordered
            let mut page_translations = page_translations.clone();
            page_translations.sort_unstable();

            if page_translations != library_page_translations {
                // Some translations don't match, print some context
                // There is a special case where the index page may be autogenerated for a lang
                // by zola so if we are looking at the index page, library may contain more (not
                // less) languages than our tests.
                if unified_path == "_index.md" {
                    for lang in &page_translations {
                        if !library_page_translations.contains(lang) {
                            println!(
                                "Library is missing language: {} for page {}",
                                lang, unified_path
                            );
                            return false;
                        }
                    }
                    // All languages from Markdown were found. We don't care if the library
                    // auto-generated more.
                    return true;
                }
                println!("Translations don't match for {}:", path);
                println!("  - library: {:?}", library_page_translations);
                println!("  - tests: {:?}", page_translations);
                return false;
            }
            // Everything went well
            return true;
        } else {
            // Should never happen because even the default language counts as a translation
            // Reaching here means either there is a logic error in the tests themselves,
            // or the permalinks contained a page which does not exist for some reason
            unreachable!("Translations not found for {}", unified_path);
        }
    } else {
        // None means the page does not end with .md. Only markdown pages should be passed to this function.
        // Maybe a non-markdown path was found in site's permalinks?
        unreachable!("{} is not a markdown page (extension not .md)", path);
    }
}

/// For a given URL (from the permalinks), find the corresponding output page
/// and ensure all translation permalinks are linked inside
pub fn ensure_translations_in_output(site: &Site, path: &str, permalink: &str) -> bool {
    let library_page_translations = Translations::for_path(site, path);
    let translations_permalinks = library_page_translations.permalinks();

    let output_path = permalink.trim_start_matches(&site.config.base_url);
    // Strip leading / so it's not interpreted as an absolute path
    let output_path = output_path.trim_start_matches('/');
    // Don't forget to remove / because
    let output_path = site.output_path.join(output_path);

    let output = std::fs::read_to_string(&output_path)
        .expect(&format!("Output not found in {}", output_path.display()));

    for permalink in &translations_permalinks {
        if !output.contains(permalink) {
            println!("Page {} has translation {}, but it was not found in output", path, permalink);
            return false;
        }
    }

    return true;
}
