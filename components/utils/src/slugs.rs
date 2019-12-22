fn strip_chars(s: &str, chars: &str) -> String {
    let mut sanitized_string = s.to_string();
    sanitized_string.retain(|c| !chars.contains(c));
    sanitized_string
}

fn strip_invalid_paths_chars(s: &str) -> String {
    // NTFS forbidden characters : https://gist.github.com/doctaphred/d01d05291546186941e1b7ddc02034d3
    // Also we need to trim . from the end of filename
    let trimmed = s.trim_end_matches(|c| c == ' ' || c == '.');
    let cleaned = trimmed.replace(" ", "_");
    // And () [] since they are not allowed in markdown links
    strip_chars(&cleaned, "<>:/|?*#()[]\n\"\\\r\t")
}

fn strip_invalid_anchors_chars(s: &str) -> String {
    // spaces are not valid in markdown links
    let cleaned = s.replace(" ", "_");
    // https://tools.ietf.org/html/rfc3986#section-3.5
    strip_chars(&cleaned, "\"#%<>[\\]()^`{|}")
}

pub fn maybe_slugify_paths(s: &str, slugify: bool) -> String {
    if slugify {
        // ASCII slugification
        slug::slugify(s)
    } else {
        // Only remove forbidden characters
        strip_invalid_paths_chars(s)
    }
}

pub fn maybe_slugify_anchors(s: &str, slugify: bool) -> String {
    if slugify {
        // ASCII slugification
        slug::slugify(s)
    } else {
        // Only remove forbidden characters
        strip_invalid_anchors_chars(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_invalid_paths_chars_works() {
        let tests = vec![
            // no newlines
            ("test\ntest", "testtest"),
            // no whitespaces
            ("test ", "test"),
            ("t est ", "t_est"),
            // invalid NTFS
            ("test .", "test"),
            ("test. ", "test"),
            ("test#test/test?test", "testtesttesttest"),
            // Invalid CommonMark chars in links
            ("test (hey)", "test_hey"),
            ("test (hey", "test_hey"),
            ("test hey)", "test_hey"),
            ("test [hey]", "test_hey"),
            ("test [hey", "test_hey"),
            ("test hey]", "test_hey"),
            // UTF-8
            ("日本", "日本"),
        ];

        for (input, expected) in tests {
            assert_eq!(strip_invalid_paths_chars(&input), expected);
        }
    }

    #[test]
    fn strip_invalid_anchors_chars_works() {
        let tests = vec![
            ("日本", "日本"),
            // Some invalid chars get removed
            ("test#", "test"),
            ("test<", "test"),
            ("test%", "test"),
            ("test^", "test"),
            ("test{", "test"),
            ("test|", "test"),
            ("test(", "test"),
            // Spaces are replaced by `_`
            ("test hey", "test_hey"),
        ];

        for (input, expected) in tests {
            assert_eq!(strip_invalid_anchors_chars(&input), expected);
        }
    }

    #[test]
    fn maybe_slugify_paths_enabled() {
        assert_eq!(maybe_slugify_paths("héhé", true), "hehe");
    }

    #[test]
    fn maybe_slugify_paths_disabled() {
        assert_eq!(maybe_slugify_paths("héhé", false), "héhé");
    }
}
