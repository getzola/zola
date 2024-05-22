//! Utilities to simplify working with events raised by the `notify*` family of file system
//! event-watching libraries.

use notify_debouncer_full::notify::event::*;

/// This enum abstracts over the fine-grained group of enums in `notify`.
#[derive(Clone, Debug, PartialEq)]
pub enum SimpleFSEventKind {
    Create,
    Modify,
    Remove,
}

/// Filter `notify_debouncer_full` events. For events that we care about,
/// return our internal simplified representation. For events we don't care about,
/// return `None`.
pub fn get_relevant_event_kind(event_kind: &EventKind) -> Option<SimpleFSEventKind> {
    match event_kind {
        EventKind::Create(CreateKind::File) | EventKind::Create(CreateKind::Folder) => {
            Some(SimpleFSEventKind::Create)
        }
        EventKind::Modify(ModifyKind::Data(DataChange::Size))
        | EventKind::Modify(ModifyKind::Data(DataChange::Content))
        // Intellij modifies file metadata on edit.
        // https://github.com/passcod/notify/issues/150#issuecomment-494912080
        | EventKind::Modify(ModifyKind::Metadata(MetadataKind::WriteTime))
        | EventKind::Modify(ModifyKind::Metadata(MetadataKind::Permissions))
        | EventKind::Modify(ModifyKind::Metadata(MetadataKind::Ownership))
        | EventKind::Modify(ModifyKind::Name(RenameMode::To)) => Some(SimpleFSEventKind::Modify),
        EventKind::Remove(RemoveKind::File) | EventKind::Remove(RemoveKind::Folder) => {
            Some(SimpleFSEventKind::Remove)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use notify_debouncer_full::notify::event::*;

    use super::{get_relevant_event_kind, SimpleFSEventKind};

    // This test makes sure we at least have code coverage on the `notify` event kinds we care
    // about when watching the file system for site changes. This is to make sure changes to the
    // event mapping and filtering don't cause us to accidentally ignore things we care about.
    #[test]
    fn test_get_relative_event_kind() {
        let cases = vec![
            (EventKind::Create(CreateKind::File), Some(SimpleFSEventKind::Create)),
            (EventKind::Create(CreateKind::Folder), Some(SimpleFSEventKind::Create)),
            (
                EventKind::Modify(ModifyKind::Data(DataChange::Size)),
                Some(SimpleFSEventKind::Modify),
            ),
            (
                EventKind::Modify(ModifyKind::Data(DataChange::Content)),
                Some(SimpleFSEventKind::Modify),
            ),
            (
                EventKind::Modify(ModifyKind::Metadata(MetadataKind::WriteTime)),
                Some(SimpleFSEventKind::Modify),
            ),
            (
                EventKind::Modify(ModifyKind::Metadata(MetadataKind::Permissions)),
                Some(SimpleFSEventKind::Modify),
            ),
            (
                EventKind::Modify(ModifyKind::Metadata(MetadataKind::Ownership)),
                Some(SimpleFSEventKind::Modify),
            ),
            (EventKind::Modify(ModifyKind::Name(RenameMode::To)), Some(SimpleFSEventKind::Modify)),
            (EventKind::Remove(RemoveKind::File), Some(SimpleFSEventKind::Remove)),
            (EventKind::Remove(RemoveKind::Folder), Some(SimpleFSEventKind::Remove)),
        ];
        for (case, expected) in cases.iter() {
            let ek = get_relevant_event_kind(&case);
            assert_eq!(ek, *expected);
        }
    }
}
