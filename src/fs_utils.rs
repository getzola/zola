//! Utilities to simplify working with events raised by the `notify*` family of file system
//! event-watching libraries.

use libs::ahash::HashMap;
use libs::globset::GlobSet;
use notify_debouncer_full::notify::event::*;
use notify_debouncer_full::DebouncedEvent;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use utils::fs::is_temp_file;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ChangeKind {
    Content,
    Templates,
    Themes,
    StaticFiles,
    Sass,
    Config,
}

/// This enum abstracts over the fine-grained group of enums in `notify`.
#[derive(Clone, Debug, PartialEq)]
pub enum SimpleFileSystemEventKind {
    Create,
    Modify,
    Remove,
}

// (partial path, full path, ..)
pub type MeaningfulEvent = (PathBuf, PathBuf, SimpleFileSystemEventKind);

/// Filter `notify_debouncer_full` events. For events that we care about,
/// return our internal simplified representation. For events we don't care about,
/// return `None`.
fn get_relevant_event_kind(event_kind: &EventKind) -> Option<SimpleFileSystemEventKind> {
    match event_kind {
        EventKind::Create(CreateKind::File) | EventKind::Create(CreateKind::Folder) => {
            Some(SimpleFileSystemEventKind::Create)
        }
        EventKind::Modify(ModifyKind::Data(_))
        // Intellij modifies file metadata on edit.
        // https://github.com/passcod/notify/issues/150#issuecomment-494912080
        | EventKind::Modify(ModifyKind::Metadata(MetadataKind::WriteTime))
        | EventKind::Modify(ModifyKind::Metadata(MetadataKind::Permissions))
        | EventKind::Modify(ModifyKind::Metadata(MetadataKind::Ownership))
        | EventKind::Modify(ModifyKind::Name(RenameMode::To)) => Some(SimpleFileSystemEventKind::Modify),
        EventKind::Remove(RemoveKind::File) | EventKind::Remove(RemoveKind::Folder) => {
            Some(SimpleFileSystemEventKind::Remove)
        }
        _ => None,
    }
}

pub fn filter_events(
    mut events: Vec<DebouncedEvent>,
    root_dir: &Path,
    config_path: &Path,
    ignored_content_globset: &Option<GlobSet>,
) -> HashMap<ChangeKind, Vec<MeaningfulEvent>> {
    // Arrange events from oldest to newest.
    events.sort_by(|e1, e2| e1.time.cmp(&e2.time));

    // Use a map to keep only the last event that occurred for a particular path.
    // Map `full_path -> (partial_path, simple_event_kind, change_kind)`.
    let mut meaningful_events: HashMap<PathBuf, (PathBuf, SimpleFileSystemEventKind, ChangeKind)> =
        HashMap::default();

    for event in events.iter() {
        let simple_kind = get_relevant_event_kind(&event.event.kind);
        if simple_kind.is_none() {
            continue;
        }

        // We currently only handle notify events that report a single path per event.
        if event.event.paths.len() != 1 {
            console::error(&format!(
                "Skipping unsupported file system event with multiple paths: {:?}",
                event.event.kind
            ));
            continue;
        }
        let path = event.event.paths[0].clone();

        if is_ignored_file(ignored_content_globset, &path) {
            continue;
        }

        if is_temp_file(&path) {
            continue;
        }

        // We only care about changes in non-empty folders
        if path.is_dir() && is_folder_empty(&path) {
            continue;
        }

        // Ignore ordinary files peer to config.toml. This assumes all other files we care
        // about are nested more deeply than config.toml or are directories peer to config.toml.
        if path != config_path && path.is_file() && path.parent() == config_path.parent() {
            continue;
        }

        let (change_k, partial_p) = detect_change_kind(root_dir, &path, config_path);
        meaningful_events.insert(path, (partial_p, simple_kind.unwrap(), change_k));
    }

    // Bin changes by change kind to support later iteration over batches of changes.
    let mut changes = HashMap::default();
    for (full_path, (partial_path, event_kind, change_kind)) in meaningful_events.into_iter() {
        let c = changes.entry(change_kind).or_insert(vec![]);
        c.push((partial_path, full_path, event_kind));
    }

    changes
}

fn is_ignored_file(ignored_content_globset: &Option<GlobSet>, path: &Path) -> bool {
    match ignored_content_globset {
        Some(gs) => gs.is_match(path),
        None => false,
    }
}

/// Check if the directory at path contains any file
fn is_folder_empty(dir: &Path) -> bool {
    // Can panic if we don't have the rights I guess?

    read_dir(dir).expect("Failed to read a directory to see if it was empty").next().is_none()
}

/// Detect what changed from the given path so we have an idea what needs
/// to be reloaded
fn detect_change_kind(pwd: &Path, path: &Path, config_path: &Path) -> (ChangeKind, PathBuf) {
    let mut partial_path = PathBuf::from("/");
    partial_path.push(path.strip_prefix(pwd).unwrap_or(path));

    let change_kind = if partial_path.starts_with("/templates") {
        ChangeKind::Templates
    } else if partial_path.starts_with("/themes") {
        ChangeKind::Themes
    } else if partial_path.starts_with("/content") {
        ChangeKind::Content
    } else if partial_path.starts_with("/static") {
        ChangeKind::StaticFiles
    } else if partial_path.starts_with("/sass") {
        ChangeKind::Sass
    } else if path == config_path {
        ChangeKind::Config
    } else {
        unreachable!("Got a change in an unexpected path: {}", partial_path.display());
    };

    (change_kind, partial_path)
}

#[cfg(test)]
mod tests {
    use notify_debouncer_full::notify::event::*;
    use std::path::{Path, PathBuf};

    use super::{
        detect_change_kind, get_relevant_event_kind, is_temp_file, ChangeKind,
        SimpleFileSystemEventKind,
    };

    // This test makes sure we at least have code coverage on the `notify` event kinds we care
    // about when watching the file system for site changes. This is to make sure changes to the
    // event mapping and filtering don't cause us to accidentally ignore things we care about.
    #[test]
    fn test_get_relative_event_kind() {
        let cases = vec![
            (EventKind::Create(CreateKind::File), Some(SimpleFileSystemEventKind::Create)),
            (EventKind::Create(CreateKind::Folder), Some(SimpleFileSystemEventKind::Create)),
            (
                EventKind::Modify(ModifyKind::Data(DataChange::Size)),
                Some(SimpleFileSystemEventKind::Modify),
            ),
            (
                EventKind::Modify(ModifyKind::Data(DataChange::Content)),
                Some(SimpleFileSystemEventKind::Modify),
            ),
            (
                EventKind::Modify(ModifyKind::Data(DataChange::Any)),
                Some(SimpleFileSystemEventKind::Modify),
            ),
            (
                EventKind::Modify(ModifyKind::Data(DataChange::Other)),
                Some(SimpleFileSystemEventKind::Modify),
            ),
            (
                EventKind::Modify(ModifyKind::Metadata(MetadataKind::WriteTime)),
                Some(SimpleFileSystemEventKind::Modify),
            ),
            (
                EventKind::Modify(ModifyKind::Metadata(MetadataKind::Permissions)),
                Some(SimpleFileSystemEventKind::Modify),
            ),
            (
                EventKind::Modify(ModifyKind::Metadata(MetadataKind::Ownership)),
                Some(SimpleFileSystemEventKind::Modify),
            ),
            (
                EventKind::Modify(ModifyKind::Name(RenameMode::To)),
                Some(SimpleFileSystemEventKind::Modify),
            ),
            (EventKind::Remove(RemoveKind::File), Some(SimpleFileSystemEventKind::Remove)),
            (EventKind::Remove(RemoveKind::Folder), Some(SimpleFileSystemEventKind::Remove)),
        ];
        for (case, expected) in cases.iter() {
            let ek = get_relevant_event_kind(&case);
            assert_eq!(ek, *expected, "case: {:?}", case);
        }
    }

    #[test]
    fn can_recognize_temp_files() {
        let test_cases = vec![
            Path::new("hello.swp"),
            Path::new("hello.swx"),
            Path::new(".DS_STORE"),
            Path::new("hello.tmp"),
            Path::new("hello.html.__jb_old___"),
            Path::new("hello.html.__jb_tmp___"),
            Path::new("hello.html.__jb_bak___"),
            Path::new("hello.html~"),
            Path::new("#hello.html"),
            Path::new(".index.md.kate-swp"),
        ];

        for t in test_cases {
            assert!(is_temp_file(t));
        }
    }

    #[test]
    fn can_detect_kind_of_changes() {
        let test_cases = vec![
            (
                (ChangeKind::Templates, PathBuf::from("/templates/hello.html")),
                Path::new("/home/vincent/site"),
                Path::new("/home/vincent/site/templates/hello.html"),
                Path::new("/home/vincent/site/config.toml"),
            ),
            (
                (ChangeKind::Themes, PathBuf::from("/themes/hello.html")),
                Path::new("/home/vincent/site"),
                Path::new("/home/vincent/site/themes/hello.html"),
                Path::new("/home/vincent/site/config.toml"),
            ),
            (
                (ChangeKind::StaticFiles, PathBuf::from("/static/site.css")),
                Path::new("/home/vincent/site"),
                Path::new("/home/vincent/site/static/site.css"),
                Path::new("/home/vincent/site/config.toml"),
            ),
            (
                (ChangeKind::Content, PathBuf::from("/content/posts/hello.md")),
                Path::new("/home/vincent/site"),
                Path::new("/home/vincent/site/content/posts/hello.md"),
                Path::new("/home/vincent/site/config.toml"),
            ),
            (
                (ChangeKind::Sass, PathBuf::from("/sass/print.scss")),
                Path::new("/home/vincent/site"),
                Path::new("/home/vincent/site/sass/print.scss"),
                Path::new("/home/vincent/site/config.toml"),
            ),
            (
                (ChangeKind::Config, PathBuf::from("/config.toml")),
                Path::new("/home/vincent/site"),
                Path::new("/home/vincent/site/config.toml"),
                Path::new("/home/vincent/site/config.toml"),
            ),
            (
                (ChangeKind::Config, PathBuf::from("/config.staging.toml")),
                Path::new("/home/vincent/site"),
                Path::new("/home/vincent/site/config.staging.toml"),
                Path::new("/home/vincent/site/config.staging.toml"),
            ),
        ];

        for (expected, pwd, path, config_filename) in test_cases {
            assert_eq!(expected, detect_change_kind(pwd, path, config_filename));
        }
    }

    #[test]
    #[cfg(windows)]
    fn windows_path_handling() {
        let expected = (ChangeKind::Templates, PathBuf::from("/templates/hello.html"));
        let pwd = Path::new(r#"C:\Users\johan\site"#);
        let path = Path::new(r#"C:\Users\johan\site\templates\hello.html"#);
        let config_filename = Path::new(r#"C:\Users\johan\site\config.toml"#);
        assert_eq!(expected, detect_change_kind(pwd, path, config_filename));
    }

    #[test]
    fn relative_path() {
        let expected = (ChangeKind::Templates, PathBuf::from("/templates/hello.html"));
        let pwd = Path::new("/home/johan/site");
        let path = Path::new("templates/hello.html");
        let config_filename = Path::new("config.toml");
        assert_eq!(expected, detect_change_kind(pwd, path, config_filename));
    }
}
