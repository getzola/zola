pub use slug::slugify;

pub fn strip_chars(s: &str, chars: &str) -> String {
    let mut sanitized_string = s.to_string();
    sanitized_string.retain( |c| !chars.contains(c));
    sanitized_string
}

pub fn quasi_slugify(s: &str) -> String {
    // NTFS forbidden characters : https://gist.github.com/doctaphred/d01d05291546186941e1b7ddc02034d3
    strip_chars(s, "<>:/|?*#\n\"\\")
}

pub fn maybe_slugify(s: &str, enabled: bool) -> String {
    if enabled == true {
        return slugify(s);
    }
    return quasi_slugify(s);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maybe_slugify_enabled() {
        assert_eq!(maybe_slugify("héhé", true), "hehe");
    }

    #[test]
    fn maybe_slugify_disabled() {
        assert_eq!(maybe_slugify("héhé", false), "héhé");
    }

    #[test]
    fn quasi_slugify_strips_bad_symbols() {
        assert_eq!(quasi_slugify("test#test/test?test"), "testtesttesttest");
    }

    #[test]
    fn quasi_slugify_strips_newline() {
        assert_eq!(
            quasi_slugify("test
test"),
            "testtest"
        );
    }
}
