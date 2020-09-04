use serde_derive::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SlugifyStrategy {
    /// Classic slugification, the default
    On,
    /// No slugification, only remove unsafe characters for filepaths/urls
    Safe,
    /// Nothing is changed, hope for the best!
    Off,
}

impl Default for SlugifyStrategy {
    fn default() -> Self {
        SlugifyStrategy::On
    }
}

fn strip_chars(s: &str, chars: &str) -> String {
    let mut sanitized_string = s.to_string();
    sanitized_string.retain(|c| !chars.contains(c));
    sanitized_string
}

fn strip_invalid_paths_chars(s: &str) -> String {
    // NTFS forbidden characters : https://gist.github.com/doctaphred/d01d05291546186941e1b7ddc02034d3
    // Also we need to trim whitespaces and `.` from the end of filename
    let trimmed = s.trim_end_matches(|c| c == ' ' || c == '.');
    strip_chars(&trimmed, r#"<>:"/\|?*"#)
}

pub fn slugify_paths(s: &str, strategy: SlugifyStrategy) -> String {
    match strategy {
        SlugifyStrategy::On => slug::slugify(s),
        SlugifyStrategy::Safe => strip_invalid_paths_chars(s),
        SlugifyStrategy::Off => s.to_string(),
    }
}

pub fn slugify_anchors(s: &str, strategy: SlugifyStrategy) -> String {
    match strategy {
        SlugifyStrategy::On => slug::slugify(s),
        SlugifyStrategy::Safe | SlugifyStrategy::Off => {
            s.replace(|c: char| c.is_ascii_whitespace(), "_")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_slugify_paths() {
        let tests = vec![
            // input, (on, safe, off)
            ("input", ("input", "input", "input")),
            ("test ", ("test", "test", "test ")),
            ("tes t", ("tes-t", "tes t", "tes t")),
            // Invalid NTFS
            ("dot. ", ("dot", "dot", "dot. ")),
            ("日本", ("ri-ben", "日本", "日本")),
            ("héhé", ("hehe", "héhé", "héhé")),
            ("test (hey)", ("test-hey", "test (hey)", "test (hey)")),
        ];

        for (input, (on, safe, off)) in tests {
            assert_eq!(on, slugify_paths(input, SlugifyStrategy::On));
            assert_eq!(safe, slugify_paths(input, SlugifyStrategy::Safe));
            assert_eq!(off, slugify_paths(input, SlugifyStrategy::Off));
        }
    }

    #[test]
    fn can_slugify_anchors() {
        let tests = vec![
            // input, (on, safe, off)
            ("input", ("input", "input", "input")),
            ("test ", ("test", "test_", "test_")),
            ("tes t", ("tes-t", "tes_t", "tes_t")),
            // Invalid NTFS
            ("dot. ", ("dot", "dot._", "dot._")),
            ("日本", ("ri-ben", "日本", "日本")),
            ("héhé", ("hehe", "héhé", "héhé")),
            ("test (hey)", ("test-hey", "test_(hey)", "test_(hey)")),
        ];

        for (input, (on, safe, off)) in tests {
            assert_eq!(on, slugify_anchors(input, SlugifyStrategy::On));
            assert_eq!(safe, slugify_anchors(input, SlugifyStrategy::Safe));
            assert_eq!(off, slugify_anchors(input, SlugifyStrategy::Off));
        }
    }
}
