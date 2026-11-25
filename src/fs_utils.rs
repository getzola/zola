//! Utilities to simplify working with events raised by the `notify*` family of file system
//! event-watching libraries.

use ahash::HashMap;
use fs_err as fs;
use globset::GlobSet;
use notify_debouncer_full::DebouncedEvent;
use notify_debouncer_full::notify::event::*;
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
    /// A change in one of the extra paths to watch provided by the user.
    ExtraPath,
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
    // Prefer having match arms for all specific event types, so that on a notify crate upgrade, we're forced to
    // evaluate any changes and decide what corresponding mapping changes should happen.
    match event_kind {
        // Nova on macOS reports this as its final event on change
        EventKind::Modify(ModifyKind::Name(RenameMode::Any)) => {
            Some(SimpleFileSystemEventKind::Modify)
        }
        // Windows only reports `Any`. Some filesystems also report `Other` in some situations.
        EventKind::Create(CreateKind::Any | CreateKind::File | CreateKind::Folder | CreateKind::Other) => {
            Some(SimpleFileSystemEventKind::Create)
        }
        EventKind::Modify(ModifyKind::Data(DataChange::Any | DataChange::Size | DataChange::Content | DataChange::Other))
        // Windows only reports modify events at the `Any` granularity.
        | EventKind::Modify(ModifyKind::Any)
        // Intellij modifies file metadata on edit.
        // https://github.com/passcod/notify/issues/150#issuecomment-494912080
        | EventKind::Modify(ModifyKind::Metadata(MetadataKind::WriteTime | MetadataKind::Permissions | MetadataKind::Ownership))
        | EventKind::Modify(ModifyKind::Name(RenameMode::To)) => Some(SimpleFileSystemEventKind::Modify),
        // Windows only reports remove events at the `Any` granularity.
        // `Other` sometimes represents unmount events in some filesystems.
        EventKind::Remove(RemoveKind::Any | RemoveKind::File | RemoveKind::Folder | RemoveKind::Other)
        | EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
            Some(SimpleFileSystemEventKind::Remove)
        }
        EventKind::Any |
        // `Access` events according to docs are non-mutating, so it's safe to catch-all ignore them.
        EventKind::Access(_) |
        EventKind::Modify(
            // `RenameMode::Both` carries `[from, to]` and is expanded directly in `filter_events`.
            ModifyKind::Name(RenameMode::Both | RenameMode::Other) |
            ModifyKind::Metadata(MetadataKind::Any | MetadataKind::AccessTime | MetadataKind::Extended | MetadataKind::Other) |
            ModifyKind::Other) |
        EventKind::Other => None,
    }
}

pub fn filter_events(
    mut events: Vec<DebouncedEvent>,
    root_dir: &Path,
    config_path: &Path,
    ignored_content_globset: &Option<GlobSet>,
) -> HashMap<ChangeKind, Vec<MeaningfulEvent>> {
    // Arrange events from oldest to newest.
    events.sort_by_key(|e1| e1.time);

    // Use a map to keep only the last event that occurred for a particular path.
    // Map `full_path -> (partial_path, simple_event_kind, change_kind)`.
    let mut meaningful_events: HashMap<PathBuf, (PathBuf, SimpleFileSystemEventKind, ChangeKind)> =
        HashMap::default();

    for event in events.iter() {
        log::debug!(target: "file_change", "file change event: {:?}", event);

        let path_events = match &event.event.kind {
            EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
                if event.event.paths.len() != 2 {
                    log::error!(
                        "Skipping unsupported rename event with {} paths: {:?}",
                        event.event.paths.len(),
                        event.event.kind
                    );
                    continue;
                }
                [
                    Some((event.event.paths[0].clone(), SimpleFileSystemEventKind::Remove)),
                    Some((event.event.paths[1].clone(), SimpleFileSystemEventKind::Modify)),
                ]
            }
            kind => {
                let Some(simple_kind) = get_relevant_event_kind(kind) else {
                    continue;
                };
                // We currently only handle notify events that report a single path per event.
                if event.event.paths.len() != 1 {
                    log::error!(
                        "Skipping unsupported file system event with multiple paths: {:?}",
                        event.event.kind
                    );
                    continue;
                }
                [Some((event.event.paths[0].clone(), simple_kind)), None]
            }
        };

        for (path, simple_kind) in path_events.into_iter().flatten() {
            // Since we debounce things, some files might already not exist anymore by the
            // time we get to them. Of course, if this was a removal event, the path won't
            // exist, but we do want to consider it meaningful.
            if simple_kind != SimpleFileSystemEventKind::Remove && !path.exists() {
                log::debug!(target: "file_change", "skipping file change event because path does not exist");
                continue;
            }

            let (change_k, partial_p) = detect_change_kind(root_dir, &path, config_path);
            if change_k == ChangeKind::Content && is_ignored_file(ignored_content_globset, &path) {
                log::debug!(target: "file_change", "skipping file change event because it is ignored by content globset {:?}", ignored_content_globset);
                continue;
            }

            if is_temp_file(&path) {
                log::debug!(target: "file_change", "skipping file change event because it is a temp file");
                continue;
            }

            // We only care about changes in non-empty folders
            if path.is_dir() && is_folder_empty(&path) {
                log::debug!(target: "file_change", "skipping file change event because it is an empty dir");
                continue;
            }

            // Ignore ordinary files peer to config.toml. This assumes all other files we care
            // about are nested more deeply than config.toml or are directories peer to config.toml.
            if path != config_path && path.is_file() && path.parent() == config_path.parent() {
                log::debug!(target: "file_change", "skipping file change event because it is a peer to config.toml");
                continue;
            }

            log::debug!(target: "file_change", "tracking meaningful file change event: path {:?}, partial_p {:?}, simple_kind {:?}, change_k {:?}", path, partial_p, simple_kind, change_k);
            meaningful_events.insert(path, (partial_p, simple_kind, change_k));
        }
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

    fs::read_dir(dir).expect("Failed to read a directory to see if it was empty").next().is_none()
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
        ChangeKind::ExtraPath
    };

    (change_kind, partial_path)
}

#[cfg(test)]
mod tests {
    use notify_debouncer_full::DebouncedEvent;
    use notify_debouncer_full::notify::event::*;
    use std::path::{Path, PathBuf};
    use std::time::Instant;

    use super::{
        ChangeKind, SimpleFileSystemEventKind, detect_change_kind, filter_events,
        get_relevant_event_kind, is_temp_file,
    };

    // This test makes sure we at least have code coverage on the `notify` event kinds we care
    // about when watching the file system for site changes. This is to make sure changes to the
    // event mapping and filtering don't cause us to accidentally ignore things we care about.
    #[test]
    fn test_get_relative_event_kind() {
        let cases = [
            (EventKind::Create(CreateKind::Any), Some(SimpleFileSystemEventKind::Create)),
            (EventKind::Create(CreateKind::File), Some(SimpleFileSystemEventKind::Create)),
            (EventKind::Create(CreateKind::Folder), Some(SimpleFileSystemEventKind::Create)),
            (EventKind::Create(CreateKind::Other), Some(SimpleFileSystemEventKind::Create)),
            (EventKind::Modify(ModifyKind::Any), Some(SimpleFileSystemEventKind::Modify)),
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
                EventKind::Modify(ModifyKind::Name(RenameMode::Any)),
                Some(SimpleFileSystemEventKind::Modify),
            ),
            (
                EventKind::Modify(ModifyKind::Name(RenameMode::To)),
                Some(SimpleFileSystemEventKind::Modify),
            ),
            (
                EventKind::Modify(ModifyKind::Name(RenameMode::From)),
                Some(SimpleFileSystemEventKind::Remove),
            ),
            (EventKind::Remove(RemoveKind::Any), Some(SimpleFileSystemEventKind::Remove)),
            (EventKind::Remove(RemoveKind::File), Some(SimpleFileSystemEventKind::Remove)),
            (EventKind::Remove(RemoveKind::Folder), Some(SimpleFileSystemEventKind::Remove)),
            (EventKind::Remove(RemoveKind::Other), Some(SimpleFileSystemEventKind::Remove)),
        ];
        for (case, expected) in cases.iter() {
            let ek = get_relevant_event_kind(case);
            assert_eq!(ek, *expected, "case: {case:?}");
        }
    }

    #[test]
    fn ignored_content_only_filters_content_changes() {
        let root = Path::new("/home/vincent/site");
        let ignored = Some(
            utils::globs::build_ignore_glob_set(&vec!["*.html".to_string()], "ignored_content")
                .unwrap(),
        );
        // Remove events skip the existence check, so no files need to exist.
        let events = ["content/notes.html", "templates/index.html"]
            .into_iter()
            .map(|path| {
                let event =
                    Event::new(EventKind::Remove(RemoveKind::File)).add_path(root.join(path));
                DebouncedEvent::new(event, Instant::now())
            })
            .collect();

        let changes = filter_events(events, root, &root.join("config.toml"), &ignored);

        assert!(!changes.contains_key(&ChangeKind::Content));
        assert_eq!(changes[&ChangeKind::Templates][0].0, PathBuf::from("/templates/index.html"));
    }

    #[test]
    fn handles_rename_both_events() {
        let root =
            std::env::temp_dir().join(format!("zola_test_rename_both_{}", std::process::id()));
        let static_dir = root.join("static");
        std::fs::create_dir_all(&static_dir).unwrap();

        let old_path = static_dir.join("test.txt");
        let new_path = static_dir.join("uhoh.txt");
        std::fs::write(&new_path, "hello").unwrap();

        let event = Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::Both)))
            .add_path(old_path.clone())
            .add_path(new_path.clone());
        let events = vec![DebouncedEvent::new(event, Instant::now())];

        let changes = filter_events(events, &root, &root.join("config.toml"), &None);
        std::fs::remove_dir_all(&root).unwrap();

        let static_changes = &changes[&ChangeKind::StaticFiles];
        assert_eq!(static_changes.len(), 2);
        assert!(static_changes.contains(&(
            PathBuf::from("/").join("static").join("test.txt"),
            old_path,
            SimpleFileSystemEventKind::Remove,
        )));
        assert!(static_changes.contains(&(
            PathBuf::from("/").join("static").join("uhoh.txt"),
            new_path,
            SimpleFileSystemEventKind::Modify,
        )));
    }

    #[test]
    fn handles_rename_from_temp_file() {
        let root =
            std::env::temp_dir().join(format!("zola_test_rename_temp_{}", std::process::id()));
        let content_dir = root.join("content");
        std::fs::create_dir_all(&content_dir).unwrap();

        let tmp_path = content_dir.join("post.md.tmp");
        let dst_path = content_dir.join("post.md");
        std::fs::write(&dst_path, "title").unwrap();

        let event = Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::Both)))
            .add_path(tmp_path)
            .add_path(dst_path.clone());
        let events = vec![DebouncedEvent::new(event, Instant::now())];

        let changes = filter_events(events, &root, &root.join("config.toml"), &None);
        std::fs::remove_dir_all(&root).unwrap();

        let content_changes = &changes[&ChangeKind::Content];
        assert_eq!(content_changes.len(), 1);
        assert_eq!(
            content_changes[0],
            (
                PathBuf::from("/").join("content").join("post.md"),
                dst_path,
                SimpleFileSystemEventKind::Modify,
            )
        );
    }

    #[test]
    fn handles_rename_from_and_malformed_rename_both() {
        let root = Path::new("/home/vincent/site");
        let moved_out = root.join("content").join("old.md");

        let events = vec![
            DebouncedEvent::new(
                Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::From)))
                    .add_path(moved_out.clone()),
                Instant::now(),
            ),
            DebouncedEvent::new(
                Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::Both)))
                    .add_path(root.join("static").join("only_one.txt")),
                Instant::now(),
            ),
        ];

        let changes = filter_events(events, root, &root.join("config.toml"), &None);

        assert!(!changes.contains_key(&ChangeKind::StaticFiles));
        assert_eq!(
            changes[&ChangeKind::Content],
            vec![(
                PathBuf::from("/").join("content").join("old.md"),
                moved_out,
                SimpleFileSystemEventKind::Remove,
            )]
        );
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
            Path::new("smtp.md0HlVyu.bck"),
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
