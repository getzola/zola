fn strip_chars(s: &str, chars: &str) -> String {
    let mut sanitized_string = s.to_string();
    sanitized_string.retain( |c| !chars.contains(c));
    sanitized_string
}

fn quasi_slugify(s: &str) -> String {
    // NTFS forbidden characters : https://gist.github.com/doctaphred/d01d05291546186941e1b7ddc02034d3
    // Also we need to trim . from the end of filename
    let trimmed = s.trim_end_matches(|c| c == ' ' || c == '.');
    // Also forbids whitespace to avoid users having to use %20 in .md
    // And () [] since they are not allowed in markdown links
    strip_chars(trimmed, "<>:/|?*#()[] \n\"\\\r\t")
}

pub fn maybe_slugify(s: &str, slugify: bool) -> String {
    if slugify {
        // ASCII slugification
        slug::slugify(s)
    }
    else {
        // Only remove forbidden characters
        quasi_slugify(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quasi_slugify_works() {
        let tests = vec![
            // no newlines
            ("test\ntest", "testtest"),
            // no whitespaces
            ("test ", "test"),
            ("t est ", "test"),
            // invalid NTFS
            ("test .", "test"),
            ("test. ", "test"),
            ("test#test/test?test", "testtesttesttest"),
            // Invalid CommonMark chars in links
            ("test (hey)", "testhey"),
            ("test (hey", "testhey"),
            ("test hey)", "testhey"),
            ("test [hey]", "testhey"),
            ("test [hey", "testhey"),
            ("test hey]", "testhey"),
        ];

        for (input, expected) in tests {
            println!("Input: {:?}", input);
            assert_eq!(quasi_slugify(&input), expected);
        }
    }

    #[test]
    fn maybe_slugify_enabled() {
        assert_eq!(maybe_slugify("héhé", true), "hehe");
    }

    #[test]
    fn maybe_slugify_disabled() {
        assert_eq!(maybe_slugify("héhé", false), "héhé");
    }
}
